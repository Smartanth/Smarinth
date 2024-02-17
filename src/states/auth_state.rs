use std::sync::Arc;

use crate::services::auth_service::AuthService;
use crate::services::token_service::TokenService;
use crate::services::user_service::UserService;

#[derive(Clone)]
pub struct AuthState {
    pub auth_service: Arc<AuthService>,
    pub token_service: Arc<TokenService>,
    pub user_service: Arc<UserService>,
}