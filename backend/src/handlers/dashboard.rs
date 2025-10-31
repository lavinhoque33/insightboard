use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{
    auth::UserCtx,
    error::{AppError, Result},
    models::{CreateDashboardRequest, Dashboard, DashboardResponse, UpdateDashboardRequest},
    AppState,
};

/// List all dashboards for the authenticated user
pub async fn list_dashboards(
    user_ctx: UserCtx,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let dashboards: Vec<Dashboard> = sqlx::query_as(
        "SELECT id, user_id, name, layout_json, settings_json, created_at, updated_at 
         FROM dashboards 
         WHERE user_id = $1 
         ORDER BY updated_at DESC"
    )
    .bind(user_ctx.user_id)
    .fetch_all(state.db.pool())
    .await?;

    let response: Vec<DashboardResponse> = dashboards
        .into_iter()
        .map(DashboardResponse::from)
        .collect();

    Ok(Json(response))
}

/// Get a specific dashboard
pub async fn get_dashboard(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let dashboard: Option<Dashboard> = sqlx::query_as(
        "SELECT id, user_id, name, layout_json, settings_json, created_at, updated_at 
         FROM dashboards 
         WHERE id = $1 AND user_id = $2"
    )
    .bind(dashboard_id)
    .bind(user_ctx.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    let dashboard = dashboard.ok_or_else(|| AppError::NotFound("Dashboard not found".to_string()))?;

    Ok(Json(DashboardResponse::from(dashboard)))
}

/// Create a new dashboard
pub async fn create_dashboard(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    Json(payload): Json<CreateDashboardRequest>,
) -> Result<impl IntoResponse> {
    // Validate input
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Dashboard name is required".to_string()));
    }

    let dashboard: Dashboard = sqlx::query_as(
        "INSERT INTO dashboards (user_id, name, layout_json, settings_json) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, user_id, name, layout_json, settings_json, created_at, updated_at"
    )
    .bind(user_ctx.user_id)
    .bind(payload.name.trim())
    .bind(payload.layout_json)
    .bind(payload.settings_json)
    .fetch_one(state.db.pool())
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(DashboardResponse::from(dashboard)),
    ))
}

/// Update an existing dashboard
pub async fn update_dashboard(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<UpdateDashboardRequest>,
) -> Result<impl IntoResponse> {
    // Check if dashboard exists and belongs to user
    let existing: Option<Dashboard> = sqlx::query_as(
        "SELECT id, user_id, name, layout_json, settings_json, created_at, updated_at 
         FROM dashboards 
         WHERE id = $1 AND user_id = $2"
    )
    .bind(dashboard_id)
    .bind(user_ctx.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    let existing = existing.ok_or_else(|| AppError::NotFound("Dashboard not found".to_string()))?;

    // Build update query dynamically
    let name = payload.name.unwrap_or(existing.name);
    let layout_json = payload.layout_json.unwrap_or(existing.layout_json);
    let settings_json = payload.settings_json.unwrap_or(existing.settings_json);

    let dashboard: Dashboard = sqlx::query_as(
        "UPDATE dashboards 
         SET name = $1, layout_json = $2, settings_json = $3, updated_at = NOW() 
         WHERE id = $4 
         RETURNING id, user_id, name, layout_json, settings_json, created_at, updated_at"
    )
    .bind(name)
    .bind(layout_json)
    .bind(settings_json)
    .bind(dashboard_id)
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(DashboardResponse::from(dashboard)))
}

/// Delete a dashboard
pub async fn delete_dashboard(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let result = sqlx::query(
        "DELETE FROM dashboards WHERE id = $1 AND user_id = $2"
    )
    .bind(dashboard_id)
    .bind(user_ctx.user_id)
    .execute(state.db.pool())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Dashboard not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
