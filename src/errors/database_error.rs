use ntex::http::StatusCode;
use ntex::web::WebResponseError;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Internal database error: {0}")]
    InternalDatabaseError(String),

    #[error("Internal sqlx layer error {0}")]
    InternalSqlxError(String),

    #[error("Unique constraint violation")]
    UniqueConstraintViolation,
}

impl WebResponseError for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            DatabaseError::UniqueConstraintViolation => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => DatabaseError::UniqueConstraintViolation, // Postgres
                        "1062" => DatabaseError::UniqueConstraintViolation, // Mysql
                        "2067" => DatabaseError::UniqueConstraintViolation, // Sqlite
                        _ => DatabaseError::InternalDatabaseError(db_err.message().to_string()),
                    }
                } else {
                    DatabaseError::InternalDatabaseError(db_err.to_string())
                }
            }
            _ => DatabaseError::InternalSqlxError(err.to_string()),
        }
    }
}
