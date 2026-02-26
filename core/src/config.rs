use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_DIR_NAME: &str = ".kirei";
const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub default_repo: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LinearConfig {
    pub default_workspace: Option<String>,
    pub token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrelloConfig {
    pub default_board: Option<String>,
    pub api_key: Option<String>,
    pub token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JiraConfig {
    pub server_url: Option<String>,
    pub default_project: Option<String>,
    pub email: Option<String>,
    pub token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_provider: String,
    pub github: GitHubConfig,
    pub linear: LinearConfig,
    pub trello: TrelloConfig,
    pub jira: JiraConfig,
}

pub struct ConfigStore {
    dir: PathBuf,
    path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Result<Self, anyhow::Error> {
        let base_dirs = directories::BaseDirs::new()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
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

    pub fn load_or_default(&self) -> Result<Config, anyhow::Error> {
        if !self.path.exists() {
            return Ok(Config::default());
        }

        let bytes = fs::read(&self.path)?;
        match serde_json::from_slice::<Config>(&bytes) {
            Ok(config) => Ok(config),
            Err(_) => {
                // Config file exists but has old/invalid format, return default
                Ok(Config::default())
            }
        }
    }

    pub fn save(&self, config: &Config) -> Result<PathBuf, anyhow::Error> {
        fs::create_dir_all(&self.dir)?;

        let json = serde_json::to_vec_pretty(config)?;
        fs::write(&self.path, json)?;
        Ok(self.path.clone())
    }
}
