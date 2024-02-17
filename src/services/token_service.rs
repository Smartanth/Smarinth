use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header, TokenData, Validation};
use jsonwebtoken::errors::ErrorKind;

use crate::configs::settings::Settings;
use crate::entities::user::User;
use crate::errors::token_error::TokenError;
use crate::payload::token_dto::{TokenClaimsDto, TokenDto};

#[derive(Clone)]
pub struct TokenService {
    expiration: u64,
    secret: String,
}

impl TokenService {
    pub fn new(settings: &Arc<Settings>) -> Self {
        Self {
            expiration: settings.auth.expiration,
            secret: settings.auth.secret.clone(),
        }
    }

    pub fn retrieve_token_claims(&self, token: &str) -> Result<TokenData<TokenClaimsDto>, TokenError> {
        match decode::<TokenClaimsDto>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default()
        ) {
            Ok(claims) => Ok(claims),
            Err(err) => match err.kind() {
                ErrorKind::ExpiredSignature => Err(TokenError::TokenExpired)?,
                _ => Err(TokenError::InvalidToken(token.to_string()))?
            }
        }
    }

    pub fn generate_token(&self, user: User) -> Result<TokenDto, TokenError> {
        let iat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let exp = iat + self.expiration;

        let claims = TokenClaimsDto {
            sub: user.id.to_string(),
            email: user.email,
            iat,
            exp,
        };

        let encoding_key = EncodingKey::from_secret(self.secret.as_ref());

        let token = encode(&Header::default(), &claims, &encoding_key)
            .map_err(|e| TokenError::TokenCreationError(e.to_string()))?;

        Ok(TokenDto { token, iat, exp })
    }
}

