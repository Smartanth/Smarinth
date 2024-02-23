use ntex::http::StatusCode;
use ntex::web::WebResponseError;
use ntex_mqtt::v5;

#[derive(thiserror::Error, Debug)]
pub enum ControlError {
    #[error("Internal mqtt error")]
    Error,
}

impl WebResponseError for ControlError {
    fn status_code(&self) -> StatusCode {
        match self {
            ControlError::Error => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<()> for ControlError {
    fn from(_value: ()) -> Self {
        ControlError::Error
    }
}

impl TryFrom<ControlError> for v5::PublishAck {
    type Error = ControlError;

    fn try_from(err: ControlError) -> Result<Self, Self::Error> {
        Err(err)
    }
}