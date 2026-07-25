use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// URL of the AMOS API server.
    pub api_url: String,
    /// AMOS User ID (from login).
    pub user_id: Option<String>,
    /// User email (from login).
    pub user_email: Option<String>,
    /// Unique agent ID for this installation.
    pub agent_id: String,
    /// WebSocket URL for real-time commands from AMOS API.
    pub ws_url: Option<String>,
    /// Device-agent API key (from AMOS dashboard).
    pub device_agent_key: Option<String>,
    /// Device-agent API secret (from AMOS dashboard).
    pub device_agent_secret: Option<String>,
    /// Workspace UUID for this user (auto-created on first run).
    pub workspace_id: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: String::from("http://127.0.0.1:8100"),
            user_id: None,
            user_email: None,
            agent_id: uuid::Uuid::new_v4().to_string(),
            ws_url: None,
            device_agent_key: None,
            device_agent_secret: None,
            workspace_id: None,
        }
    }
}

pub struct ConfigStore {
    config: Config,
    path: PathBuf,
}

impl ConfigStore {
    /// Load config from disk, or return defaults if not found.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;
        if !path.exists() {
            info!("Config not found at {:?}, using defaults", path);
            return Ok(Self {
                config: Config::default(),
                path,
            });
        }

        let contents = fs::read_to_string(&path)?;
        let config: Config =
            toml::from_str(&contents).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        info!("Config loaded from {:?}", path);
        Ok(Self { config, path })
    }

    /// Save config to disk.
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        let contents = toml::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        fs::write(&self.path, contents).map_err(|e| ConfigError::IoError(e.to_string()))?;

        info!("Config saved to {:?}", self.path);
        Ok(())
    }

    /// Get the API URL.
    pub fn get_api_url(&self) -> String {
        self.config.api_url.clone()
    }

    /// Set the API URL.
    pub fn set_api_url(&mut self, url: String) {
        self.config.api_url = url;
    }

    /// Get the user ID.
    pub fn get_user_id(&self) -> Option<String> {
        self.config.user_id.clone()
    }

    /// Set the user ID.
    pub fn set_user_id(&mut self, user_id: Option<String>) {
        self.config.user_id = user_id;
    }

    /// Get the user email.
    pub fn get_user_email(&self) -> Option<String> {
        self.config.user_email.clone()
    }

    /// Set the user email.
    pub fn set_user_email(&mut self, email: Option<String>) {
        self.config.user_email = email;
    }

    /// Get the agent ID.
    pub fn get_agent_id(&self) -> String {
        self.config.agent_id.clone()
    }

    /// Get the WebSocket URL.
    pub fn get_ws_url(&self) -> Option<String> {
        self.config.ws_url.clone()
    }

    /// Set the WebSocket URL.
    pub fn set_ws_url(&mut self, url: Option<String>) {
        self.config.ws_url = url;
    }

    /// Get the device-agent API key.
    pub fn get_device_agent_key(&self) -> Option<String> {
        self.config.device_agent_key.clone()
    }

    /// Set the device-agent API key.
    pub fn set_device_agent_key(&mut self, key: Option<String>) {
        self.config.device_agent_key = key;
    }

    /// Get the device-agent API secret.
    pub fn get_device_agent_secret(&self) -> Option<String> {
        self.config.device_agent_secret.clone()
    }

    /// Set the device-agent API secret.
    pub fn set_device_agent_secret(&mut self, secret: Option<String>) {
        self.config.device_agent_secret = secret;
    }

    /// Get the workspace ID.
    pub fn get_workspace_id(&self) -> Option<String> {
        self.config.workspace_id.clone()
    }

    /// Set the workspace ID.
    pub fn set_workspace_id(&mut self, workspace_id: Option<String>) {
        self.config.workspace_id = workspace_id;
    }

    fn config_path() -> Result<PathBuf, ConfigError> {
        let base = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(base.join("amos-companion").join("config.toml"))
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self {
            config: Config::default(),
            path: Self::config_path().unwrap_or_else(|_| PathBuf::from("config.toml")),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(String),

    #[error("Failed to parse config: {0}")]
    ParseError(String),

    #[error("Failed to serialize config: {0}")]
    SerializeError(String),

    #[error("Could not determine config directory")]
    NoConfigDir,
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::IoError(e.to_string())
    }
}
