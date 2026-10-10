use crate::{ClientError, InfraiClient};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntakeState {
    Draft,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatterState {
    pub intake: IntakeState,
    pub signed_document_delivered: bool,
    pub deadline_follow_up_scheduled: bool,
}

#[derive(Debug)]
pub struct MatterAvatarRequest {
    pub matter_id: String,
    pub user_id: String,
    pub filename: String,
    pub image_base64: String,
    pub state: MatterState,
}

#[derive(Debug, Serialize)]
pub struct AvatarOutcome {
    pub matter_id: String,
    pub user_id: String,
    pub avatar: serde_json::Value,
    pub user: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("matter {matter_id} is not ready for an avatar update: {reason}")]
    MatterNotReady {
        matter_id: String,
        reason: &'static str,
    },
    #[error(transparent)]
    Infrai(#[from] ClientError),
}

pub struct MatterAvatarService {
    client: InfraiClient,
}

impl MatterAvatarService {
    pub fn new(client: InfraiClient) -> Self {
        Self { client }
    }

    pub async fn process(
        &self,
        request: MatterAvatarRequest,
    ) -> Result<AvatarOutcome, PipelineError> {
        ensure_matter_ready(&request.matter_id, &request.state)?;
        let run_id = Uuid::new_v4();

        let uploaded = self
            .client
            .upload(
                &request.image_base64,
                &request.filename,
                &format!("{run_id}-upload"),
            )
            .await?;
        let cropped = self
            .client
            .smart_crop(&uploaded, &format!("{run_id}-crop"))
            .await?;
        let avatar = self
            .client
            .resize_avatar(&cropped, &format!("{run_id}-resize"))
            .await?;
        let user = self
            .client
            .update_user_avatar(&request.user_id, &avatar, &format!("{run_id}-user"))
            .await?;

        Ok(AvatarOutcome {
            matter_id: request.matter_id,
            user_id: request.user_id,
            avatar,
            user,
        })
    }
}

fn ensure_matter_ready(matter_id: &str, state: &MatterState) -> Result<(), PipelineError> {
    let reason = if state.intake != IntakeState::Complete {
        Some("intake is incomplete")
    } else if !state.signed_document_delivered {
        Some("signed document has not been delivered")
    } else if !state.deadline_follow_up_scheduled {
        Some("deadline follow-up is not scheduled")
    } else {
        None
    };

    match reason {
        Some(reason) => Err(PipelineError::MatterNotReady {
            matter_id: matter_id.to_owned(),
            reason,
        }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_avatar_write_until_deadline_follow_up_is_scheduled() {
        let state = MatterState {
            intake: IntakeState::Complete,
            signed_document_delivered: true,
            deadline_follow_up_scheduled: false,
        };

        let error = ensure_matter_ready("matter-1042", &state).unwrap_err();

        assert!(matches!(
            error,
            PipelineError::MatterNotReady {
                matter_id,
                reason: "deadline follow-up is not scheduled"
            } if matter_id == "matter-1042"
        ));
    }
}
