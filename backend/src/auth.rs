use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::{AppError, Result}, AppState};

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub email: String,
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
}

/// User context extracted from JWT
#[derive(Debug, Clone)]
pub struct UserCtx {
    pub user_id: Uuid,
    pub email: String,
}

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();
    
    Ok(password_hash)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Internal(format!("Failed to parse password hash: {}", e)))?;
    
    let argon2 = Argon2::default();
    
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

/// Generate a JWT token
pub fn generate_token(user_id: Uuid, email: &str, secret: &str) -> Result<String> {
    let now = chrono::Utc::now();
    let exp = (now + chrono::Duration::days(7)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Auth(format!("Failed to generate token: {}", e)))?;

    Ok(token)
}

/// Validate a JWT token
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

/// Extract user context from request (auth middleware)
#[async_trait]
impl FromRequestParts<AppState> for UserCtx {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        // Extract the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        // Validate the token
        let claims = validate_token(bearer.token(), &state.config.jwt_secret)?;

        // Parse user ID
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Auth("Invalid user ID in token".to_string()))?;

        Ok(UserCtx {
            user_id,
            email: claims.email,
        })
    }
}
