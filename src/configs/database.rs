use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::any::{Any, AnyConnectOptions, AnyPoolOptions};
use sqlx::migrate::{MigrateDatabase, Migrator};
use sqlx::{AnyPool, ConnectOptions, Connection};

use crate::errors::DatabaseError;
use super::schema::SchemaManager;
use super::settings::Settings;

#[macro_export]
macro_rules! sql {
    ($scheme:expr, $query:expr) => {{
        match $scheme {
            crate::configs::DatabaseScheme::MYSQL => {
                let mut result = String::new();
                let mut chars = $query.chars().peekable();
                let mut in_single_quotes = false;

                while let Some(c) = chars.next() {
                    match c {
                        '\'' => {
                            in_single_quotes = !in_single_quotes;
                            result.push(c);
                        }
                        '$' if !in_single_quotes && chars.peek().map_or(false, |next| next.is_ascii_digit()) => {
                            // Consume the digits following the '$'
                            while chars.peek().map_or(false, |next| next.is_ascii_digit()) {
                                chars.next();
                            }
                            result.push('?');
                        }
                        _ => result.push(c),
                    }
                }
                result
            },
            _ => $query.to_string(),
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
    pub async fn new(settings: &Arc<Settings>, schema_manager: &SchemaManager) -> Result<Self, DatabaseError> {
        let db_url = settings.database.url.clone();
        let db_options = AnyConnectOptions::from_str(&db_url)?;
        let db_scheme = match db_options.database_url.scheme() {
            "postgres" => DatabaseScheme::POSTGRES,
            "mysql" => DatabaseScheme::MYSQL,
            _ => DatabaseScheme::SQLITE,
        };

        sqlx::any::install_default_drivers();

        match db_options.connect().await {
            Ok(try_conn) => try_conn.close().await?,
            Err(_) => Any::create_database(&db_url).await?,
        }

        let pool = AnyPoolOptions::new().connect_with(db_options).await?;

        if settings.database.clean_start {
            let dispose_statements = schema_manager.dispose_schema();
            let create_statements = schema_manager.create_schema(&db_scheme);
            let statements = [&dispose_statements[..], &create_statements[..]].concat();

            sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
                .execute(&pool)
                .await?;

            for statement in statements.iter() {
                sqlx::query(&statement)
                    .execute(&pool)
                    .await?;
            }

            tracing::warn!("perform a clean boot: clean and recreate schema");
        }

        if let Some(migration_path) = settings.database.migration_path.clone() {
            let mut pool_connection = pool.acquire().await?;
            let migrator = Migrator::new(Path::new(&migration_path)).await?;
            migrator.run(&mut pool_connection).await?;

            tracing::info!("database migration success");
        }

        Ok(Self {
            scheme: db_scheme,
            pool,
        })
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
