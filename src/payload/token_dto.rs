use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDto {
    pub token: String,
    pub iat: u64,
    pub exp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaimsDto {
    pub sub: String,
    pub username: String,
    pub email: String,
    pub iat: u64,
    pub exp: u64,
}