use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{
    auth::UserCtx,
    error::{AppError, Result},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub urls: String, // Comma-separated URLs
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCheck {
    pub url: String,
    pub status: u16,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

pub async fn fetch_status_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    Query(query): Query<StatusQuery>,
) -> Result<impl IntoResponse> {
    if query.urls.trim().is_empty() {
        return Err(AppError::Validation("urls is required".to_string()));
    }

    let cache_key = format!("status:{}", query.urls);

    // Check cache first
    if let Some(cached) = state
        .cache
        .get::<Vec<StatusCheck>>(&cache_key)
        .await
        .ok()
        .flatten()
    {
        tracing::debug!("Cache hit for status data");
        return Ok(Json(cached));
    }

    let urls: Vec<&str> = query.urls.split(',').collect();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let mut checks = Vec::new();

    for url in urls {
        let url = url.trim();
        if url.is_empty() {
            continue;
        }

        let start = std::time::Instant::now();

        match client.get(url).send().await {
            Ok(response) => {
                let elapsed = start.elapsed().as_millis() as u64;
                checks.push(StatusCheck {
                    url: url.to_string(),
                    status: response.status().as_u16(),
                    response_time_ms: elapsed,
                    error: None,
                });
            }
            Err(e) => {
                checks.push(StatusCheck {
                    url: url.to_string(),
                    status: 0,
                    response_time_ms: 0,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    // Cache for 2 minutes
    let _ = state.cache.set(&cache_key, &checks, 120).await;

    Ok(Json(checks))
}
