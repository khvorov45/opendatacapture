use crate::{error::Unauthorized, Error, Result};

const SALT_LENGTH: usize = 30;
const AUTH_TOKEN_LENGTH: usize = 30;
const N_SUBSECS: u16 = 6; // Postgres precision
pub const AUTH_TOKEN_HOURS_TO_LIVE: i64 = 24;

/// Generate an auth token
fn gen_auth_token() -> String {
    gen_rand_string(AUTH_TOKEN_LENGTH)
}

/// Hash a string
pub fn hash(password: &str) -> Result<String> {
    let hash = argon2::hash_encoded(
        password.as_bytes(),
        gen_rand_string(SALT_LENGTH).as_bytes(),
        &argon2::Config::default(),
    )?;
    Ok(hash)
}

/// Hash a string but quickly
pub fn hash_fast(token: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(token.as_bytes());
    let hash_result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&hash_result);
    hex::encode(&out)
}

/// Generates a random string
fn gen_rand_string(len: usize) -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .collect()
}

/// Parses the bearer header
pub fn parse_bearer_header(raw: &str) -> Result<&str> {
    let header: Vec<&str> = raw.splitn(2, ' ').collect();
    if header[0] != "Bearer" {
        return Err(Error::Unauthorized(Unauthorized::WrongAuthType(
            header[0].to_string(),
        )));
    }
    Ok(header[1])
}

/// Authentication outcome for email/password
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub enum PasswordOutcome {
    /// Contains the auth token
    Ok(Token),
    WrongPassword,
    EmailNotFound,
}

/// Authentication outcome for id/token
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub enum TokenOutcome {
    Ok(Access),
    TokenTooOld,
    TokenNotFound,
}

/// Auth token
#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, sqlx::FromRow,
)]
pub struct Token {
    user: i32,
    token: String,
    created: chrono::DateTime<chrono::Utc>,
}

impl Token {
    pub fn new(user: i32) -> Self {
        use chrono::SubsecRound;
        Self {
            user,
            token: gen_auth_token(),
            created: chrono::Utc::now().round_subsecs(N_SUBSECS),
        }
    }
    pub fn user(&self) -> i32 {
        self.user
    }
    pub fn token(&self) -> &str {
        self.token.as_str()
    }
    pub fn created(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created
    }
    pub fn age_hours(&self) -> i64 {
        chrono::Utc::now()
            .signed_duration_since(self.created)
            .num_hours()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct IdToken {
    pub id: i32,
    pub token: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct EmailPassword {
    pub email: String,
    pub password: String,
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    sqlx::Type,
)]
#[sqlx(rename = "odc_user_access")]
// Need to modify the postgres type declaration in `admin` on any changes
pub enum Access {
    User,
    Admin,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_access() {
        assert!(Access::Admin > Access::User);
        assert_eq!(Access::Admin, Access::Admin);
    }
    #[test]
    fn test_parse_header() {
        assert_eq!(parse_bearer_header("Bearer 123abc").unwrap(), "123abc");
        assert_eq!(
            parse_bearer_header("Bearer 12 3; \\abc").unwrap(),
            "12 3; \\abc"
        );
        assert!(matches!(
            parse_bearer_header("Basic u:p").unwrap_err(),
            Error::Unauthorized(Unauthorized::WrongAuthType(b)) if b == "Basic"
        ));
    }
    #[test]
    fn test_token() {
        use chrono::prelude::*;
        let mut tok = Token::new(1);
        assert!(tok.age_hours() < 1);
        tok.created = chrono::Utc.ymd(2000, 1, 1).and_hms(0, 0, 0);
        assert!(tok.age_hours() > 1000);
    }
}
