use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "insightboard-backend",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}
