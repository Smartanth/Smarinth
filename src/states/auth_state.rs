use std::sync::Arc;

use crate::services::AuthService;
use crate::services::TokenService;

#[derive(Clone)]
pub struct AuthState {
    pub auth_service: Arc<AuthService>,
    pub token_service: Arc<TokenService>,
}
