use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;

/// OpenAI-compatible error response
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIError {
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Voice not found: {0}")]
    VoiceNotFound(String),

    #[error("Invalid response format: {0}")]
    InvalidResponseFormat(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Internal server error")]
    Internal,
}

impl AppError {
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest(msg.into())
    }

    pub fn unsupported_format(format: impl Into<String>) -> Self {
        Self::InvalidResponseFormat(format.into())
    }

    pub fn voice_not_found(voice: impl Into<String>) -> Self {
        Self::VoiceNotFound(voice.into())
    }

    pub fn model_not_found(model: impl Into<String>) -> Self {
        Self::ModelNotFound(model.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message, param, code) = match &self {
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                self.to_string(),
                None,
                None,
            ),
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                msg.clone(),
                None,
                None,
            ),
            AppError::ModelNotFound(model) => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                format!("Model '{}' not found", model),
                Some("model".to_string()),
                None,
            ),
            AppError::VoiceNotFound(voice) => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                format!("Voice '{}' not found", voice),
                Some("voice".to_string()),
                None,
            ),
            AppError::InvalidResponseFormat(format) => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                format!(
                    "Response format '{}' not supported. Supported formats: wav, pcm",
                    format
                ),
                Some("response_format".to_string()),
                None,
            ),
            AppError::Backend(msg) => {
                error!("Backend error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "api_error",
                    "Backend processing error".to_string(),
                    None,
                    None,
                )
            }
            AppError::Internal => {
                error!("Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "api_error",
                    "Internal server error".to_string(),
                    None,
                    None,
                )
            }
        };

        let body = Json(OpenAIError {
            error: ErrorDetails {
                message,
                error_type: error_type.to_string(),
                param,
                code,
            },
        });

        (status, body).into_response()
    }
}

/// Type alias for API results
pub type ApiResult<T> = Result<T, AppError>;
