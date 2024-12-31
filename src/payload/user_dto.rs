use serde::{Deserialize, Serialize};

use crate::entities::User;

#[derive(Clone, Serialize, Deserialize)]
pub enum UserIdentity {
    Id(i32),
    Username(String),
    Email(String),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserAuthDto {
    pub identity: UserIdentity,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserCreateDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserUpdateDto {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UserDto {
    pub id: i32,
    pub username: String,
    pub email: String,
}

impl From<User> for UserDto {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
        }
    }
}
