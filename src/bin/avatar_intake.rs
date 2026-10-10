use legal_matter_avatar_pipeline::{
    InfraiClient, IntakeState, MatterAvatarRequest, MatterAvatarService, MatterState,
};
use std::{env, fs, process::ExitCode};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("avatar-intake: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        return Err("usage: avatar-intake <matter-id> <user-id> <image-file> <filename>".into());
    }

    let image_base64 = fs::read_to_string(&args[3])?;
    let service = MatterAvatarService::new(InfraiClient::from_env()?);
    let outcome = service
        .process(MatterAvatarRequest {
            matter_id: args[1].clone(),
            user_id: args[2].clone(),
            filename: args[4].clone(),
            image_base64,
            state: MatterState {
                intake: IntakeState::Complete,
                signed_document_delivered: true,
                deadline_follow_up_scheduled: true,
            },
        })
        .await?;

    println!("{}", serde_json::to_string_pretty(&outcome)?);
    Ok(())
}
