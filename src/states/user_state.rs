use std::sync::Arc;

use crate::services::UserService;

#[derive(Clone)]
pub struct UserState {
    pub user_service: Arc<UserService>,
}
