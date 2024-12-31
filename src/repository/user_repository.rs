use std::sync::Arc;

use crate::configs::{Database, Password};
use crate::entities::User;
use crate::errors::{ApiError, DatabaseError, UserError};
use crate::payload::{UserCreateDao, UserUpdateDao};
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

        query.fetch_optional(&self.database.pool).await.unwrap_or(None)
    }

    pub async fn find_by_username(&self, username: &str) -> Option<User> {
        let statement = sql!(self.database.scheme, "SELECT * FROM users WHERE username = $1");

        let query = sqlx::query_as::<_, User>(&statement).bind(username);

        query.fetch_optional(&self.database.pool).await.unwrap_or(None)
    }

    pub async fn find(&self, id: i32) -> Option<User> {
        let statement = sql!(self.database.scheme, "SELECT * FROM users WHERE id = $1");

        let query = sqlx::query_as::<_, User>(&statement).bind(id);

        query.fetch_optional(&self.database.pool).await.unwrap_or(None)
    }

    pub async fn add<T: Into<UserCreateDao>>(&self, data: T) -> Result<User, ApiError> {
        let UserCreateDao { username, email, password } = data.into();

        let user_password = self.password.hash(&password)?;

        let statement = sql!(self.database.scheme, "INSERT INTO users (username, email, password) VALUES ($1, $2, $3)");

        let query = sqlx::query(&statement).bind(&username).bind(&email).bind(&user_password);

        if query.execute(&self.database.pool).await.map_err(DatabaseError::from)?.rows_affected() > 0 {
            self.find_by_email(&email).await.ok_or(UserError::UserNotFound.into())
        } else {
            Err(UserError::UserCreateFail.into())
        }
    }

    pub async fn update<T: Into<UserUpdateDao>>(&self, data: T) -> Result<User, ApiError> {
        let UserUpdateDao { id, username, email, password } = data.into();

        let mut updates = Vec::new();
        let mut bindings = Vec::new();

        if let Some(username_value) = username {
            updates.push("username = $".to_owned() + &(bindings.len() + 1).to_string());
            bindings.push(username_value);
        }

        if let Some(email_value) = email {
            updates.push("email = $".to_owned() + &(bindings.len() + 1).to_string());
            bindings.push(email_value);
        }

        if let Some(password_value) = password {
            let hashed_password = self.password.hash(&password_value)?;
            updates.push("password = $".to_owned() + &(bindings.len() + 1).to_string());
            bindings.push(hashed_password);
        }

        if updates.len() > 0 {
            bindings.push(id.to_string());

            let statement = format!("UPDATE users SET {} WHERE id = ${}", updates.join(", "), bindings.len() + 1);

            let mut query = sqlx::query(&statement);
            for value in bindings {
                query = query.bind(value);
            }

            query.execute(&self.database.pool).await.map_err(DatabaseError::from)?;

            self.find(id).await.ok_or(ApiError::from(UserError::UserNotFound))
        } else {
            Err(UserError::UserUpdateFail)?
        }
    }

    pub async fn remove(&self, id: i32) -> Result<bool, ApiError> {
        let statement = sql!(self.database.scheme, "DELETE FROM users WHERE id = $1");

        let query = sqlx::query(&statement).bind(id);

        query.execute(&self.database.pool).await.map_err(DatabaseError::from)?;

        Ok(true)
    }
}

#[cfg(test)]
mod user_repository_tests {
    use std::sync::Arc;

    use super::*;
    use crate::configs::{Argon2Hash, Database, Password, SchemaManager, Settings};

    #[tokio::test]
    async fn test_crud_operations() {
        let settings = Arc::new(Settings::new().unwrap());
        let database = Arc::new(Database::new(&settings, &SchemaManager::default()).await.unwrap());
        let password = Arc::new(Argon2Hash::new()) as Arc<dyn Password>;
        let repo = UserRepository::new(&password, &database);

        let mock_dao = UserCreateDao {
            username: "test_user".to_string(),
            email: "test_user@test_email.com".to_string(),
            password: "test_password".to_string(),
        };
        let user = repo.add(mock_dao).await;

        assert!(user.is_ok(), "Should create and find registered user.");

        let result = repo.remove(user.unwrap().id).await.unwrap();

        assert!(result, "Record should be remove.");
    }
}
