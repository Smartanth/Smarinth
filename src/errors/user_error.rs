use ntex::http::StatusCode;
use ntex::web::WebResponseError;

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("User Identity Error: Missing required user identity information. Either name or email must be provided.")]
    MissingIdentity,

    #[error("User Retrieval Error: The specified user account could not be found.")]
    UserNotFound,

    #[error("User Creation Error: A user account with the provided details already exists.")]
    UserAlreadyExists,

    #[error("User Creation Failure: Unable to create the user account. Please verify the provided information and try again.")]
    UserCreateFail,

    #[error("User Update Failure: Unable to update the user account. Please verify the details and attempt the operation again.")]
    UserUpdateFail,
}

impl WebResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserError::MissingIdentity => StatusCode::BAD_REQUEST,
            UserError::UserNotFound => StatusCode::NOT_FOUND,
            UserError::UserAlreadyExists => StatusCode::BAD_REQUEST,
            UserError::UserCreateFail => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::UserUpdateFail => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
