use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use crate::{
    error::FieldError,
    extract::{FromJson, ObjectReader},
};

/// Widget types the contract's `WidgetLayout.type` allows.
pub const WIDGET_TYPES: [&str; 5] = ["weather", "crypto", "github", "news", "status"];

/// Dashboard model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dashboard {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub layout_json: JsonValue,
    pub settings_json: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Validated create request; omitted JSON fields take the contract defaults.
#[derive(Debug)]
pub struct CreateDashboardRequest {
    pub name: String,
    pub layout_json: JsonValue,
    pub settings_json: JsonValue,
}

/// Validated update request; `None` keeps the stored value.
#[derive(Debug)]
pub struct UpdateDashboardRequest {
    pub name: Option<String>,
    pub layout_json: Option<JsonValue>,
    pub settings_json: Option<JsonValue>,
}

/// `{"widgets":[]}`, the stored layout when none is given.
pub fn default_layout() -> JsonValue {
    json!({ "widgets": [] })
}

/// `{}`, the stored settings when none are given.
pub fn default_settings() -> JsonValue {
    json!({})
}

impl FromJson for CreateDashboardRequest {
    fn from_json(value: JsonValue) -> Result<Self, Vec<FieldError>> {
        let mut r = ObjectReader::new(&value);
        let name = r.required_string("name").and_then(|n| non_blank(&mut r, n));
        let (layout_json, settings_json) = read_json_fields(&mut r);
        match name {
            Some(name) if r.errors.is_empty() => Ok(Self {
                name,
                layout_json: layout_json.unwrap_or_else(default_layout),
                settings_json: settings_json.unwrap_or_else(default_settings),
            }),
            _ => Err(r.errors),
        }
    }
}

impl FromJson for UpdateDashboardRequest {
    fn from_json(value: JsonValue) -> Result<Self, Vec<FieldError>> {
        let mut r = ObjectReader::new(&value);
        let name = r.optional_string("name").and_then(|n| non_blank(&mut r, n));
        let (layout_json, settings_json) = read_json_fields(&mut r);
        if r.errors.is_empty() {
            Ok(Self {
                name,
                layout_json,
                settings_json,
            })
        } else {
            Err(r.errors)
        }
    }
}

fn non_blank(r: &mut ObjectReader<'_>, name: String) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        r.push("name", "must not be blank");
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Validate present `layout_json` / `settings_json` against the contract shapes. An explicit
/// `null` counts as absent.
fn read_json_fields(r: &mut ObjectReader<'_>) -> (Option<JsonValue>, Option<JsonValue>) {
    let layout = r.get("layout_json").cloned();
    if let Some(layout) = &layout {
        validate_layout(layout, &mut r.errors);
    }
    let settings = r.get("settings_json").cloned();
    if let Some(settings) = &settings {
        validate_settings(settings, &mut r.errors);
    }
    (layout, settings)
}

/// `LayoutConfig`: `{"widgets": [WidgetLayout]}` with no other keys.
pub fn validate_layout(layout: &JsonValue, errors: &mut Vec<FieldError>) {
    const FIELD: &str = "layout_json";
    let Some(object) = layout.as_object() else {
        errors.push(FieldError::new(FIELD, "must be an object"));
        return;
    };
    for key in object.keys().filter(|k| k.as_str() != "widgets") {
        errors.push(FieldError::new(format!("{FIELD}.{key}"), "is not allowed"));
    }
    match object.get("widgets") {
        None => errors.push(FieldError::new(format!("{FIELD}.widgets"), "is required")),
        Some(JsonValue::Array(widgets)) => {
            for (i, widget) in widgets.iter().enumerate() {
                validate_widget(widget, &format!("{FIELD}.widgets.{i}"), errors);
            }
        }
        Some(_) => errors.push(FieldError::new(
            format!("{FIELD}.widgets"),
            "must be an array",
        )),
    }
}

fn validate_widget(widget: &JsonValue, prefix: &str, errors: &mut Vec<FieldError>) {
    let Some(object) = widget.as_object() else {
        errors.push(FieldError::new(prefix, "must be an object"));
        return;
    };
    let field = |name: &str| format!("{prefix}.{name}");
    const KEYS: [&str; 7] = ["id", "type", "x", "y", "w", "h", "config"];
    for key in object.keys().filter(|k| !KEYS.contains(&k.as_str())) {
        errors.push(FieldError::new(field(key), "is not allowed"));
    }
    match object.get("id") {
        None => errors.push(FieldError::new(field("id"), "is required")),
        Some(JsonValue::String(id)) if !id.is_empty() => {}
        Some(_) => errors.push(FieldError::new(field("id"), "must be a non-empty string")),
    }
    match object.get("type") {
        None => errors.push(FieldError::new(field("type"), "is required")),
        Some(JsonValue::String(t)) if WIDGET_TYPES.contains(&t.as_str()) => {}
        Some(_) => errors.push(FieldError::new(
            field("type"),
            format!("must be one of {}", WIDGET_TYPES.join(", ")),
        )),
    }
    for (name, min) in [("x", 0), ("y", 0), ("w", 1), ("h", 1)] {
        match object.get(name) {
            None => errors.push(FieldError::new(field(name), "is required")),
            Some(v) if is_integer_at_least(v, min) => {}
            Some(_) => errors.push(FieldError::new(
                field(name),
                format!("must be an integer >= {min}"),
            )),
        }
    }
    if let Some(config) = object.get("config") {
        if !config.is_object() {
            errors.push(FieldError::new(field("config"), "must be an object"));
        }
    }
}

/// `DashboardSettings`: an object; `theme`, `title`, `refresh_interval` typed when present.
pub fn validate_settings(settings: &JsonValue, errors: &mut Vec<FieldError>) {
    const FIELD: &str = "settings_json";
    let Some(object) = settings.as_object() else {
        errors.push(FieldError::new(FIELD, "must be an object"));
        return;
    };
    if let Some(theme) = object.get("theme") {
        if !matches!(theme.as_str(), Some("light" | "dark")) {
            errors.push(FieldError::new(
                format!("{FIELD}.theme"),
                "must be one of light, dark",
            ));
        }
    }
    if let Some(title) = object.get("title") {
        if !title.is_string() {
            errors.push(FieldError::new(
                format!("{FIELD}.title"),
                "must be a string",
            ));
        }
    }
    if let Some(interval) = object.get("refresh_interval") {
        if !is_integer_at_least(interval, 0) {
            errors.push(FieldError::new(
                format!("{FIELD}.refresh_interval"),
                "must be an integer >= 0",
            ));
        }
    }
}

/// JSON Schema `integer` (a number with no fractional part) with a lower bound.
fn is_integer_at_least(value: &JsonValue, min: i64) -> bool {
    if let Some(i) = value.as_i64() {
        return i >= min;
    }
    if value.is_u64() {
        return true;
    }
    value
        .as_f64()
        .is_some_and(|f| f.is_finite() && f.fract() == 0.0 && f >= min as f64)
}

/// Dashboard response
#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub layout_json: JsonValue,
    pub settings_json: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Dashboard> for DashboardResponse {
    fn from(dashboard: Dashboard) -> Self {
        Self {
            id: dashboard.id,
            user_id: dashboard.user_id,
            name: dashboard.name,
            layout_json: dashboard.layout_json,
            settings_json: dashboard.settings_json,
            created_at: dashboard.created_at,
            updated_at: dashboard.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CreateDashboardRequest, UpdateDashboardRequest};
    use crate::extract::FromJson;
    use serde_json::json;

    fn fields(errors: Vec<crate::error::FieldError>) -> Vec<String> {
        errors.into_iter().map(|e| e.field).collect()
    }

    #[test]
    fn create_defaults_missing_or_null_json_fields() {
        let req = CreateDashboardRequest::from_json(json!({"name": " Home "})).unwrap();
        assert_eq!(req.name, "Home");
        assert_eq!(req.layout_json, json!({"widgets": []}));
        assert_eq!(req.settings_json, json!({}));
        let req = CreateDashboardRequest::from_json(
            json!({"name": "x", "layout_json": null, "settings_json": null}),
        )
        .unwrap();
        assert_eq!(req.layout_json, json!({"widgets": []}));
    }

    #[test]
    fn layout_errors_use_dotted_fields() {
        let errors = CreateDashboardRequest::from_json(json!({
            "name": "x",
            "layout_json": {"widgets": [
                {"id": "a", "type": "weather", "x": 0, "y": 0, "w": 2, "h": 2, "config": {}},
                {"id": "", "type": "clock", "x": -1, "y": 1.5, "w": 0, "extra": true}
            ], "other": 1},
            "settings_json": {"theme": "blue", "refresh_interval": -5}
        }))
        .unwrap_err();
        let mut got = fields(errors);
        got.sort();
        let mut want = vec![
            "layout_json.other",
            "layout_json.widgets.1.extra",
            "layout_json.widgets.1.id",
            "layout_json.widgets.1.type",
            "layout_json.widgets.1.x",
            "layout_json.widgets.1.y",
            "layout_json.widgets.1.w",
            "layout_json.widgets.1.h",
            "settings_json.theme",
            "settings_json.refresh_interval",
        ];
        want.sort();
        assert_eq!(got, want);
    }

    #[test]
    fn non_object_shapes_are_rejected() {
        let errors = CreateDashboardRequest::from_json(
            json!({"name": "x", "layout_json": [], "settings_json": "s"}),
        )
        .unwrap_err();
        assert_eq!(fields(errors), ["layout_json", "settings_json"]);
        let errors =
            UpdateDashboardRequest::from_json(json!({"layout_json": {"widgets": {}}})).unwrap_err();
        assert_eq!(fields(errors), ["layout_json.widgets"]);
    }

    #[test]
    fn blank_or_mistyped_name_is_rejected() {
        for body in [json!({"name": "   "}), json!({"name": 3})] {
            let errors = UpdateDashboardRequest::from_json(body).unwrap_err();
            assert_eq!(fields(errors), ["name"]);
        }
        let errors = CreateDashboardRequest::from_json(json!({})).unwrap_err();
        assert_eq!(fields(errors), ["name"]);
        let ok = UpdateDashboardRequest::from_json(json!({"settings_json": {"x": [1]}})).unwrap();
        assert!(ok.name.is_none() && ok.layout_json.is_none());
    }
}
