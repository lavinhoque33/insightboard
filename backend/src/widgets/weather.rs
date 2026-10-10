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

const MAX_CITY_CHARS: usize = 100;
const CACHE_TTL_SECS: u64 = 600;

#[derive(Debug, Deserialize)]
pub struct WeatherQuery {
    pub city: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherData {
    pub city: String,
    pub country: String,
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: i32,
    pub description: String,
    pub wind_speed: f64,
    pub icon: String,
}

pub async fn fetch_weather_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<WeatherQuery>,
) -> Result<Json<WeatherData>> {
    let city = normalize_text(query.city.as_deref().unwrap_or(""));
    if city.is_empty() {
        return Err(AppError::invalid("city", "is required"));
    }
    if city.chars().count() > MAX_CITY_CHARS {
        return Err(AppError::invalid(
            "city",
            format!("must be at most {MAX_CITY_CHARS} characters"),
        ));
    }
    // OpenWeather has no header option: the key stays in the query string, so the request URL
    // is never logged (errors are stripped of it in `fetch_json`).
    let api_key = state
        .config
        .openweather_api_key
        .as_deref()
        .ok_or(AppError::ProviderNotConfigured("OPENWEATHER_API_KEY"))?;

    let cache_key = state.cache.key("weather", &[&city.to_lowercase()]);
    if let Some(cached) = state.cache.get::<WeatherData>(&cache_key).await {
        return Ok(Json(cached));
    }

    let mut url = endpoint(
        &state.config.provider_urls.openweather,
        &["data", "2.5", "weather"],
    )?;
    url.query_pairs_mut()
        .append_pair("q", &city)
        .append_pair("appid", api_key)
        .append_pair("units", "metric");
    let json: serde_json::Value = fetch_json("OpenWeather", state.http.provider.get(url)).await?;

    let weather_data = WeatherData {
        city: json["name"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or(&city)
            .to_string(),
        country: json["sys"]["country"]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or("N/A")
            .to_string(),
        temperature: json["main"]["temp"].as_f64().unwrap_or(0.0),
        feels_like: json["main"]["feels_like"].as_f64().unwrap_or(0.0),
        humidity: json["main"]["humidity"]
            .as_i64()
            .and_then(|h| i32::try_from(h).ok())
            .unwrap_or(0),
        description: json["weather"][0]["description"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        wind_speed: json["wind"]["speed"].as_f64().unwrap_or(0.0),
        icon: json["weather"][0]["icon"]
            .as_str()
            .unwrap_or("")
            .to_string(),
    };

    state
        .cache
        .set(&cache_key, &weather_data, CACHE_TTL_SECS)
        .await;
    Ok(Json(weather_data))
}
