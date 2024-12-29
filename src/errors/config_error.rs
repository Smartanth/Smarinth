use std::io;
use std::string::FromUtf8Error;

use ntex::http::StatusCode;
use ntex::web::WebResponseError;
use toml::{de, ser};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Incompatible types at path {path:?}, expected {expected_type:?} received {actual_type:?}.")]
    IncompatibleTypeError { path: String, expected_type: String, actual_type: String },

    #[error("Failed to load the configuration file as UTF-8.")]
    Utf8LoadError(#[from] FromUtf8Error),

    #[error("Failed to serialize the configuration file.")]
    SerializeError(#[from] ser::Error),

    #[error("Failed to deserialize the configuration file.")]
    DeserializeError(#[from] de::Error),

    #[error("File path operation failed.")]
    PathError(#[from] io::Error),
}

impl WebResponseError for ConfigError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
