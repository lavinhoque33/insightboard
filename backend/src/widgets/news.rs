use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use super::{
    http::{endpoint, fetch_json},
    normalize_text,
};
use crate::{
    auth::UserCtx,
    error::{AppError, Result},
    extract::ApiQuery,
    AppState,
};

const DEFAULT_TOPIC: &str = "technology";
const MAX_TOPIC_CHARS: usize = 100;
const CACHE_TTL_SECS: u64 = 900;

#[derive(Debug, Deserialize)]
pub struct NewsQuery {
    pub topic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub description: String,
    pub url: String,
    pub source: NewsSource,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "urlToImage")]
    pub url_to_image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsSource {
    pub name: String,
}

/// Map one upstream article; articles without a URL or publication time are dropped (the
/// contract requires both).
fn map_article(article: &serde_json::Value) -> Option<NewsArticle> {
    let url = article["url"].as_str().filter(|s| !s.is_empty())?;
    let published_at = article["publishedAt"].as_str().filter(|s| !s.is_empty())?;
    Some(NewsArticle {
        title: article["title"].as_str().unwrap_or("").to_string(),
        description: article["description"].as_str().unwrap_or("").to_string(),
        url: url.to_string(),
        source: NewsSource {
            name: article["source"]["name"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or("Unknown")
                .to_string(),
        },
        published_at: published_at.to_string(),
        url_to_image: article["urlToImage"].as_str().map(str::to_string),
    })
}

pub async fn fetch_news_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<NewsQuery>,
) -> Result<Json<Vec<NewsArticle>>> {
    let topic = match query.topic.as_deref() {
        None => DEFAULT_TOPIC.to_string(),
        Some(raw) => normalize_text(raw),
    };
    if topic.is_empty() {
        return Err(AppError::invalid("topic", "must not be blank"));
    }
    if topic.chars().count() > MAX_TOPIC_CHARS {
        return Err(AppError::invalid(
            "topic",
            format!("must be at most {MAX_TOPIC_CHARS} characters"),
        ));
    }
    let api_key = state
        .config
        .newsapi_api_key
        .as_deref()
        .ok_or(AppError::ProviderNotConfigured("NEWSAPI_API_KEY"))?;

    let cache_key = state.cache.key("news", &[&topic.to_lowercase()]);
    if let Some(cached) = state.cache.get::<Vec<NewsArticle>>(&cache_key).await {
        return Ok(Json(cached));
    }

    let mut url = endpoint(&state.config.provider_urls.newsapi, &["v2", "everything"])?;
    url.query_pairs_mut()
        .append_pair("q", &topic)
        .append_pair("pageSize", "10")
        .append_pair("sortBy", "publishedAt");
    // The key travels in a header, never in the URL.
    let request = state.http.provider.get(url).header("X-Api-Key", api_key);
    let json: serde_json::Value = fetch_json("NewsAPI", request).await?;

    let articles: Vec<NewsArticle> = json["articles"]
        .as_array()
        .map(|items| items.iter().filter_map(map_article).collect())
        .unwrap_or_default();

    state.cache.set(&cache_key, &articles, CACHE_TTL_SECS).await;
    Ok(Json(articles))
}
