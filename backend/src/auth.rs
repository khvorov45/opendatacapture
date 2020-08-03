const SALT_LENGTH: usize = 30;
const AUTH_TOKEN_LENGTH: usize = 30;
const N_SUBSECS: u16 = 6;

/// Generate an auth token
fn gen_auth_token() -> String {
    gen_rand_string(AUTH_TOKEN_LENGTH)
}

/// Hash a string
pub fn hash(password: &str) -> Result<String, argon2::Error> {
    argon2::hash_encoded(
        password.as_bytes(),
        gen_rand_string(SALT_LENGTH).as_bytes(),
        &argon2::Config::default(),
    )
}

/// Generates a random string
fn gen_rand_string(len: usize) -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .collect()
}

/// Authentication outcome
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Contains the auth token
    Ok(Token),
    Wrong,
    IdNotFound,
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
}
