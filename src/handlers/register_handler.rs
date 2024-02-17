use axum::extract::State;
use axum::Json;

use crate::errors::api_error::ApiError;
use crate::payload::user_dto::{UserDto, UserRegisterDto};
use crate::repository::user_repository::UserRepository;
use crate::states::user_state::UserState;

pub async fn register<R>(
    State(state): State<UserState<R>>,
    Json(payload): Json<UserRegisterDto>,
) -> Result<Json<UserDto>, ApiError>
    where
        R: UserRepository
{
    let user = state.user_service.create_user(payload).await?;

    Ok(Json(user))
}