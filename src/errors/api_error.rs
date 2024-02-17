use ntex::http::StatusCode;
use ntex::web::WebResponseError;

use crate::errors::database_error::DatabaseError;
use crate::errors::token_error::TokenError;
use crate::errors::user_error::UserError;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
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
            ApiError::TokenError(error) => error.status_code(),
            ApiError::UserError(error) => error.status_code(),
            ApiError::DatabaseError(error) => error.status_code(),
        }
    }
}