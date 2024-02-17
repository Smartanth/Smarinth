use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::entities::user::User;

#[derive(Clone, Serialize, Deserialize)]
pub struct UserLoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserRegisterDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserDto {
    pub id: i32,
    pub username: String,
    pub email: String,
}

impl UserDto {
    pub fn from(model: User) -> UserDto {
        Self {
            id: model.id,
            username: model.username,
            email: model.email,
        }
    }
}

impl Debug for UserLoginDto {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("User")
            .field("email", &self.email)
            .finish()
    }
}

impl Debug for UserRegisterDto {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("User")
            .field("username", &self.username)
            .field("email", &self.email)
            .finish()
    }
}