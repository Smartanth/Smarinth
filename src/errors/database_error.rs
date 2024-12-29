use ntex::http::StatusCode;
use ntex::web::WebResponseError;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Internal database error: {0}")]
    InternalDatabaseError(String),
    #[error("Unique constraint violation")]
    UniqueConstraintViolation,
}

impl WebResponseError for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            DatabaseError::InternalDatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::UniqueConstraintViolation => StatusCode::CONFLICT,
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(err) => {
                if err
                    .code()
                    .map_or(false, |code| code == "23000" || code == "1062")
                {
                    DatabaseError::UniqueConstraintViolation
                } else {
                    DatabaseError::InternalDatabaseError(err.to_string())
                }
            }
            _ => DatabaseError::InternalDatabaseError(err.to_string()),
        }
    }
}
