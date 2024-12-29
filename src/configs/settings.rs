use std::{env, fs, io};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use toml::map::Map;
use toml::Value;

use crate::errors::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logger {
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub migration_path: Option<String>,
    pub clean_start: bool,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    pub embed: bool,
    pub client_id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub secret: String,
    pub expiration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

        let mut settings: Settings = toml::from_str(
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/default.toml"))
        )?;

        if let Ok(settings_path) = Self::normalize_path(&format!("configs/{run_mode}.toml")) {
            if settings_path.exists() {
                let merged_settings = Self::merge(
                    Value::try_from(settings.clone())?,
                    Value::from_str(&String::from_utf8(fs::read(settings_path)?)?)?,
                    "$",
                )?;
                settings = Value::try_into(merged_settings)?;
            }
        }

        if let Some(migrate) = &mut settings.database.migration_path {
            settings.database.migration_path = if Path::new(migrate).is_dir() {
                Some(Self::normalize_path(migrate)?.to_string_lossy().to_string())
            } else {
                None
            };
        }

        Ok(settings)
    }

    fn merge_table(value: &mut Map<String, Value>, other: Map<String, Value>, path: &str) -> Result<(), ConfigError> {
        for (name, inner) in other {
            if let Some(existing) = value.remove(&name) {
                let inner_path = format!("{path}.{name}");
                value.insert(name, Self::merge(existing, inner, &inner_path)?);
            } else {
                value.insert(name, inner);
            }
        }

        Ok(())
    }

    fn merge(value: Value, other: Value, path: &str) -> Result<Value, ConfigError> {
        match (value, other) {
            (Value::String(_), Value::String(inner)) => Ok(Value::String(inner)),
            (Value::Integer(_), Value::Integer(inner)) => Ok(Value::Integer(inner)),
            (Value::Float(_), Value::Float(inner)) => Ok(Value::Float(inner)),
            (Value::Boolean(_), Value::Boolean(inner)) => Ok(Value::Boolean(inner)),
            (Value::Datetime(_), Value::Datetime(inner)) => Ok(Value::Datetime(inner)),
            (Value::Array(mut existing), Value::Array(inner)) => {
                existing.extend(inner);
                Ok(Value::Array(existing))
            }
            (Value::Table(mut existing), Value::Table(inner)) => {
                Self::merge_table(&mut existing, inner, path)?;
                Ok(Value::Table(existing))
            }
            (v, o) => Err(
                ConfigError::IncompatibleTypeError {
                    path: String::from(path),
                    expected_type: String::from(v.type_str()),
                    actual_type: String::from(o.type_str()),
                }
            ),
        }
    }

    fn normalize_path(path: &str) -> io::Result<PathBuf> {
        let path_buf = PathBuf::from(path);

        if path_buf.is_absolute() {
            Ok(path_buf)
        } else {
            Ok(env::current_dir()?.join(path_buf))
        }
    }
}
