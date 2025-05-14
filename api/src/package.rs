use crate::config::Config;
use crate::handlers::distributions::types::Package;
use crate::storage::Storage;
use sqlx::PgPool;
use tracing::error;

impl Package {
    pub async fn new(
        name: &str,
        version: &str,
        architecture: &str,
        file_name: &str,
        file_data: &[u8],
        config: &'static Config,
        pool: &PgPool,
    ) -> anyhow::Result<Package> {
        Storage::save_to_s3(&config.packages_bucket_name, None, file_name, file_data).await?;

        match sqlx::query_as!(
            Package,
            "
          INSERT INTO package (name, version, architecture, file)
          VALUES ($1, $2, $3, $4)
          RETURNING *
          ",
            name,
            version,
            architecture,
            file_name
        )
        .fetch_one(pool)
        .await
        {
            Ok(package) => Ok(package),
            Err(err) => {
                let bucket_name = config.packages_bucket_name.clone();
                let file_name = file_name.to_string();
                tokio::spawn(async move {
                    if let Err(e) = Storage::delete_from_s3(&bucket_name, &file_name).await {
                        error!("Failed to delete S3 object after database error: {:?}", e);
                    }
                });
                Err(err.into())
            }
        }
    }
    pub async fn delete(
        package_id: &i32,
        config: &'static Config,
        pool: &PgPool,
    ) -> anyhow::Result<Package> {
        let package = sqlx::query_as!(
            Package,
            "DELETE FROM package WHERE id = $1 RETURNING *
          ",
            package_id,
        )
        .fetch_one(pool)
        .await?;
        Storage::delete_from_s3(&config.packages_bucket_name, &package.file).await?;
        Ok(package)
    }
}
