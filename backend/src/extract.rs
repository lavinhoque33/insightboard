//! Request extractors whose rejections render as the JSON error model (D16).

use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Path, Query, Request},
    Json,
};
use serde_json::Value;

use crate::error::{AppError, FieldError};

/// A request body type built from parsed JSON, reporting every invalid field.
pub trait FromJson: Sized {
    fn from_json(value: Value) -> Result<Self, Vec<FieldError>>;
}

/// JSON body extractor: 415 for a wrong content type, 400 for malformed JSON, 422 with field
/// details for well-formed JSON that does not describe a valid `T`.
pub struct ValidJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: FromJson,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<Value>::from_request(req, state).await?;
        T::from_json(value)
            .map(ValidJson)
            .map_err(AppError::Validation)
    }
}

/// `Query` with a JSON rejection.
#[derive(FromRequestParts)]
#[from_request(via(Query), rejection(AppError))]
pub struct ApiQuery<T>(pub T);

/// `Path` with a JSON rejection.
#[derive(FromRequestParts)]
#[from_request(via(Path), rejection(AppError))]
pub struct ApiPath<T>(pub T);

/// Field-by-field reader for a JSON object body.
pub struct ObjectReader<'a> {
    object: Option<&'a serde_json::Map<String, Value>>,
    pub errors: Vec<FieldError>,
}

impl<'a> ObjectReader<'a> {
    /// A non-object body records a `body` error; every later read is then skipped.
    pub fn new(value: &'a Value) -> Self {
        match value.as_object() {
            Some(object) => Self {
                object: Some(object),
                errors: Vec::new(),
            },
            None => Self {
                object: None,
                errors: vec![FieldError::new("body", "must be a JSON object")],
            },
        }
    }

    /// The raw value of a present field; an explicit `null` counts as absent.
    pub fn get(&self, field: &str) -> Option<&'a Value> {
        self.object
            .and_then(|o| o.get(field))
            .filter(|v| !v.is_null())
    }

    /// A required string field; records "is required" or "must be a string".
    pub fn required_string(&mut self, field: &str) -> Option<String> {
        self.object?;
        match self.get(field) {
            None => {
                self.errors.push(FieldError::new(field, "is required"));
                None
            }
            Some(value) => self.string_value(field, value),
        }
    }

    /// An optional string field (absent or `null` is `None`); a non-string records an error.
    pub fn optional_string(&mut self, field: &str) -> Option<String> {
        let value = self.get(field)?;
        self.string_value(field, value)
    }

    fn string_value(&mut self, field: &str, value: &Value) -> Option<String> {
        match value.as_str() {
            Some(s) => Some(s.to_string()),
            None => {
                self.errors.push(FieldError::new(field, "must be a string"));
                None
            }
        }
    }

    pub fn push(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(FieldError::new(field, message));
    }
}
