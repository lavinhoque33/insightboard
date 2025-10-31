use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{
    auth::{generate_token, hash_password, verify_password, UserCtx},
    error::{AppError, Result},
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse},
    AppState,
};

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse> {
    // Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::Validation("Email and password are required".to_string()));
    }

    if payload.password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters".to_string()));
    }

    // Check if user already exists
    let existing_user: Option<User> = sqlx::query_as(
        "SELECT id, email, password_hash, created_at FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(state.db.pool())
    .await?;

    if existing_user.is_some() {
        return Err(AppError::Validation("Email already registered".to_string()));
    }

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Insert user
    let user: User = sqlx::query_as(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id, email, password_hash, created_at"
    )
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(state.db.pool())
    .await?;

    // Generate token
    let token = generate_token(user.id, &user.email, &state.config.jwt_secret)?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user.into(),
        }),
    ))
}

/// Login a user
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse> {
    // Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(AppError::Validation("Email and password are required".to_string()));
    }

    // Find user
    let user: Option<User> = sqlx::query_as(
        "SELECT id, email, password_hash, created_at FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(state.db.pool())
    .await?;

    let user = user.ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

    // Verify password
    let is_valid = verify_password(&payload.password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::Auth("Invalid credentials".to_string()));
    }

    // Generate token
    let token = generate_token(user.id, &user.email, &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// Get current authenticated user
pub async fn me(
    user_ctx: UserCtx,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let user: User = sqlx::query_as(
        "SELECT id, email, password_hash, created_at FROM users WHERE id = $1"
    )
    .bind(user_ctx.user_id)
    .fetch_one(state.db.pool())
    .await?;

    Ok(Json(UserResponse::from(user)))
}
