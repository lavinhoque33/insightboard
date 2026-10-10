use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::http::{endpoint, fetch_json};
use crate::{
    auth::UserCtx,
    error::{AppError, Result},
    extract::ApiQuery,
    AppState,
};

/// GitHub's own limit on username length.
const MAX_USERNAME_CHARS: usize = 39;
const CACHE_TTL_SECS: u64 = 300;

#[derive(Debug, Deserialize)]
pub struct GitHubQuery {
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub repo: GitHubRepo,
    pub created_at: String,
    pub payload: Option<JsonValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub name: String,
}

/// GitHub's username rules: ASCII letters, digits, and single hyphens, not starting or ending
/// with a hyphen, at most 39 characters. Anything else (`/`, `..`, `?`) never reaches GitHub.
pub fn is_valid_username(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_USERNAME_CHARS
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

pub async fn fetch_github_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<GitHubQuery>,
) -> Result<Json<Vec<GitHubEvent>>> {
    let username = query.username.unwrap_or_default().trim().to_string();
    if username.is_empty() {
        return Err(AppError::invalid("username", "is required"));
    }
    if !is_valid_username(&username) {
        return Err(AppError::invalid(
            "username",
            "must be a GitHub username: letters, digits, and single hyphens, at most 39 characters",
        ));
    }

    // Usernames are case-insensitive; one cache entry per account.
    let normalized = username.to_ascii_lowercase();
    let cache_key = state.cache.key("github", &[&normalized]);
    if let Some(cached) = state.cache.get::<Vec<GitHubEvent>>(&cache_key).await {
        return Ok(Json(cached));
    }

    let url = endpoint(
        &state.config.provider_urls.github,
        &["users", &normalized, "events", "public"],
    )?;
    let mut request = state
        .http
        .provider
        .get(url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json");
    if let Some(token) = &state.config.github_api_token {
        request = request.bearer_auth(token);
    }
    let events: Vec<GitHubEvent> = fetch_json("GitHub", request).await?;

    state.cache.set(&cache_key, &events, CACHE_TTL_SECS).await;
    Ok(Json(events))
}

#[cfg(test)]
mod tests {
    use super::is_valid_username;

    #[test]
    fn username_rules_match_github() {
        for ok in ["octocat", "a", "a-b-c", "A1", &"x".repeat(39)] {
            assert!(is_valid_username(ok), "{ok}");
        }
        for bad in [
            "",
            "-a",
            "a-",
            "a--b",
            "a/b",
            "..",
            "a/../b",
            "a.b",
            "a b",
            "a?x=1",
            "ü",
            &"x".repeat(40),
        ] {
            assert!(!is_valid_username(bad), "{bad}");
        }
    }
}
