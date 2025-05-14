use anyhow::Context;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub packages_bucket_name: String,
    pub assets_bucket_name: String,
    pub aws_region: String,
    pub sentry_url: Option<String>,
    pub slack_hook_url: Option<String>,
    pub victoria_metrics_auth_token: String,
}

impl Config {
    pub fn new() -> anyhow::Result<Config> {
        _ = dotenvy::dotenv();

        Ok(Config {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required.")?,
            packages_bucket_name: env::var("PACKAGES_BUCKET_NAME")
                .context("PACKAGES_BUCKET_NAME is required.")?,
            assets_bucket_name: env::var("ASSETS_BUCKET_NAME")
                .context("ASSETS_BUCKET_NAME is required.")?,
            aws_region: env::var("AWS_REGION").context("AWS_REGION is required.")?,
            sentry_url: env::var("SENTRY_URL").ok(),
            slack_hook_url: env::var("SLACK_HOOK_URL").ok(),
            victoria_metrics_auth_token: env::var("VICTORIA_METRICS_AUTH_TOKEN")
                .context("VICTORIA_METRICS_AUTH_TOKEN is required.")?,
        })
    }
}
