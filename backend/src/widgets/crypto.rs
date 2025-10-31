use axum::{extract::{Query, State}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{auth::UserCtx, error::{AppError, Result}, AppState};

#[derive(Debug, Deserialize)]
pub struct CryptoQuery {
    #[serde(default = "default_symbols")]
    pub symbols: String, // Comma-separated, e.g., "BTC,ETH,SOL"
}

fn default_symbols() -> String {
    "BTC,ETH".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoPrice {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_24h: f64,
    pub change_percentage_24h: f64,
}

pub async fn fetch_crypto_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    Query(query): Query<CryptoQuery>,
) -> Result<impl IntoResponse> {
    let cache_key = format!("crypto:{}", query.symbols);
    
    // Check cache first
    if let Some(cached) = state.cache.get::<Vec<CryptoPrice>>(&cache_key).await.ok().flatten() {
        tracing::debug!("Cache hit for crypto data: {}", query.symbols);
        return Ok(Json(cached));
    }
    
    // For now, use CoinGecko API (free, no key required)
    // Alternative: CoinMarketCap if API key is configured
    let symbols_list: Vec<&str> = query.symbols.split(',').collect();
    let ids = symbols_list.join(",").to_lowercase();
    
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_24hr_change=true",
        ids
    );
    
    let response = reqwest::get(&url).await
        .map_err(|e| AppError::ExternalApi(format!("CoinGecko API error: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(AppError::ExternalApi(
            format!("CoinGecko API returned status: {}", response.status())
        ));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| AppError::ExternalApi(format!("Failed to parse crypto response: {}", e)))?;
    
    let mut prices = Vec::new();
    for symbol in &symbols_list {
        let id = symbol.to_lowercase();
        if let Some(data) = json[&id].as_object() {
            prices.push(CryptoPrice {
                symbol: symbol.to_uppercase().to_string(),
                name: symbol.to_string(),
                price: data["usd"].as_f64().unwrap_or(0.0),
                change_24h: data["usd_24h_change"].as_f64().unwrap_or(0.0),
                change_percentage_24h: data["usd_24h_change"].as_f64().unwrap_or(0.0),
            });
        }
    }
    
    // Cache for 5 minutes
    let _ = state.cache.set(&cache_key, &prices, 300).await;
    
    Ok(Json(prices))
}
