use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::entities::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserIdentity {
    Id(i32),
    Username(String),
    Email(String),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserQueryDto {
    pub identity: UserIdentity,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserCreateDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserDto {
    pub id: i32,
    pub username: String,
    pub email: String,
}

impl From<User> for UserQueryDto {
    fn from(model: User) -> UserQueryDto {
        Self {
            identity: UserIdentity::Id(model.id),
            password: model.password,
        }
    }
}

impl From<User> for UserCreateDto {
    fn from(model: User) -> UserCreateDto {
        Self {
            username: model.username,
            email: model.email,
            password: model.password,
        }
    }
}

impl From<User> for UserDto {
    fn from(model: User) -> UserDto {
        Self {
            id: model.id,
            username: model.username,
            email: model.email,
        }
    }
}

impl Debug for UserQueryDto {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("User")
            .field("identity", &self.identity)
            .finish()
    }
}

impl Debug for UserCreateDto {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("User")
            .field("username", &self.username)
            .field("email", &self.email)
            .finish()
    }
}
