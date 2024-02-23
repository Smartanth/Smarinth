use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logger {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Control {
    pub embed: bool,
    pub client_id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    pub secret: String,
    pub expiration: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub logger: Logger,
    pub database: Database,
    pub control: Control,
    pub auth: Auth,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or("development".into());

        let mut builder = Config::builder()
            .add_source(File::with_name("configs/default"))
            .add_source(File::with_name(&format!("configs/{}", run_mode)).required(false))
            .add_source(Environment::default().separator("__"));

        if let Ok(port) = env::var("PORT") {
            builder = builder.set_override("server.port", port)?;
        }

        builder.build()?.try_deserialize()
    }
}