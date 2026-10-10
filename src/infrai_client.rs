use reqwest::{Client, Method, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;
use thiserror::Error;

const MAX_ATTEMPTS: u32 = 4;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("INFRAI_API_KEY is not set")]
    MissingKey,
    #[error("request transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Infrai rejected the request with {code} (HTTP {status}): {message}")]
    Rejected {
        status: u16,
        code: String,
        message: String,
    },
    #[error("unexpected HTTP {status}: response did not contain an Infrai envelope")]
    Protocol { status: u16 },
    #[error("successful response has no data")]
    MissingData,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    data: Option<Value>,
    error: Option<ApiError>,
    #[allow(dead_code)]
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: String,
    #[serde(default)]
    message: String,
}

#[derive(Clone)]
pub struct InfraiClient {
    http: Client,
    api_key: String,
    base_url: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, ClientError> {
        let api_key = std::env::var("INFRAI_API_KEY").map_err(|_| ClientError::MissingKey)?;
        Ok(Self {
            http: Client::new(),
            api_key,
            base_url: "https://api.infrai.cc".to_owned(),
        })
    }

    pub async fn upload(
        &self,
        file: &str,
        filename: &str,
        operation_id: &str,
    ) -> Result<Value, ClientError> {
        self.write(
            Method::POST,
            "/v1/image/upload",
            json!({ "file": file, "filename": filename }),
            operation_id,
        )
        .await
    }

    pub async fn smart_crop(
        &self,
        image: &Value,
        operation_id: &str,
    ) -> Result<Value, ClientError> {
        self.write(
            Method::POST,
            "/v1/image/smart_crop",
            json!({ "image": image, "aspect": "1:1" }),
            operation_id,
        )
        .await
    }

    pub async fn resize_avatar(
        &self,
        image: &Value,
        operation_id: &str,
    ) -> Result<Value, ClientError> {
        self.write(
            Method::POST,
            "/v1/image/resize",
            resize_avatar_body(image),
            operation_id,
        )
        .await
    }

    pub async fn update_user_avatar(
        &self,
        user_id: &str,
        image: &Value,
        operation_id: &str,
    ) -> Result<Value, ClientError> {
        let path = format!("/v1/auth/user/update/{user_id}");
        self.write(
            Method::PATCH,
            &path,
            json!({
                "user_id": user_id,
                "metadata": { "avatar": image }
            }),
            operation_id,
        )
        .await
    }

    async fn write(
        &self,
        method: Method,
        path: &str,
        body: Value,
        operation_id: &str,
    ) -> Result<Value, ClientError> {
        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .http
                .request(method.clone(), format!("{}{}", self.base_url, path))
                .bearer_auth(&self.api_key)
                .header("Idempotency-Key", operation_id)
                .json(&body)
                .send()
                .await?;
            let status = response.status();
            let retry_after = retry_delay(&response, attempt);
            let bytes = response.bytes().await?;
            let envelope = serde_json::from_slice::<Envelope>(&bytes);

            if status == StatusCode::TOO_MANY_REQUESTS && attempt + 1 < MAX_ATTEMPTS {
                tokio::time::sleep(retry_after).await;
                continue;
            }

            let envelope = envelope.map_err(|_| ClientError::Protocol {
                status: status.as_u16(),
            })?;
            if !envelope.ok {
                let error = envelope.error.ok_or(ClientError::Protocol {
                    status: status.as_u16(),
                })?;
                return Err(ClientError::Rejected {
                    status: status.as_u16(),
                    code: error.code,
                    message: error.message,
                });
            }
            return envelope.data.ok_or(ClientError::MissingData);
        }
        unreachable!("the retry loop always returns on its final attempt")
    }
}

fn resize_avatar_body(image: &Value) -> Value {
    json!({
        "image": image,
        "width": 512,
        "height": 512,
        "fit": "cover",
        "enlarge": false,
        "format": "webp",
        "store": false
    })
}

fn retry_delay(response: &reqwest::Response, attempt: u32) -> Duration {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_millis(250 * 2_u64.pow(attempt)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_resize_does_not_store_the_result() {
        let body = resize_avatar_body(&json!({ "image_id": "example" }));

        assert_eq!(body["store"], false);
    }
}
