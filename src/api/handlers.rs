use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{db, error};

#[derive(Deserialize, Serialize)]
pub struct EmailPassword {
    email: String,
    password: String,
}

/// Authenticates email and password combination
pub fn authenticate_email_password(
    _admindb: Arc<db::DB>,
    _cred: EmailPassword,
) -> Result<bool, error::APIError> {
    Ok(false)
}
