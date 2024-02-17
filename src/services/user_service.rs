use std::sync::Arc;

use crate::entities::user::User;
use crate::errors::api_error::ApiError;
use crate::errors::user_error::UserError;
use crate::payload::user_dto::{UserCreateDto, UserIdentity, UserQueryDto};
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

    pub async fn create_user(&self, data: UserCreateDto) -> Result<User, ApiError> {
        let UserCreateDto { username, email, password } = data.into();

        let user_exist = self.user_repo.find_by_email(&email).await.is_some();
        let email_exist = self.user_repo.find_by_username(&username).await.is_some();

        if !user_exist && !email_exist {
            let user_data = UserCreateDto { username, email, password };
            let user = self.user_repo.add(user_data).await?;

            Ok(user)
        } else {
            Err(UserError::UserAlreadyExists)?
        }
    }

    pub async fn find_user(&self, data: UserQueryDto) -> Result<User, ApiError> {
        let user = match data.identity {
            UserIdentity::Username(username) => {
                self.user_repo.find_by_username(&username)
                    .await
                    .ok_or(UserError::UserNotFound)?
            }
            UserIdentity::Email(email) => {
                self.user_repo.find_by_email(&email)
                    .await
                    .ok_or(UserError::UserNotFound)?
            }
        };

        Ok(user)
    }
}