use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Custom error type for the application
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    #[error("External API error: {0}")]
    ExternalApi(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::Redis(ref e) => {
                tracing::error!("Redis error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Cache error occurred")
            }
            AppError::Auth(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::ExternalApi(ref msg) => {
                tracing::error!("External API error: {}", msg);
                (StatusCode::BAD_GATEWAY, "External service error")
            }
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, AppError>;
