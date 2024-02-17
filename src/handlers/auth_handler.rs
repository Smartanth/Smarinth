use ntex::web::{Error, HttpResponse, post, Responder};
use ntex::web::types::{Json, State};
use serde::{Deserialize, Serialize};

use crate::errors::user_error::UserError;
use crate::payload::user_dto::{UserCreateDto, UserDto, UserIdentity, UserQueryDto};
use crate::states::auth_state::AuthState;
use crate::states::user_state::UserState;

#[derive(Clone, Serialize, Deserialize)]
struct LoginPayload {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

#[post("/login")]
pub async fn auth(
    payload: Json<LoginPayload>,
    auth_state: State<AuthState>,
) -> Result<impl Responder, Error> {
    let Json(LoginPayload { username, email, password }) = payload;
    let query_data = UserQueryDto {
        identity: match username {
            Some(username) => UserIdentity::Username(username),
            None => match email {
                Some(email) => UserIdentity::Email(email),
                None => Err(UserError::MissingIdentity)?,
            },
        },
        password: password.clone(),
    };

    let user = auth_state.user_service.find_user(query_data).await?;

    match auth_state.auth_service.verify_password(&user, &password) {
        true => {
            let result = auth_state.token_service.generate_token(user)?;

            Ok(HttpResponse::Ok().json(&result))
        },
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
    payload: Json<RegisterPayload>,
    user_state: State<UserState>,
) -> Result<impl Responder, Error> {
    let Json(RegisterPayload { username, email, password }) = payload;
    let create_data = UserCreateDto { username, email, password };

    let user = user_state.user_service.create_user(create_data).await?;
    let result = UserDto::from(user);

    Ok(HttpResponse::Ok().json(&result))
}