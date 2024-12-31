use std::sync::Arc;

use crate::configs::Password;
use crate::errors::{ApiError, AuthError, UserError};
use crate::payload::{TokenClaimsDto, UserAuthDto, UserCreateDao, UserCreateDto, UserDto, UserIdentity};
use crate::repository::UserRepository;

#[derive(Clone)]
pub struct AuthService {
    user_repo: Arc<UserRepository>,
    password: Arc<dyn Password>,
}

impl AuthService {
    pub fn new(user_repo: &Arc<UserRepository>, hasher: &Arc<dyn Password>) -> Self {
        Self {
            user_repo: Arc::clone(user_repo),
            password: Arc::clone(hasher),
        }
    }

    pub async fn authorization_user(&self, data: UserAuthDto) -> Result<UserDto, ApiError> {
        let UserAuthDto { identity, password } = data.into();

        let user = match identity {
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
            _ => Err(UserError::UserNotFound)?
        };

        if self.password.verify(&password, &user.password).unwrap_or(false) {
            Ok(user.into())
        } else {
            Err(AuthError::InvalidPassword)?
        }
    }

    pub async fn create_user(&self, data: UserCreateDto) -> Result<UserDto, ApiError> {
        let UserCreateDto { username, email, password } = data.into();

        let user_exist = self.user_repo.find_by_email(&email).await.is_some();
        let email_exist = self.user_repo.find_by_username(&username).await.is_some();

        if !user_exist && !email_exist {
            let user_data = UserCreateDao { username, email, password };
            let user = self.user_repo.add(user_data).await?;

            Ok(user.into())
        } else {
            Err(UserError::UserAlreadyExists)?
        }
    }

    pub async fn authentication_user(&self, data: TokenClaimsDto) -> Result<UserDto, ApiError> {
        let id = data.sub.parse::<i32>().map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        self.user_repo.find(id).await.ok_or(UserError::UserNotFound.into()).map(UserDto::from)
    }
}
