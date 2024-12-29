use ntex::web::{post, types, Error, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::errors::UserError;
use crate::payload::{UserCreateDto, UserDto, UserIdentity};
use crate::states::{AuthState, UserState};

#[derive(Clone, Serialize, Deserialize)]
struct LoginPayload {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

#[post("/login")]
pub async fn auth(
    payload: types::Json<LoginPayload>,
    auth_state: types::State<AuthState>,
) -> Result<impl Responder, Error> {
    let types::Json(LoginPayload {
        username,
        email,
        password,
    }) = payload;
    let identity = match username {
        Some(username) => UserIdentity::Username(username),
        None => match email {
            Some(email) => UserIdentity::Email(email),
            None => Err(UserError::MissingIdentity)?,
        },
    };

    let user = auth_state.user_service.find_user(identity).await?;

    match auth_state.auth_service.verify_password(&user, &password) {
        true => {
            let result = auth_state.token_service.generate_token(user)?;

            Ok(HttpResponse::Ok().json(&result))
        }
        false => Err(UserError::InvalidPassword)?,
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[post("/register")]
pub async fn register(
    payload: types::Json<RegisterPayload>,
    user_state: types::State<UserState>,
) -> Result<impl Responder, Error> {
    let types::Json(RegisterPayload {
        username,
        email,
        password,
    }) = payload;
    let create_data = UserCreateDto {
        username,
        email,
        password,
    };

    let user = user_state.user_service.create_user(create_data).await?;
    let result = UserDto::from(user);

    Ok(HttpResponse::Ok().json(&result))
}
