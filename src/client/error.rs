use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct ApiErrorPayload {
    #[serde(rename = "_type")]
    pub error_type: Option<String>,
    #[serde(rename = "errorIdentifier")]
    pub error_identifier: Option<String>,
    pub message: Option<String>,
    #[serde(rename = "_embedded")]
    pub embedded: Option<ApiErrorEmbedded>,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorEmbedded {
    pub errors: Option<Vec<ApiErrorDetail>>,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    pub message: Option<String>,
    #[serde(rename = "errorIdentifier")]
    pub error_identifier: Option<String>,
}

#[derive(Debug, Error)]
pub enum OpenProjectError {
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Authentication failed (401 Unauthorized): {0}")]
    Unauthorized(String),

    #[error("Permission denied (403 Forbidden): {0}")]
    Forbidden(String),

    #[error("Resource not found (404 Not Found): {0}")]
    NotFound(String),

    #[error("Concurrent modification conflict (409 Conflict): {0}")]
    LockConflict(String),

    #[error("Validation error (422 Unprocessable Entity): {0}")]
    Validation(String),

    #[error("API error ({status}): {message}")]
    Api {
        status: StatusCode,
        identifier: Option<String>,
        message: String,
        details: Vec<String>,
    },

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal/URL error: {0}")]
    Url(#[from] url::ParseError),
}

impl OpenProjectError {
    pub fn from_response(status: StatusCode, body: &str) -> Self {
        // Try to parse structured HAL error
        if let Ok(err_payload) = serde_json::from_str::<ApiErrorPayload>(body) {
            let msg = err_payload
                .message
                .unwrap_or_else(|| format!("HTTP status {}", status));
            let identifier = err_payload.error_identifier;
            let mut details = Vec::new();
            if let Some(err_list) = err_payload.embedded.and_then(|e| e.errors) {
                for e in err_list {
                    if let Some(m) = e.message {
                        details.push(m);
                    }
                }
            }

            let full_message = if details.is_empty() {
                msg.clone()
            } else {
                format!("{}: {}", msg, details.join("; "))
            };

            match status {
                StatusCode::UNAUTHORIZED => Self::Unauthorized(full_message),
                StatusCode::FORBIDDEN => Self::Forbidden(full_message),
                StatusCode::NOT_FOUND => Self::NotFound(full_message),
                StatusCode::CONFLICT => Self::LockConflict(full_message),
                StatusCode::UNPROCESSABLE_ENTITY => Self::Validation(full_message),
                _ => Self::Api {
                    status,
                    identifier,
                    message: msg,
                    details,
                },
            }
        } else {
            let fallback_msg = if body.trim().is_empty() {
                format!("HTTP {}", status)
            } else {
                format!("HTTP {}: {}", status, body.trim())
            };

            match status {
                StatusCode::UNAUTHORIZED => Self::Unauthorized(fallback_msg),
                StatusCode::FORBIDDEN => Self::Forbidden(fallback_msg),
                StatusCode::NOT_FOUND => Self::NotFound(fallback_msg),
                StatusCode::CONFLICT => Self::LockConflict(fallback_msg),
                StatusCode::UNPROCESSABLE_ENTITY => Self::Validation(fallback_msg),
                _ => Self::Api {
                    status,
                    identifier: None,
                    message: fallback_msg,
                    details: Vec::new(),
                },
            }
        }
    }
}
