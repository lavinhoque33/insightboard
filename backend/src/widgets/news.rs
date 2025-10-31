use axum::{extract::{Query, State}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{auth::UserCtx, error::{AppError, Result}, AppState};

#[derive(Debug, Deserialize)]
pub struct NewsQuery {
    #[serde(default = "default_topic")]
    pub topic: String,
}

fn default_topic() -> String {
    "technology".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub source: String,
    pub published_at: String,
    pub url_to_image: Option<String>,
}

pub async fn fetch_news_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    Query(query): Query<NewsQuery>,
) -> Result<impl IntoResponse> {
    let api_key = state.config.newsapi_api_key.as_ref()
        .ok_or_else(|| AppError::Internal("NewsAPI key not configured".to_string()))?;
    
    let cache_key = format!("news:{}", query.topic);
    
    // Check cache first
    if let Some(cached) = state.cache.get::<Vec<NewsArticle>>(&cache_key).await.ok().flatten() {
        tracing::debug!("Cache hit for news data: {}", query.topic);
        return Ok(Json(cached));
    }
    
    // Fetch from NewsAPI
    let url = format!(
        "https://newsapi.org/v2/everything?q={}&apiKey={}&pageSize=10&sortBy=publishedAt",
        query.topic, api_key
    );
    
    let response = reqwest::get(&url).await
        .map_err(|e| AppError::ExternalApi(format!("NewsAPI error: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(AppError::ExternalApi(
            format!("NewsAPI returned status: {}", response.status())
        ));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| AppError::ExternalApi(format!("Failed to parse news response: {}", e)))?;
    
    let articles: Vec<NewsArticle> = json["articles"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|article| NewsArticle {
            title: article["title"].as_str().unwrap_or("").to_string(),
            description: article["description"].as_str().map(|s| s.to_string()),
            url: article["url"].as_str().unwrap_or("").to_string(),
            source: article["source"]["name"].as_str().unwrap_or("Unknown").to_string(),
            published_at: article["publishedAt"].as_str().unwrap_or("").to_string(),
            url_to_image: article["urlToImage"].as_str().map(|s| s.to_string()),
        })
        .collect();
    
    // Cache for 15 minutes
    let _ = state.cache.set(&cache_key, &articles, 900).await;
    
    Ok(Json(articles))
}
