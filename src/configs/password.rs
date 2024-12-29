use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core, Error, SaltString};

pub trait Password: Send + Sync {
    fn new() -> Self where Self: Sized;

    fn hash(&self, password: &str) -> Result<String, Error>;

    fn verify(&self, password: &str, password_hash: &str) -> Result<bool, Error>;
}

#[derive(Debug, Clone)]
pub struct Argon2Hash(Argon2<'static>);

impl Password for Argon2Hash {
    fn new() -> Self {
        Self(Argon2::default())
    }

    fn hash(&self, password: &str) -> Result<String, Error> {
        let hash_salt = SaltString::generate(&mut rand_core::OsRng);
        let hash = self.0.hash_password(password.as_ref(), &hash_salt)?;

        Ok(hash.to_string())
    }

    fn verify(&self, password: &str, password_hash: &str) -> Result<bool, Error> {
        let parsed_hash = PasswordHash::new(password_hash)?;
        let result = self.0.verify_password(password.as_ref(), &parsed_hash);

        Ok(result.is_ok())
    }
}

#[cfg(test)]
mod password_tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hasher = Argon2Hash::new();
        let password = "test_password";
        let password_hash = hasher.hash(password).unwrap();

        assert!(hasher.verify(password, &password_hash).unwrap());
    }
}
