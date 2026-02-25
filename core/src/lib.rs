pub mod config;
pub mod error;

pub use config::{Config, ConfigStore};

pub fn greeting(user_name: &str) -> String {
    format!("Hello, {user_name}!")
}
