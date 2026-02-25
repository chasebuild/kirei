use crate::error::CoreError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_name: "World".to_string(),
        }
    }
}

pub struct ConfigStore {
    project_dirs: ProjectDirs,
}

impl ConfigStore {
    pub fn new(qualifier: &str, organization: &str, application: &str) -> Result<Self, CoreError> {
        let project_dirs = ProjectDirs::from(qualifier, organization, application)
            .ok_or(CoreError::NoConfigDir)?;
        Ok(Self { project_dirs })
    }

    pub fn dir(&self) -> &Path {
        self.project_dirs.config_dir()
    }

    pub fn path(&self) -> PathBuf {
        self.dir().join("config.json")
    }

    pub fn load_or_default(&self) -> Result<Config, CoreError> {
        let path = self.path();
        if !path.exists() {
            return Ok(Config::default());
        }

        let bytes = fs::read(&path).map_err(|source| CoreError::ReadConfig {
            path: path.clone(),
            source,
        })?;
        serde_json::from_slice(&bytes).map_err(|source| CoreError::ParseConfig {
            path: path.clone(),
            source,
        })
    }

    pub fn save(&self, config: &Config) -> Result<PathBuf, CoreError> {
        let dir = self.dir().to_path_buf();
        fs::create_dir_all(&dir)
            .map_err(|source| CoreError::CreateConfigDir { path: dir, source })?;

        let path = self.path();
        let json = serde_json::to_vec_pretty(config)
            .map_err(|source| CoreError::SerializeConfig { source })?;
        fs::write(&path, json).map_err(|source| CoreError::WriteConfig {
            path: path.clone(),
            source,
        })?;
        Ok(path)
    }
}
