use crate::magic::structure::ConfigCheck;
use anyhow::{Result, anyhow};
use tracing::{error, info};

pub struct InitialCheck {
    pub name: String,
    pub cmd: String,
    pub success: bool,
    pub data: Option<String>,
}

impl From<ConfigCheck> for InitialCheck {
    fn from(config: ConfigCheck) -> Self {
        InitialCheck {
            name: config.name,
            cmd: config.cmd,
            success: false,
            data: None,
        }
    }
}

impl InitialCheck {
    pub async fn execute(&mut self) -> Result<()> {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&self.cmd)
            .output()
            .await?;

        self.success = output.status.success();
        self.data
            .get_or_insert_with(String::new)
            .push_str(String::from_utf8(output.stdout)?.trim());

        if !self.success {
            error!("[FAIL] [{}] [{}]", self.name, self.cmd);
            Err(anyhow!("Check failed!"))
        } else {
            info!("[ OK ] [{}] [{}]", self.name, self.cmd);
            Ok(())
        }
    }
}
