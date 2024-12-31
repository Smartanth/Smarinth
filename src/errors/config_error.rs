use std::io;
use std::string::FromUtf8Error;

use ntex::http::StatusCode;
use ntex::web::WebResponseError;
use toml::{de, ser};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Configuration Error: Type mismatch detected at path '{path}'. Expected type '{expected_type}', but received type '{actual_type}'.")]
    IncompatibleTypeError { path: String, expected_type: String, actual_type: String },

    #[error("Configuration Loading Error: Failed to load the configuration file. Please ensure the file is encoded in valid UTF-8 format.")]
    Utf8LoadError(#[from] FromUtf8Error),

    #[error("Configuration Serialization Error: Unable to serialize the configuration data.")]
    SerializeError(#[from] ser::Error),

    #[error("Configuration Deserialization Error: Unable to deserialize the configuration data.")]
    DeserializeError(#[from] de::Error),

    #[error("File Path Error: Operation on the file path failed with error: {0}.")]
    PathError(#[from] io::Error),
}

impl WebResponseError for ConfigError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
