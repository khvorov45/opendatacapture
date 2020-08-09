use crate::{Error, Result};

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

/// Generates a random string
fn gen_rand_string(len: usize) -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .collect()
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
    Ok,
    TokenTooOld,
    TokenNotFound,
}

/// Auth token
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
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
    pub fn user(&self) -> &i32 {
        &self.user
    }
    pub fn token(&self) -> &String {
        &self.token
    }
    pub fn created(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created
    }
    pub fn from_row(row: &tokio_postgres::Row) -> Self {
        Self {
            user: row.get("user"),
            token: row.get("token"),
            created: row.get("created"),
        }
    }
    pub fn age_hours(&self) -> i64 {
        self.created
            .signed_duration_since(chrono::Utc::now())
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

/// Parses the authentication header
pub fn parse_basic_header(header_content: &str) -> Result<IdToken> {
    let header: Vec<&str> = header_content.splitn(2, ' ').collect();
    if header[0] != "Basic" {
        return Err(Error::WrongAuthType(header[0].to_string()));
    }
    let header_decoded = base64::decode(header[1])?;
    let header_parsed: Vec<&str> = std::str::from_utf8(&header_decoded)?
        .splitn(2, ':')
        .collect();
    Ok(IdToken {
        id: header_parsed[0].parse()?,
        token: header_parsed[1].to_string(),
    })
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    strum_macros::Display,
    strum_macros::EnumString,
)]
pub enum Access {
    User,
    Admin,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_parse_basic_header_tok(tok: &str) {
        let header_raw =
            format!("Basic {}", base64::encode(format!("1:{}", tok)));
        let header_parsed = parse_basic_header(header_raw.as_str()).unwrap();
        assert_eq!(
            header_parsed,
            IdToken {
                id: 1,
                token: tok.to_string()
            }
        )
    }
    #[test]
    fn test_parse_basic_header() {
        test_parse_basic_header_tok("pass123");
        test_parse_basic_header_tok("123: sad gg")
    }
    #[test]
    fn test_access() {
        assert!(Access::Admin > Access::User);
        assert_eq!(Access::Admin, Access::Admin);
    }
}
