use std::str::FromStr;
use std::sync::Arc;

use sqlx::{AnyPool, ConnectOptions, Error, Executor};
use sqlx::any::{AnyConnectOptions, AnyPoolOptions};

use crate::configs::settings::Settings;
use crate::entities::Builder;
use crate::entities::user::UserBuilder;

#[macro_export]
macro_rules! sql {
    ($scheme:expr, $query:expr) => {{
        match $scheme {
            DatabaseScheme::MYSQL => {
                let mut result = String::new();
                let mut chars = $query.chars().peekable();
                let mut in_single_quotes = false;

                while let Some(c) = chars.next() {
                    match c {
                        '\'' => {
                            in_single_quotes = !in_single_quotes;
                            result.push(c);
                        }
                        '$' if !in_single_quotes && chars.peek().map_or(false, |next| next.is_digit(10)) => {
                            // Consume the digits following the '$'
                            while chars.peek().map_or(false, |next| next.is_digit(10)) {
                                chars.next();
                            }
                            result.push('?');
                        }
                        _ => result.push(c),
                    }
                }
                result
            },
            DatabaseScheme::POSTGRES | DatabaseScheme::SQLITE => $query.to_string(),
        }
    }};
}

#[derive(Debug, Clone)]
pub enum DatabaseScheme {
    POSTGRES,
    SQLITE,
    MYSQL,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub scheme: DatabaseScheme,
    pub pool: AnyPool,
}

impl Database {
    pub async fn new(settings: &Arc<Settings>) -> Result<Self, Error> {
        let db_url = &settings.database.url;
        let db_options = AnyConnectOptions::from_str(db_url)?;
        let db_scheme = match db_options.database_url.scheme() {
            "postgres" => DatabaseScheme::POSTGRES,
            "mysql" => DatabaseScheme::MYSQL,
            _ => DatabaseScheme::SQLITE,
        };

        sqlx::any::install_default_drivers();

        let pool = match AnyPoolOptions::new().connect_with(db_options).await {
            Ok(pool) => pool,
            Err(Error::Database(_)) => {
                let pool = Self::create_database(db_url).await?;
                Self::create_tables(&db_scheme, &pool).await?;

                pool
            }
            Err(err) => Err(err)?,
        };

        Ok(Self {
            scheme: db_scheme,
            pool,
        })
    }

    pub fn get_pool(&self) -> &AnyPool {
        &self.pool
    }

    async fn create_database(url: &str) -> Result<AnyPool, Error> {
        let (base_url, path) = url.split_at(url.rfind('/').unwrap());

        let mut db_conn = AnyConnectOptions::from_str(base_url)?.connect().await?;

        let statement = format!("CREATE DATABASE {}", &path[1..]);

        db_conn.execute(&statement[..]).await?;

        let db_options = AnyConnectOptions::from_str(url)?;

        AnyPoolOptions::new().connect_with(db_options).await
    }

    async fn create_tables(scheme: &DatabaseScheme, pool: &AnyPool) -> Result<(), Error> {
        UserBuilder::build(&scheme, pool).await?;

        Ok(())
    }
}

#[cfg(test)]
mod database_tests {
    use super::*;

    #[test]
    fn test_replace_under_not_postgres() {
        let query = "SELECT * FROM users WHERE id = $1";
        let expected = "SELECT * FROM users WHERE id = ?";
        let actual = sql!(DatabaseScheme::MYSQL, query);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_not_replace_under_postgres() {
        let query = "SELECT * FROM users WHERE id = $1";
        let expected = "SELECT * FROM users WHERE id = $1";
        let actual = sql!(DatabaseScheme::POSTGRES, query);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_multiple_placeholders() {
        let query = "INSERT INTO users (name, age, salary) VALUES ($1, $2, $3)";
        let expected = "INSERT INTO users (name, age, salary) VALUES (?, ?, ?)";
        let actual = sql!(DatabaseScheme::MYSQL, query);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ignore_text_within_quotes() {
        let query = "SELECT * FROM users WHERE name = '$1' AND email = 'email$2@example.com'";
        let expected = "SELECT * FROM users WHERE name = '$1' AND email = 'email$2@example.com'";
        let actual = sql!(DatabaseScheme::MYSQL, query);
        assert_eq!(actual, expected);
    }
}