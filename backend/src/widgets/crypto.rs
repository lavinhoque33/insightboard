use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::UserCtx,
    error::{AppError, Result},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct CryptoQuery {
    #[serde(default = "default_ids")]
    pub ids: String, // Comma-separated CoinGecko IDs, e.g., "bitcoin,ethereum,solana"
}

fn default_ids() -> String {
    "bitcoin,ethereum".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoPrice {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: f64,
    pub price_change_percentage_24h: f64,
    pub market_cap: f64,
    pub total_volume: f64,
}

pub async fn fetch_crypto_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    Query(query): Query<CryptoQuery>,
) -> Result<impl IntoResponse> {
    let ids_list: Vec<String> = query
        .ids
        .split(',')
        .map(|id| id.trim().to_lowercase())
        .filter(|id| !id.is_empty())
        .collect();

    if ids_list.is_empty() {
        return Err(AppError::Validation("ids is required".to_string()));
    }

    let ids = ids_list.join(",");
    let cache_key = format!("crypto:{}", ids);

    // Check cache first
    if let Some(cached) = state
        .cache
        .get::<Vec<CryptoPrice>>(&cache_key)
        .await
        .ok()
        .flatten()
    {
        tracing::debug!("Cache hit for crypto data: {}", ids);
        return Ok(Json(cached));
    }

    // CoinGecko market endpoint returns stable fields needed by frontend contract.
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&ids={}&price_change_percentage=24h",
        ids
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| AppError::ExternalApi(format!("CoinGecko API error: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::ExternalApi(format!(
            "CoinGecko API returned status: {}",
            response.status()
        )));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::ExternalApi(format!("Failed to parse crypto response: {}", e)))?;

    let prices: Vec<CryptoPrice> = serde_json::from_value(json)
        .map_err(|e| AppError::ExternalApi(format!("Failed to decode crypto payload: {}", e)))?;

    // Cache for 5 minutes
    let _ = state.cache.set(&cache_key, &prices, 300).await;

    Ok(Json(prices))
}
