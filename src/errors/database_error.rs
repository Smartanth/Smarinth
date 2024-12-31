use ntex::http::StatusCode;
use ntex::web::WebResponseError;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Database Migration Error: Failed to migrate the database. Details: {0}.")]
    DatabaseMigrateError(#[from] sqlx::migrate::MigrateError),

    #[error("Database Access Error: Unable to access the database. Details: {0}.")]
    DatabaseAccessError(String),

    #[error("Database SQL Execution Error: Failed to execute the SQL statement. Returned: {0}.")]
    DatabaseExecuteError(String),

    #[error("Database Constraint Violation: A unique constraint violation has been detected.")]
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
                        _ => DatabaseError::DatabaseExecuteError(db_err.message().to_string()),
                    }
                } else {
                    DatabaseError::DatabaseExecuteError(db_err.to_string())
                }
            }
            _ => DatabaseError::DatabaseAccessError(err.to_string()),
        }
    }
}
