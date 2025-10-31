use axum::{extract::{Query, State}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{auth::UserCtx, error::{AppError, Result}, AppState};

#[derive(Debug, Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherData {
    pub temp: f64,
    pub feels_like: f64,
    pub humidity: i32,
    pub description: String,
    pub icon: String,
    pub city_name: String,
}

pub async fn fetch_weather_data(
    _user_ctx: UserCtx,
    State(state): State<AppState>,
    Query(query): Query<WeatherQuery>,
) -> Result<impl IntoResponse> {
    let api_key = state.config.openweather_api_key.as_ref()
        .ok_or_else(|| AppError::Internal("OpenWeather API key not configured".to_string()))?;
    
    let cache_key = format!("weather:{}", query.city);
    
    // Check cache first
    if let Some(cached) = state.cache.get::<WeatherData>(&cache_key).await.ok().flatten() {
        tracing::debug!("Cache hit for weather data: {}", query.city);
        return Ok(Json(cached));
    }
    
    // Fetch from OpenWeather API
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        query.city, api_key
    );
    
    let response = reqwest::get(&url).await
        .map_err(|e| AppError::ExternalApi(format!("OpenWeather API error: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(AppError::ExternalApi(
            format!("OpenWeather API returned status: {}", response.status())
        ));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| AppError::ExternalApi(format!("Failed to parse weather response: {}", e)))?;
    
    let weather_data = WeatherData {
        temp: json["main"]["temp"].as_f64().unwrap_or(0.0),
        feels_like: json["main"]["feels_like"].as_f64().unwrap_or(0.0),
        humidity: json["main"]["humidity"].as_i64().unwrap_or(0) as i32,
        description: json["weather"][0]["description"].as_str().unwrap_or("").to_string(),
        icon: json["weather"][0]["icon"].as_str().unwrap_or("").to_string(),
        city_name: json["name"].as_str().unwrap_or(&query.city).to_string(),
    };
    
    // Cache for 10 minutes
    let _ = state.cache.set(&cache_key, &weather_data, 600).await;
    
    Ok(Json(weather_data))
}
