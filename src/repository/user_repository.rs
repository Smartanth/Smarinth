use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

use axum::async_trait;
use sqlx::Error;

use crate::configs::database::Database;
use crate::configs::database::DatabaseScheme;
use crate::configs::password::Hasher;
use crate::entities::user::User;
use crate::payload::user_dto::UserRegisterDto;
use crate::sql;

#[derive(Clone)]
pub struct RemoteUserRepository {
    pub db_conn: Arc<Database>,
    pub hasher: Arc<dyn Hasher>,
}

#[async_trait]
pub trait UserRepository {
    fn new(hasher: &Arc<dyn Hasher>, db_conn: &Arc<Database>) -> Self;

    async fn find_by_email(&self, email: &str) -> Option<User>;

    async fn find(&self, id: i32) -> Option<User>;

    async fn add(&self, payload: UserRegisterDto) -> Result<User, Error>;
}

#[async_trait]
impl UserRepository for RemoteUserRepository {
    fn new(hasher: &Arc<dyn Hasher>, db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
            hasher: Arc::clone(hasher),
        }
    }

    async fn find_by_email(&self, email: &str) -> Option<User> {
        let statement = sql!(self.db_conn.scheme, "SELECT * FROM users WHERE email = $1");

        let query = sqlx::query_as::<_, User>(&statement).bind(email);

        query.fetch_optional(self.db_conn.get_pool()).await.unwrap_or(None)
    }

    async fn find(&self, id: i32) -> Option<User> {
        let statement = sql!(self.db_conn.scheme, "SELECT * FROM users WHERE id = $1");

        let query = sqlx::query_as::<_, User>(&statement).bind(id);

        query.fetch_optional(self.db_conn.get_pool()).await.unwrap_or(None)
    }

    async fn add(&self, payload: UserRegisterDto) -> Result<User, Error> {
        let user_password = self.hasher.hash(&payload.password).unwrap();

        let statement = sql!(self.db_conn.scheme, "INSERT INTO users (username, email, password) VALUES ($1, $2, $3)");

        let query = sqlx::query(&statement).bind(&payload.username).bind(&payload.email).bind(&user_password);

        let affected = query.execute(self.db_conn.get_pool()).await?.rows_affected();
        if affected > 0 {
            self.find_by_email(&payload.email).await.ok_or(Error::RowNotFound)
        } else {
            Err(Error::Io(io::Error::new(ErrorKind::InvalidData, "Fail to insert data")))
        }
    }
}

#[cfg(test)]
mod user_repository_tests {
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};

    use crate::configs::database::Database;
    use crate::configs::password::{Argon2Hash, Hasher};
    use crate::configs::settings::Settings;

    use super::*;

    #[derive(sqlx::FromRow, Clone, Deserialize, Serialize)]
    pub struct User {
        pub id: i32,
        pub username: String,
        pub email: String,
        pub password: String,
    }

    #[tokio::test]
    async fn test_add_and_find() {
        let settings = Arc::new(Settings::new(Option::from("test".to_string())).expect("Failed to load settings."));
        let db_conn = Arc::new(Database::new(&settings).await.unwrap());
        let hasher = Arc::new(Argon2Hash::new()) as Arc<dyn Hasher>;
        let repo = RemoteUserRepository::new(&hasher, &db_conn);

        let mock_dto = UserRegisterDto {
            username: "test_user".to_string(),
            email: "test_user@test_email.com".to_string(),
            password: "test_password".to_string(),
        };
        let user = repo.add(mock_dto).await;

        assert!(user.is_ok(), "Should create and find registed user.");

        let statement = sql!(DatabaseScheme::MYSQL, "DELETE FROM users WHERE id = $1");
        let query = sqlx::query(&statement).bind(user.unwrap().id);
        let affected = query.execute(repo.db_conn.get_pool()).await.unwrap().rows_affected();

        assert!(affected > 0, "Record should be remove.")
    }
}