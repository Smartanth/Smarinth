use crate::services::auth_service::AuthService;
use crate::services::token_service::TokenService;

#[derive(Clone)]
pub struct AuthState<R> {
    pub token_service: TokenService,
    pub auth_service: AuthService,
    pub user_repo: R,
}