use std::sync::Arc;

use crate::configs::password::Hasher;
use crate::entities::user::User;

#[derive(Clone)]
pub struct AuthService {
    hasher: Arc<dyn Hasher>,
}

impl AuthService {
    pub fn new(hasher: &Arc<dyn Hasher>) -> Self {
        Self {
            hasher: Arc::clone(hasher),
        }
    }

    pub fn verify_password(&self, user: &User, password: &str) -> bool {
        self.hasher.verify(password, &user.password).unwrap_or(false)
    }
}