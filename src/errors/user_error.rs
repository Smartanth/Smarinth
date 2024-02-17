use ntex::http::StatusCode;
use ntex::web::WebResponseError;

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("Missing user name or email")]
    MissingIdentity,
    #[error("User account not found")]
    UserNotFound,
    #[error("User account already exists")]
    UserAlreadyExists,
    #[error("Fail to create user account")]
    UserCreateFail,
    #[error("Invalid password")]
    InvalidPassword,
}

impl WebResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserError::MissingIdentity => StatusCode::BAD_REQUEST,
            UserError::UserNotFound => StatusCode::NOT_FOUND,
            UserError::UserAlreadyExists => StatusCode::BAD_REQUEST,
            UserError::UserCreateFail => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::InvalidPassword => StatusCode::BAD_REQUEST,
        }
    }
}