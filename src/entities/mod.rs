mod user;

pub use user::{User, UserTable};

use crate::configs::DatabaseScheme;

pub trait Table {
    fn name(&self) -> &'static str;

    fn create(&self, scheme: &DatabaseScheme) -> String;

    fn dispose(&self) -> String;

    fn dependencies(&self) -> Vec<&'static str>;
}