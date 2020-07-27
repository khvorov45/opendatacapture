use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::db;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Serialize)]
pub struct EmailPassword {
    email: String,
    password: String,
}

/// Authenticates email and password combination
pub fn authenticate_email_password(
    _admindb: Arc<db::DB>,
    _cred: EmailPassword,
) -> Result<bool> {
    Ok(false)
}

pub mod error {
    /// Handler errors
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        /// Database errors
        #[error(transparent)]
        DB(#[from] super::db::Error),
    }
}
