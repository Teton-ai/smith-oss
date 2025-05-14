use crate::config::Config;
use crate::storage::Storage;

pub struct Asset;

impl Asset {
    pub async fn new(
        file_name: &str,
        file_data: &[u8],
        config: &'static Config,
    ) -> anyhow::Result<Self> {
        Storage::save_to_s3(&config.assets_bucket_name, None, file_name, file_data).await?;
        Ok(Asset)
    }
}
