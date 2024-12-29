use std::sync::Arc;

use crate::configs::Password;
use crate::entities::User;

#[derive(Clone)]
pub struct AuthService {
    password: Arc<dyn Password>,
}

impl AuthService {
    pub fn new(hasher: &Arc<dyn Password>) -> Self {
        Self {
            password: Arc::clone(hasher),
        }
    }

    pub fn verify_password(&self, user: &User, password: &str) -> bool {
        self.password.verify(password, &user.password).unwrap_or(false)
    }
}
