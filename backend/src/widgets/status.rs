use axum::{extract::{Query, State}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{auth::UserCtx, error::Result, AppState};

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub urls: String, // Comma-separated URLs
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCheck {
    pub url: String,
    pub status: String,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
}

pub async fn fetch_status_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    Query(query): Query<StatusQuery>,
) -> Result<impl IntoResponse> {
    let cache_key = format!("status:{}", query.urls);
    
    // Check cache first
    if let Some(cached) = state.cache.get::<Vec<StatusCheck>>(&cache_key).await.ok().flatten() {
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
                    status: "up".to_string(),
                    status_code: Some(response.status().as_u16()),
                    response_time_ms: Some(elapsed),
                });
            }
            Err(e) => {
                checks.push(StatusCheck {
                    url: url.to_string(),
                    status: format!("down: {}", e),
                    status_code: None,
                    response_time_ms: None,
                });
            }
        }
    }
    
    // Cache for 2 minutes
    let _ = state.cache.set(&cache_key, &checks, 120).await;
    
    Ok(Json(checks))
}
