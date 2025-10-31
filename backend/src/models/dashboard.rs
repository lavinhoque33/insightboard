use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

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

/// Create dashboard request
#[derive(Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
    #[serde(default)]
    pub layout_json: JsonValue,
    #[serde(default)]
    pub settings_json: JsonValue,
}

/// Update dashboard request
#[derive(Debug, Deserialize)]
pub struct UpdateDashboardRequest {
    pub name: Option<String>,
    pub layout_json: Option<JsonValue>,
    pub settings_json: Option<JsonValue>,
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
