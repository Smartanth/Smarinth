use crate::errors::api_error::ApiError;
use crate::errors::db_error::DbError;
use crate::errors::user_error::UserError;
use crate::payload::user_dto::{UserDto, UserRegisterDto};
use crate::repository::user_repository::UserRepository;

#[derive(Clone)]
pub struct UserService<R> {
    user_repo: R,
}

impl<R> UserService<R> where R: UserRepository {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    pub async fn create_user(&self, payload: UserRegisterDto) -> Result<UserDto, ApiError> {
        return match self.user_repo.find_by_email(&payload.email).await {
            Some(_) => Err(UserError::UserAlreadyExists)?,
            None => {
                let user = self.user_repo.add(payload).await;

                match user {
                    Ok(user) => Ok(UserDto::from(user)),
                    Err(e) => match e {
                        sqlx::Error::Database(e) => match e.code() {
                            Some(code) => {
                                if code == "23000" {
                                    Err(DbError::UniqueConstraintViolation(e.to_string()))?
                                } else {
                                    Err(DbError::InternalDatabaseError(e.to_string()))?
                                }
                            }
                            _ => Err(DbError::InternalDatabaseError(e.to_string()))?,
                        },
                        _ => Err(DbError::InternalDatabaseError(e.to_string()))?,
                    }
                }
            }
        };
    }
}