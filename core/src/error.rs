use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Could not determine a config directory for this platform")]
    NoConfigDir,

    #[error("Failed to create config directory: {path}")]
    CreateConfigDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read config file: {path}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Config file is not valid JSON: {path}")]
    ParseConfig {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to write config file: {path}")]
    WriteConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to serialize config as JSON")]
    SerializeConfig {
        #[source]
        source: serde_json::Error,
    },
}
