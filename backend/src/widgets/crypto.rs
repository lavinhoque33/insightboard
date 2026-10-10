use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::http::{endpoint, fetch_json};
use crate::{
    auth::UserCtx,
    error::{AppError, Result},
    extract::ApiQuery,
    AppState,
};

const DEFAULT_IDS: &str = "bitcoin,ethereum";
const MAX_IDS: usize = 50;
const MAX_ID_CHARS: usize = 100;
const CACHE_TTL_SECS: u64 = 300;

#[derive(Debug, Deserialize)]
pub struct CryptoQuery {
    /// Comma-separated CoinGecko IDs, e.g. "bitcoin,ethereum,solana".
    pub ids: Option<String>,
}

/// A contract item; a numeric field that is null or missing upstream stays `null` (D20).
#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoPrice {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub market_cap: Option<f64>,
    pub total_volume: Option<f64>,
}

/// The upstream item, tolerant of nulls and missing fields.
#[derive(Debug, Deserialize)]
struct UpstreamCoin {
    id: Option<String>,
    symbol: Option<String>,
    name: Option<String>,
    current_price: Option<f64>,
    price_change_percentage_24h: Option<f64>,
    market_cap: Option<f64>,
    total_volume: Option<f64>,
}

impl UpstreamCoin {
    /// Items without their identifying strings are dropped; numbers pass through as given.
    fn into_price(self) -> Option<CryptoPrice> {
        let non_empty = |s: Option<String>| s.filter(|s| !s.is_empty());
        Some(CryptoPrice {
            id: non_empty(self.id)?,
            symbol: non_empty(self.symbol)?,
            name: non_empty(self.name)?,
            current_price: self.current_price,
            price_change_percentage_24h: self.price_change_percentage_24h,
            market_cap: self.market_cap,
            total_volume: self.total_volume,
        })
    }
}

/// Normalise `ids`: trimmed, lower-cased, de-duplicated, sorted (one cache entry per set).
fn parse_ids(raw: &str) -> Result<Vec<String>> {
    let ids: BTreeSet<String> = raw
        .split(',')
        .map(|id| id.trim().to_lowercase())
        .filter(|id| !id.is_empty())
        .collect();
    if ids.is_empty() {
        return Err(AppError::invalid("ids", "is required"));
    }
    if ids.len() > MAX_IDS {
        return Err(AppError::invalid(
            "ids",
            format!("must list at most {MAX_IDS} ids"),
        ));
    }
    if ids.iter().any(|id| id.chars().count() > MAX_ID_CHARS) {
        return Err(AppError::invalid(
            "ids",
            format!("each id must be at most {MAX_ID_CHARS} characters"),
        ));
    }
    Ok(ids.into_iter().collect())
}

pub async fn fetch_crypto_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<CryptoQuery>,
) -> Result<Json<Vec<CryptoPrice>>> {
    let ids = parse_ids(query.ids.as_deref().unwrap_or(DEFAULT_IDS))?.join(",");

    let cache_key = state.cache.key("crypto", &[&ids]);
    if let Some(cached) = state.cache.get::<Vec<CryptoPrice>>(&cache_key).await {
        return Ok(Json(cached));
    }

    let mut url = endpoint(
        &state.config.provider_urls.coingecko,
        &["api", "v3", "coins", "markets"],
    )?;
    url.query_pairs_mut()
        .append_pair("vs_currency", "usd")
        .append_pair("ids", &ids)
        .append_pair("price_change_percentage", "24h");
    let coins: Vec<UpstreamCoin> = fetch_json("CoinGecko", state.http.provider.get(url)).await?;
    let prices: Vec<CryptoPrice> = coins
        .into_iter()
        .filter_map(UpstreamCoin::into_price)
        .collect();

    state.cache.set(&cache_key, &prices, CACHE_TTL_SECS).await;
    Ok(Json(prices))
}

#[cfg(test)]
mod tests {
    use super::parse_ids;

    #[test]
    fn ids_are_normalised_deduplicated_and_sorted() {
        assert_eq!(
            parse_ids(" Ethereum, bitcoin ,,BITCOIN").unwrap(),
            ["bitcoin", "ethereum"]
        );
        assert!(parse_ids(" , ").is_err());
        let many: Vec<String> = (0..51).map(|i| format!("c{i}")).collect();
        assert!(parse_ids(&many.join(",")).is_err());
    }
}
