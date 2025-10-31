use axum::{extract::{Query, State}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{auth::UserCtx, error::Result, AppState};

#[derive(Debug, Deserialize)]
pub struct GitHubQuery {
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub repo: GitHubRepo,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub name: String,
}

pub async fn fetch_github_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    Query(query): Query<GitHubQuery>,
) -> Result<impl IntoResponse> {
    let cache_key = format!("github:{}", query.username);
    
    // Check cache first
    if let Some(cached) = state.cache.get::<Vec<GitHubEvent>>(&cache_key).await.ok().flatten() {
        tracing::debug!("Cache hit for GitHub data: {}", query.username);
        return Ok(Json(cached));
    }
    
    // Fetch from GitHub API
    let client = reqwest::Client::new();
    let mut request = client
        .get(format!("https://api.github.com/users/{}/events/public", query.username))
        .header("User-Agent", "InsightBoard");
    
    if let Some(token) = &state.config.github_api_token {
        request = request.header("Authorization", format!("token {}", token));
    }
    
    let response = request.send().await
        .map_err(|e| crate::error::AppError::ExternalApi(format!("GitHub API error: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(crate::error::AppError::ExternalApi(
            format!("GitHub API returned status: {}", response.status())
        ));
    }
    
    let events: Vec<GitHubEvent> = response.json().await
        .map_err(|e| crate::error::AppError::ExternalApi(format!("Failed to parse GitHub response: {}", e)))?;
    
    // Cache for 5 minutes
    let _ = state.cache.set(&cache_key, &events, 300).await;
    
    Ok(Json(events))
}
