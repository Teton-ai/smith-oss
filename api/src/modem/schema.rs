use serde::Serialize;
use sqlx::types::chrono;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct Modem {
    pub id: i32,
    pub imei: String,
    pub network_provider: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
