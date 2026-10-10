use axum::{extract::State, http::StatusCode, Json};

use crate::{
    auth::{generate_token, hash_password_blocking, verify_password_blocking, UserCtx},
    error::{AppError, Result},
    extract::ValidJson,
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse},
    AppState,
};

const DUPLICATE_EMAIL: &str = "Email already registered";

/// Register a new user. A duplicate email is 409, including two concurrent registrations: the
/// insert itself decides (`ON CONFLICT DO NOTHING`), not an earlier existence check.
pub async fn register(
    State(state): State<AppState>,
    ValidJson(payload): ValidJson<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>)> {
    let password_hash = hash_password_blocking(payload.password).await?;

    let user: Option<User> = sqlx::query_as(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2)
         ON CONFLICT (email) DO NOTHING
         RETURNING id, email, password_hash, created_at",
    )
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_optional(state.db.pool())
    .await?;
    let user = user.ok_or(AppError::Conflict(DUPLICATE_EMAIL))?;

    let token = generate_token(user.id, &user.email, &state.config.jwt_secret)?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user.into(),
        }),
    ))
}

/// Login a user. An unknown email still runs one Argon2 verification (against a dummy hash) so
/// it is not distinguishable by timing from a wrong password.
pub async fn login(
    State(state): State<AppState>,
    ValidJson(payload): ValidJson<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let user: Option<User> =
        sqlx::query_as("SELECT id, email, password_hash, created_at FROM users WHERE email = $1")
            .bind(&payload.email)
            .fetch_optional(state.db.pool())
            .await?;

    let hash = user
        .as_ref()
        .map_or_else(|| state.dummy_hash.to_string(), |u| u.password_hash.clone());
    let is_valid = verify_password_blocking(payload.password, hash).await?;
    let user = match user {
        Some(user) if is_valid => user,
        _ => return Err(AppError::Unauthorized),
    };

    let token = generate_token(user.id, &user.email, &state.config.jwt_secret)?;
    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// Get current authenticated user; a token for a deleted user is 401.
pub async fn me(user_ctx: UserCtx, State(state): State<AppState>) -> Result<Json<UserResponse>> {
    let user: Option<User> =
        sqlx::query_as("SELECT id, email, password_hash, created_at FROM users WHERE id = $1")
            .bind(user_ctx.user_id)
            .fetch_optional(state.db.pool())
            .await?;
    user.map(|u| Json(UserResponse::from(u)))
        .ok_or(AppError::Unauthorized)
}
