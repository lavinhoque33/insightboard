use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    error::FieldError,
    extract::{FromJson, ObjectReader},
};

/// Minimum password length in characters (unchanged since M0).
pub const MIN_PASSWORD_CHARS: usize = 8;

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

/// User registration request; `email` is normalised (trimmed, lower-cased).
#[derive(Debug)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// User login request; `email` is normalised (trimmed, lower-cased).
#[derive(Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl FromJson for RegisterRequest {
    fn from_json(value: Value) -> Result<Self, Vec<FieldError>> {
        let mut r = ObjectReader::new(&value);
        let email = read_email(&mut r);
        let password = r.required_string("password");
        if let Some(p) = &password {
            if p.chars().count() < MIN_PASSWORD_CHARS {
                r.push(
                    "password",
                    format!("must be at least {MIN_PASSWORD_CHARS} characters"),
                );
            }
        }
        match (email, password) {
            (Some(email), Some(password)) if r.errors.is_empty() => Ok(Self { email, password }),
            _ => Err(r.errors),
        }
    }
}

impl FromJson for LoginRequest {
    fn from_json(value: Value) -> Result<Self, Vec<FieldError>> {
        let mut r = ObjectReader::new(&value);
        let email = read_email(&mut r);
        let password = r.required_string("password");
        if password.as_deref() == Some("") {
            r.push("password", "is required");
        }
        match (email, password) {
            (Some(email), Some(password)) if r.errors.is_empty() => Ok(Self { email, password }),
            _ => Err(r.errors),
        }
    }
}

fn read_email(r: &mut ObjectReader<'_>) -> Option<String> {
    let raw = r.required_string("email")?;
    match normalize_email(&raw) {
        Ok(email) => Some(email),
        Err(message) => {
            r.push("email", message);
            None
        }
    }
}

/// Trim, lower-case, and validate an email address (a pragmatic subset of RFC 5321: one `@`,
/// a dot-free-of-specials local part, and a dotted DNS domain).
pub fn normalize_email(raw: &str) -> Result<String, &'static str> {
    let email = raw.trim().to_lowercase();
    if email.is_empty() {
        return Err("is required");
    }
    const INVALID: &str = "must be a valid email address";
    if email.len() > 254 {
        return Err(INVALID);
    }
    let (local, domain) = email.split_once('@').ok_or(INVALID)?;
    let local_ok = !local.is_empty()
        && local.len() <= 64
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "!#$%&'*+-/=?^_`{|}~.".contains(c));
    let labels: Vec<&str> = domain.split('.').collect();
    let domain_ok = labels.len() >= 2
        && labels.iter().all(|l| {
            !l.is_empty()
                && l.len() <= 63
                && !l.starts_with('-')
                && !l.ends_with('-')
                && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        });
    if local_ok && domain_ok {
        Ok(email)
    } else {
        Err(INVALID)
    }
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

#[cfg(test)]
mod tests {
    use super::{normalize_email, LoginRequest, RegisterRequest};
    use crate::extract::FromJson;
    use serde_json::json;

    #[test]
    fn email_is_trimmed_and_lower_cased() {
        assert_eq!(
            normalize_email("  Alice.Smith+tag@Example.COM ").unwrap(),
            "alice.smith+tag@example.com"
        );
    }

    #[test]
    fn invalid_emails_are_rejected() {
        for bad in [
            "",
            "   ",
            "plain",
            "a@b",
            "@example.com",
            "a@@example.com",
            "a b@example.com",
            "a@exa mple.com",
            "a@-example.com",
            "a@example..com",
            ".a@example.com",
            "a\"b@example.com",
        ] {
            assert!(normalize_email(bad).is_err(), "{bad:?} should be invalid");
        }
    }

    #[test]
    fn register_reports_every_invalid_field() {
        let errors =
            RegisterRequest::from_json(json!({"email": "nope", "password": "short"})).unwrap_err();
        let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
        assert_eq!(fields, ["email", "password"]);

        let errors = RegisterRequest::from_json(json!({"email": 5})).unwrap_err();
        let fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
        assert_eq!(fields, ["email", "password"]);

        let errors = RegisterRequest::from_json(json!(["x"])).unwrap_err();
        assert_eq!(errors[0].field, "body");
    }

    #[test]
    fn login_requires_a_password_and_normalises_email() {
        let ok =
            LoginRequest::from_json(json!({"email": " A@Example.com", "password": "x"})).unwrap();
        assert_eq!(ok.email, "a@example.com");
        let errors =
            LoginRequest::from_json(json!({"email": "a@example.com", "password": ""})).unwrap_err();
        assert_eq!(errors[0].field, "password");
    }
}
