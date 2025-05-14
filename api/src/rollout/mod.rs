pub mod routes;
pub mod schema;

use crate::rollout::schema::DistributionRolloutStats;
use sqlx::PgPool;

impl DistributionRolloutStats {
    pub async fn get(distribution_id: i32, pg_pool: &PgPool) -> anyhow::Result<Self> {
        let result = match sqlx::query_as!(
            Self,
            "
            SELECT
                r.distribution_id,
                COALESCE(COUNT(*), 0) as total_devices,
                COALESCE(COUNT(*) FILTER (WHERE d.release_id = d.target_release_id), 0) as updated_devices,
                COALESCE(COUNT(*) FILTER (WHERE d.release_id != d.target_release_id), 0) as pending_devices
            FROM device d
            JOIN release r ON d.target_release_id = r.id
            WHERE d.target_release_id IS NOT NULL
            AND r.distribution_id = $1
            GROUP BY r.distribution_id
            ",
            distribution_id
        )
          .fetch_optional(pg_pool)
          .await? {
            Some(r) => r,
            None => Self {
                distribution_id,
                total_devices: Some(0),
                updated_devices: Some(0),
                pending_devices: Some(0),
            },
        };
        Ok(result)
    }
}
