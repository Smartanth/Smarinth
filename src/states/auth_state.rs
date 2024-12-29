use std::sync::Arc;

use crate::services::AuthService;
use crate::services::TokenService;
use crate::services::UserService;

#[derive(Clone)]
pub struct AuthState {
    pub auth_service: Arc<AuthService>,
    pub token_service: Arc<TokenService>,
    pub user_service: Arc<UserService>,
}
