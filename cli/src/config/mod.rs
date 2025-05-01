use colored::{Color, ColoredString, Style};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use tokio::process::Command;

const OP_DEFAULT_CONFIG: &str = "op://Engineering/smith.env/config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Profile {
    server: String,
    tunnel_server: String,
    color: String,
    #[serde(default)]
    ask: bool,
    auth0_audience: String,
    auth0_domain: String,
    auth0_client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub current_profile: String,
    profile: HashMap<String, Profile>,
}

impl Config {
    pub async fn default() -> Self {
        // read default from one password
        let default_string = Command::new("op")
            .arg("read")
            .arg(OP_DEFAULT_CONFIG)
            .output()
            .await
            .expect("Failed to read default config from 1password")
            .stdout;

        let default_string = String::from_utf8(default_string).unwrap();
        let default_config: Config = toml::from_str(&default_string).unwrap();

        println!("Default config loaded");
        default_config
    }

    pub async fn load() -> anyhow::Result<Self> {
        let config_file = dirs::home_dir().unwrap().join(".smith").join("config.toml");
        let config_str = tokio::fs::read_to_string(config_file).await?;
        let config: Config = toml::from_str(&config_str)?;

        unsafe {
            std::env::set_var("SMITH_PROFILE", &config.current_profile);
            std::env::set_var(
                "SMITH_SERVER",
                &config.profile[&config.current_profile].server,
            );
        }

        Ok(config)
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let config_file = dirs::home_dir()
            .unwrap()
            .join(".smith")
            .join("config.toml.save");
        let config_str = toml::to_string(&self)?;
        tokio::fs::write(&config_file, config_str).await?;
        tokio::fs::rename(
            config_file,
            dirs::home_dir().unwrap().join(".smith").join("config.toml"),
        )
        .await?;
        Ok(())
    }

    pub async fn change_profile(&mut self, profile: String) -> anyhow::Result<()> {
        // Check if the profile exists in the configuration
        if !self.profile.contains_key(&profile) {
            return Err(anyhow::anyhow!("Profile '{}' does not exist", profile));
        }

        self.current_profile = profile;
        self.save().await?;

        unsafe {
            std::env::set_var("SMITH_PROFILE", &self.current_profile);
            std::env::set_var("SMITH_SERVER", &self.profile[&self.current_profile].server);
        }

        Ok(())
    }

    pub fn auth0_credentials(&self) -> (String, String, String) {
        let profile = &self.profile[&self.current_profile];
        (
            profile.auth0_domain.clone(),
            profile.auth0_client_id.clone(),
            profile.auth0_audience.clone(),
        )
    }

    pub fn tunnel_server(&self) -> String {
        self.profile[&self.current_profile].tunnel_server.clone()
    }

    pub fn current_domain(&self) -> String {
        self.profile[&self.current_profile].server.clone()
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_profile = self.profile[&self.current_profile].clone();
        let color = Color::from(current_profile.color);
        let mut colored_string = ColoredString::from(self.current_profile.clone());
        colored_string.fgcolor = Some(color);
        colored_string.style = Style::default().bold();
        let mut colored_server = ColoredString::from(current_profile.server.clone());
        colored_server.fgcolor = Some(color);
        write!(f, "{} {}", colored_string, colored_server)
    }
}
