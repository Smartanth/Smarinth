use axum::extract::{Request, State};
use axum::http;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::headers::{Authorization, Header};
use axum_extra::headers::authorization::Bearer;

use crate::errors::api_error::ApiError;
use crate::errors::token_error::TokenError;
use crate::errors::user_error::UserError;
use crate::repository::user_repository::UserRepository;
use crate::states::auth_state::AuthState;

pub async fn auth<R>(
    State(state): State<AuthState<R>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ApiError>
    where
        R: UserRepository
{
    let mut headers = req.headers_mut()
        .get_all(http::header::AUTHORIZATION)
        .iter();

    let header = Authorization::<Bearer>::decode(&mut headers)
        .map_err(|_| TokenError::MissingToken)?;

    let token = header.token();

    let token_data = state.token_service.retrieve_token_claims(token)?;

    let user = state.user_repo.find_by_email(&token_data.claims.email).await
        .ok_or(UserError::UserNotFound)?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}