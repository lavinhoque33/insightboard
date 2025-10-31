use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

/// User registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// User login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Authentication response with JWT token
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

/// User response (without password)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
        }
    }
}
