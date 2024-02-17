use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::payload::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("{0}")]
    InternalDatabaseError(String),
    #[error("Duplicate entry exists")]
    UniqueConstraintViolation(String),
}

impl IntoResponse for DbError {
    fn into_response(self) -> Response {
        let status_code = match self {
            DbError::InternalDatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DbError::UniqueConstraintViolation(_) => StatusCode::CONFLICT,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}