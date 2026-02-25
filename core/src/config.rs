use crate::error::CoreError;
use crate::unified::ProviderId;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_DIR_NAME: &str = ".kirei";
const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    pub default_provider: ProviderId,
    pub default_repo: Option<String>,
    pub default_workspace: Option<String>,
    pub tokens: HashMap<ProviderId, String>,
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            default_provider: ProviderId::Github,
            default_repo: None,
            default_workspace: None,
            tokens: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user_name: String,
    pub unified: UnifiedConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_name: "World".to_string(),
            unified: UnifiedConfig::default(),
        }
    }
}

pub struct ConfigStore {
    dir: PathBuf,
    path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Result<Self, CoreError> {
        let base_dirs = directories::BaseDirs::new().ok_or(CoreError::NoHomeDir)?;
        let home = base_dirs.home_dir();
        let dir = home.join(CONFIG_DIR_NAME);
        let path = dir.join(CONFIG_FILE_NAME);
        Ok(Self { dir, path })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_or_default(&self) -> Result<Config, CoreError> {
        if !self.path.exists() {
            return Ok(Config::default());
        }

        let bytes = fs::read(&self.path).map_err(|source| CoreError::ReadConfig {
            path: self.path.clone(),
            source,
        })?;
        serde_json::from_slice(&bytes).map_err(|source| CoreError::ParseConfig {
            path: self.path.clone(),
            source,
        })
    }

    pub fn save(&self, config: &Config) -> Result<PathBuf, CoreError> {
        fs::create_dir_all(&self.dir).map_err(|source| CoreError::CreateConfigDir {
            path: self.dir.clone(),
            source,
        })?;

        let json = serde_json::to_vec_pretty(config)
            .map_err(|source| CoreError::SerializeConfig { source })?;
        fs::write(&self.path, json).map_err(|source| CoreError::WriteConfig {
            path: self.path.clone(),
            source,
        })?;
        Ok(self.path.clone())
    }
}
