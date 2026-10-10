use axum::{extract::State, http::StatusCode, Json};
use uuid::Uuid;

use crate::{
    auth::UserCtx,
    error::{AppError, Result},
    extract::{ApiPath, ValidJson},
    models::{CreateDashboardRequest, Dashboard, DashboardResponse, UpdateDashboardRequest},
    AppState,
};

/// Per-user dashboard cap (D29); creating beyond it is 409.
pub const MAX_DASHBOARDS_PER_USER: i64 = 100;

const NOT_FOUND: &str = "Dashboard not found";
const COLUMNS: &str = "id, user_id, name, layout_json, settings_json, created_at, updated_at";

/// List all dashboards for the authenticated user (at most [`MAX_DASHBOARDS_PER_USER`]).
pub async fn list_dashboards(
    user_ctx: UserCtx,
    State(state): State<AppState>,
) -> Result<Json<Vec<DashboardResponse>>> {
    let dashboards: Vec<Dashboard> = sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM dashboards WHERE user_id = $1
         ORDER BY updated_at DESC, id LIMIT $2"
    ))
    .bind(user_ctx.user_id)
    .bind(MAX_DASHBOARDS_PER_USER)
    .fetch_all(state.db.pool())
    .await?;

    Ok(Json(
        dashboards
            .into_iter()
            .map(DashboardResponse::from)
            .collect(),
    ))
}

/// Get a specific dashboard
pub async fn get_dashboard(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    ApiPath(dashboard_id): ApiPath<Uuid>,
) -> Result<Json<DashboardResponse>> {
    let dashboard: Option<Dashboard> = sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM dashboards WHERE id = $1 AND user_id = $2"
    ))
    .bind(dashboard_id)
    .bind(user_ctx.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    dashboard
        .map(|d| Json(DashboardResponse::from(d)))
        .ok_or(AppError::NotFound(NOT_FOUND))
}

/// Create a new dashboard. The user row is locked for the transaction so concurrent creates
/// cannot pass the cap together.
pub async fn create_dashboard(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    ValidJson(payload): ValidJson<CreateDashboardRequest>,
) -> Result<(StatusCode, Json<DashboardResponse>)> {
    let mut tx = state.db.pool().begin().await?;

    let user: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1 FOR UPDATE")
        .bind(user_ctx.user_id)
        .fetch_optional(&mut *tx)
        .await?;
    if user.is_none() {
        return Err(AppError::Unauthorized);
    }
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM dashboards WHERE user_id = $1")
        .bind(user_ctx.user_id)
        .fetch_one(&mut *tx)
        .await?;
    if count >= MAX_DASHBOARDS_PER_USER {
        return Err(AppError::Conflict("Dashboard limit reached"));
    }

    let dashboard: Dashboard = sqlx::query_as(&format!(
        "INSERT INTO dashboards (user_id, name, layout_json, settings_json)
         VALUES ($1, $2, $3, $4) RETURNING {COLUMNS}"
    ))
    .bind(user_ctx.user_id)
    .bind(&payload.name)
    .bind(&payload.layout_json)
    .bind(&payload.settings_json)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(DashboardResponse::from(dashboard)),
    ))
}

/// Update an existing dashboard; absent fields keep their stored value.
pub async fn update_dashboard(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    ApiPath(dashboard_id): ApiPath<Uuid>,
    ValidJson(payload): ValidJson<UpdateDashboardRequest>,
) -> Result<Json<DashboardResponse>> {
    let dashboard: Option<Dashboard> = sqlx::query_as(&format!(
        "UPDATE dashboards
         SET name = COALESCE($1, name),
             layout_json = COALESCE($2, layout_json),
             settings_json = COALESCE($3, settings_json),
             updated_at = NOW()
         WHERE id = $4 AND user_id = $5
         RETURNING {COLUMNS}"
    ))
    .bind(payload.name)
    .bind(payload.layout_json)
    .bind(payload.settings_json)
    .bind(dashboard_id)
    .bind(user_ctx.user_id)
    .fetch_optional(state.db.pool())
    .await?;

    dashboard
        .map(|d| Json(DashboardResponse::from(d)))
        .ok_or(AppError::NotFound(NOT_FOUND))
}

/// Delete a dashboard
pub async fn delete_dashboard(
    user_ctx: UserCtx,
    State(state): State<AppState>,
    ApiPath(dashboard_id): ApiPath<Uuid>,
) -> Result<StatusCode> {
    let result = sqlx::query("DELETE FROM dashboards WHERE id = $1 AND user_id = $2")
        .bind(dashboard_id)
        .bind(user_ctx.user_id)
        .execute(state.db.pool())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(NOT_FOUND));
    }

    Ok(StatusCode::NO_CONTENT)
}
