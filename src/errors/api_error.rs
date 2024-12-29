use ntex::http::StatusCode;
use ntex::web::WebResponseError;

use super::config_error::ConfigError;
use super::database_error::DatabaseError;
use super::token_error::TokenError;
use super::user_error::UserError;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),
    #[error(transparent)]
    TokenError(#[from] TokenError),
    #[error(transparent)]
    UserError(#[from] UserError),
}

impl WebResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ConfigError(error) => error.status_code(),
            ApiError::DatabaseError(error) => error.status_code(),
            ApiError::TokenError(error) => error.status_code(),
            ApiError::UserError(error) => error.status_code(),
        }
    }
}
