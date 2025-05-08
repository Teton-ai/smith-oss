use zbus::{Result, proxy};

#[proxy(
    interface = "ai.teton.smith.Packages1",
    default_service = "ai.teton.smith",
    default_path = "/ai/teton/smith/Packages"
)]
pub(crate) trait SmithDbus {
    async fn update_packages(&self) -> Result<String>;
    async fn upgrade_packages(&self) -> Result<String>;
    async fn updater_status(&self) -> Result<String>;
    async fn expose_port(&self, port: u16) -> Result<String>;
    async fn schedule_services(&self) -> Result<String>;
    async fn unschedule_services(&self) -> Result<String>;
}
