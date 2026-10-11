use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::{
    error::{AppError, Result},
    AppState,
};

/// Liveness: no dependency is checked.
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "insightboard-backend",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Readiness: 200 when Postgres and Redis both answer, otherwise 503 `{error}` (D19). With the
/// cache disabled (Lambda without `REDIS_URL`, M6 decision 9) only Postgres is checked and the
/// body says `"cache": "disabled"`.
pub async fn readiness_check(State(state): State<AppState>) -> Result<Json<Value>> {
    if !state.cache.is_enabled() {
        return match state.db.ping().await {
            Ok(()) => Ok(Json(json!({ "status": "ready", "cache": "disabled" }))),
            Err(e) => Err(AppError::NotReady(format!("postgres: {e}"))),
        };
    }
    let (db, cache) = tokio::join!(state.db.ping(), state.cache.ping());
    let mut down = Vec::new();
    if let Err(e) = db {
        down.push(format!("postgres: {e}"));
    }
    if let Err(e) = cache {
        down.push(format!("redis: {e}"));
    }
    if down.is_empty() {
        Ok(Json(json!({ "status": "ready" })))
    } else {
        Err(AppError::NotReady(down.join("; ")))
    }
}
