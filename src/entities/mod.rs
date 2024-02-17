use async_trait::async_trait;
use sqlx::{AnyPool, Error};

use crate::configs::database::DatabaseScheme;

pub mod user;

#[async_trait]
pub trait Builder {
    async fn build(db_scheme: &DatabaseScheme, pool: &AnyPool) -> Result<(), Error>;
}