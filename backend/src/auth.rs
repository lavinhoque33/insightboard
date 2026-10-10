use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    AppState,
};

/// Token lifetime (owner, 2026-10-10; M4 decision 8).
pub const TOKEN_TTL: chrono::Duration = chrono::Duration::hours(24);

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub email: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

/// User context extracted from JWT
#[derive(Debug, Clone)]
pub struct UserCtx {
    pub user_id: Uuid,
}

/// Hash a password using Argon2 (CPU-heavy: call through [`hash_password_blocking`] from async code).
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("failed to hash password: {e}")))
}

/// Verify a password against a PHC hash string.
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Internal(format!("failed to parse password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// [`hash_password`] on the blocking pool.
pub async fn hash_password_blocking(password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(format!("hash task failed: {e}")))?
}

/// [`verify_password`] on the blocking pool.
pub async fn verify_password_blocking(password: String, password_hash: String) -> Result<bool> {
    tokio::task::spawn_blocking(move || verify_password(&password, &password_hash))
        .await
        .map_err(|e| AppError::Internal(format!("verify task failed: {e}")))?
}

/// A hash of a random password, verified against when a login names an unknown email so the
/// response takes as long as a wrong password (D17).
pub fn dummy_password_hash() -> anyhow::Result<String> {
    let random = SaltString::generate(&mut OsRng);
    hash_password(random.as_str()).map_err(|e| anyhow::anyhow!("{e}"))
}

/// Generate a JWT token valid for [`TOKEN_TTL`].
pub fn generate_token(user_id: Uuid, email: &str, secret: &str) -> Result<String> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: (now + TOKEN_TTL).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("failed to generate token: {e}")))
}

/// Validate a JWT token; every failure is the generic 401.
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| {
        tracing::debug!(kind = ?e.kind(), "rejected bearer token");
        AppError::Unauthorized
    })
}

/// Extract user context from request (auth middleware)
#[async_trait]
impl FromRequestParts<AppState> for UserCtx {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        if let Some(ctx) = parts.extensions.get::<UserCtx>() {
            return Ok(ctx.clone());
        }
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let claims = validate_token(bearer.token(), &state.config.jwt_secret)?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
        let ctx = UserCtx { user_id };
        parts.extensions.insert(ctx.clone());
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_token, hash_password, validate_token, verify_password, TOKEN_TTL};
    use crate::error::AppError;
    use uuid::Uuid;

    const SECRET: &str = "test-secret-that-is-at-least-32-bytes-long";

    #[test]
    fn password_hash_roundtrip() {
        let password = "password123";
        let hashed = hash_password(password).expect("password should hash");
        let valid = verify_password(password, &hashed).expect("verification should succeed");
        assert!(valid);
    }

    #[test]
    fn password_hash_rejects_wrong_password() {
        let hashed = hash_password("password123").expect("password should hash");
        let valid =
            verify_password("wrong-password", &hashed).expect("verification should succeed");
        assert!(!valid);
    }

    #[test]
    fn jwt_roundtrip_with_24_hour_lifetime() {
        let user_id = Uuid::new_v4();
        let email = "contract@example.com";

        let token =
            generate_token(user_id, email, SECRET).expect("token generation should succeed");
        let claims = validate_token(&token, SECRET).expect("token validation should succeed");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(
            (claims.exp - claims.iat) as i64,
            TOKEN_TTL.num_seconds(),
            "token lifetime"
        );
    }

    #[test]
    fn invalid_tokens_are_the_generic_unauthorized() {
        let token = generate_token(Uuid::new_v4(), "a@example.com", SECRET).unwrap();
        let other = "another-secret-that-is-at-least-32-bytes";
        assert!(matches!(
            validate_token(&token, other),
            Err(AppError::Unauthorized)
        ));
        assert!(matches!(
            validate_token("garbage", SECRET),
            Err(AppError::Unauthorized)
        ));
    }
}
