use super::db;

/// Credentials
pub enum Credentials {
    /// Email and password - regular GUI login
    EmailPassword(String, String),
    /// ID and token - 'remember me' type login
    IdToken(i32, String),
}

/// Authenticates the given credentials using the given database
pub fn authenticate(
    credentials: Credentials,
    admindb: &db::DB,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Delegate to the appropriate function
    match credentials {
        Credentials::EmailPassword(email, password) => {
            authenticate_email_password(email, password, admindb)
        }
        Credentials::IdToken(id, token) => {
            authenticate_id_token(id, token, admindb)
        }
    }
}

/// Authenticates email and password combination
fn authenticate_email_password(
    email: String,
    password: String,
    admindb: &db::DB,
) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(false)
}

/// Authenticates id and token combination
fn authenticate_id_token(
    id: i32,
    token: String,
    admindb: &db::DB,
) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(false)
}
