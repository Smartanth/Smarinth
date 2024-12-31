use ntex::http::StatusCode;
use ntex::web::WebResponseError;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Token Validation Error: The provided token is invalid. Details: {0}.")]
    InvalidToken(String),

    #[error("Token Expiration Error: The token has expired.")]
    TokenExpired,

    #[error("Token Generation Error: Failed to generate the token.")]
    TokenCreationError(String),

    #[error("Authentication Error: Missing Bearer token in the request.")]
    MissingToken,

    #[error("Authentication Error: The provided password is invalid.")]
    InvalidPassword,

    #[error("Authentication Error: Failed to hash the password.")]
    PasswordHashError(String),
}

impl WebResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
            AuthError::MissingToken => StatusCode::UNAUTHORIZED,
            AuthError::TokenCreationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidPassword => StatusCode::BAD_REQUEST,
            AuthError::PasswordHashError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
