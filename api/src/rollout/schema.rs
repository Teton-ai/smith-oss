use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DistributionRolloutStats {
    pub distribution_id: i32,
    pub total_devices: Option<i64>,
    pub updated_devices: Option<i64>,
    pub pending_devices: Option<i64>,
}
