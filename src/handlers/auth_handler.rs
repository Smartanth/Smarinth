use axum::extract::State;
use axum::Json;

use crate::errors::api_error::ApiError;
use crate::errors::user_error::UserError;
use crate::payload::token_dto::TokenDto;
use crate::payload::user_dto::UserLoginDto;
use crate::repository::user_repository::UserRepository;
use crate::states::auth_state::AuthState;

pub async fn auth<R>(
    State(state): State<AuthState<R>>,
    Json(payload): Json<UserLoginDto>,
) -> Result<Json<TokenDto>, ApiError>
    where
        R: UserRepository
{
    let user = state
        .user_repo
        .find_by_email(&payload.email)
        .await
        .ok_or(UserError::UserNotFound)?;

    match state.auth_service.verify_password(&user, &payload.password) {
        true => Ok(Json(state.token_service.generate_token(user)?)),
        false => Err(UserError::InvalidPassword)?,
    }
}