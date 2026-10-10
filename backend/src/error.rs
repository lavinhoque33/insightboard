use axum::{
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

/// One invalid request field in a 422 body. `field` is the request field name, dotted for nested
/// JSON (`layout_json.widgets.0.w`), or the query parameter name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Every failure a handler can return. Each variant renders as JSON `{"error": ...}` (422 adds
/// `details`); client-facing text is fixed and generic, causes are logged here once.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// 400: the body is not parseable JSON.
    #[error("malformed request: {0}")]
    BadRequest(&'static str),

    /// 413: the body exceeds the extractor limit.
    #[error("payload too large")]
    PayloadTooLarge,

    /// 415: missing or wrong JSON content type.
    #[error("unsupported media type")]
    UnsupportedMediaType,

    /// 422: well-formed input that is missing, mistyped, or invalid.
    #[error("validation failed: {0:?}")]
    Validation(Vec<FieldError>),

    #[error("unauthorized")]
    Unauthorized,

    #[error("not found: {0}")]
    NotFound(&'static str),

    #[error("method not allowed")]
    MethodNotAllowed,

    #[error("conflict: {0}")]
    Conflict(&'static str),

    #[error("rate limited")]
    RateLimited,

    /// 502: upstream failure; the cause is logged, never returned.
    #[error("external API error: {0}")]
    ExternalApi(String),

    /// 503: a provider key is not configured (names the variable in the log only).
    #[error("provider not configured: {0}")]
    ProviderNotConfigured(&'static str),

    /// 503: a dependency checked by `/readyz` is down.
    #[error("not ready: {0}")]
    NotReady(String),

    /// 504: the server-side request timeout elapsed.
    #[error("request timed out")]
    Timeout,

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// A 422 with a single field error.
    pub fn invalid(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation(vec![FieldError::new(field, message)])
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => {
                tracing::error!(error = ?e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, *msg),
            AppError::PayloadTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "Request body too large"),
            AppError::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Expected request with `Content-Type: application/json`",
            ),
            AppError::Validation(details) => {
                let body = json!({ "error": "Validation failed", "details": details });
                return (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response();
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, *msg),
            AppError::MethodNotAllowed => (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed"),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, *msg),
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many requests; try again later",
            ),
            AppError::ExternalApi(cause) => {
                tracing::warn!(cause = %cause, "external API error");
                (StatusCode::BAD_GATEWAY, "External service error")
            }
            AppError::ProviderNotConfigured(variable) => {
                tracing::warn!(variable = *variable, "provider key not configured");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "This widget's provider is not configured on the server",
                )
            }
            AppError::NotReady(cause) => {
                tracing::warn!(cause = %cause, "readiness check failed");
                (StatusCode::SERVICE_UNAVAILABLE, "Service not ready")
            }
            AppError::Timeout => (StatusCode::GATEWAY_TIMEOUT, "Request timed out"),
            AppError::Internal(cause) => {
                tracing::error!(cause = %cause, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::MissingJsonContentType(_) => AppError::UnsupportedMediaType,
            JsonRejection::JsonDataError(_) => AppError::invalid("body", "has the wrong shape"),
            _ if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => AppError::PayloadTooLarge,
            _ => AppError::BadRequest("Malformed JSON body"),
        }
    }
}

impl From<QueryRejection> for AppError {
    fn from(_: QueryRejection) -> Self {
        AppError::invalid("query", "is not a valid query string")
    }
}

/// The only path parameter is a resource id; an unparsable id names no resource.
impl From<PathRejection> for AppError {
    fn from(_: PathRejection) -> Self {
        AppError::NotFound("Not found")
    }
}

/// Result type alias for the application
pub type Result<T, E = AppError> = std::result::Result<T, E>;
