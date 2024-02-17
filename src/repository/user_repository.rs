use std::sync::Arc;

use crate::configs::database::{Database, DatabaseScheme};
use crate::configs::password::Password;
use crate::entities::user::User;
use crate::errors::api_error::ApiError;
use crate::errors::database_error::DatabaseError;
use crate::errors::user_error::UserError;
use crate::payload::user_dto::UserCreateDto;
use crate::sql;

#[derive(Clone)]
pub struct UserRepository {
    pub database: Arc<Database>,
    pub password: Arc<dyn Password>,
}

impl UserRepository {
    pub fn new(password: &Arc<dyn Password>, db_conn: &Arc<Database>) -> Self {
        Self {
            database: Arc::clone(db_conn),
            password: Arc::clone(password),
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Option<User> {
        let statement = sql!(self.database.scheme, "SELECT * FROM users WHERE email = $1");

        let query = sqlx::query_as::<_, User>(&statement).bind(email);

        query.fetch_optional(self.database.get_pool()).await.unwrap_or(None)
    }

    pub async fn find_by_username(&self, username: &str) -> Option<User> {
        let statement = sql!(self.database.scheme, "SELECT * FROM users WHERE username = $1");

        let query = sqlx::query_as::<_, User>(&statement).bind(username);

        query.fetch_optional(self.database.get_pool()).await.unwrap_or(None)
    }

    pub async fn find(&self, id: i32) -> Option<User> {
        let statement = sql!(self.database.scheme, "SELECT * FROM users WHERE id = $1");

        let query = sqlx::query_as::<_, User>(&statement).bind(id);

        query.fetch_optional(self.database.get_pool()).await.unwrap_or(None)
    }

    pub async fn add<T: Into<UserCreateDto>>(&self, data: T) -> Result<User, ApiError> {
        let UserCreateDto { username, email, password } = data.into();

        let user_password = self.password.hash(&password).map_err(|_| UserError::InvalidPassword)?;

        let statement = sql!(self.database.scheme, "INSERT INTO users (username, email, password) VALUES ($1, $2, $3)");

        let query = sqlx::query(&statement).bind(&username).bind(&email).bind(&user_password);

        let affected = query.execute(self.database.get_pool()).await
            .map_err(DatabaseError::from)?
            .rows_affected();
        if affected > 0 {
            self.find_by_email(&email).await.ok_or(UserError::UserNotFound.into())
        } else {
            Err(UserError::UserCreateFail.into())
        }
    }

    pub async fn remove(&self, id: i32) -> Result<bool, ApiError> {
        let statement = sql!(self.database.scheme, "DELETE FROM users WHERE id = $1");

        let query = sqlx::query(&statement).bind(id);

        query.execute(self.database.get_pool()).await.map_err(DatabaseError::from)?;

        Ok(true)
    }
}

#[cfg(test)]
mod user_repository_tests {
    use std::sync::Arc;

    use crate::configs::database::Database;
    use crate::configs::password::{Argon2Hash, Password};
    use crate::configs::settings::Settings;

    use super::*;

    #[tokio::test]
    async fn test_crud_operations() {
        let settings = Arc::new(Settings::new().unwrap());
        let database = Arc::new(Database::new(&settings).unwrap());
        let password = Arc::new(Argon2Hash::new()) as Arc<dyn Password>;
        let repo = UserRepository::new(&password, &database);

        let mock_dto = UserCreateDto {
            username: "test_user".to_string(),
            email: "test_user@test_email.com".to_string(),
            password: "test_password".to_string(),
        };
        let user = repo.add(mock_dto).await;

        assert!(user.is_ok(), "Should create and find registered user.");

        let result = repo.remove(user.unwrap().id).await.unwrap();

        assert!(result, "Record should be remove.")
    }
}