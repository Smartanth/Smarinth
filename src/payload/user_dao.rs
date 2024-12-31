use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserCreateDao {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserUpdateDao {
    pub id: i32,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}
