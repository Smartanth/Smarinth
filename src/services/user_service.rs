use std::sync::Arc;

use crate::errors::{ApiError, UserError};
use crate::payload::{UserDto, UserIdentity, UserUpdateDao, UserUpdateDto};
use crate::repository::user_repository::UserRepository;

#[derive(Clone)]
pub struct UserService {
    user_repo: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repo: &Arc<UserRepository>) -> Self {
        Self {
            user_repo: Arc::clone(user_repo),
        }
    }

    pub async fn find_user(&self, identity: UserIdentity) -> Result<UserDto, ApiError> {
        let user = match identity {
            UserIdentity::Id(id) => self
                .user_repo
                .find(id)
                .await
                .ok_or(UserError::UserNotFound)?,
            UserIdentity::Username(username) => self
                .user_repo
                .find_by_username(&username)
                .await
                .ok_or(UserError::UserNotFound)?,
            UserIdentity::Email(email) => self
                .user_repo
                .find_by_email(&email)
                .await
                .ok_or(UserError::UserNotFound)?,
        };

        Ok(user.into())
    }

    pub async fn update_user(&self, identity: UserIdentity, data: UserUpdateDto) -> Result<UserDto, ApiError> {
        let UserUpdateDto { username, email, password } = data.into();

        let id = self.find_user(identity).await?.id;

        let user_data = UserUpdateDao { id, username, email, password };

        let user = self.user_repo.update(user_data).await?;

        Ok(user.into())
    }
}
