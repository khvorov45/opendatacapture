const SALT_LENGTH: usize = 30;
const AUTH_TOKEN_LENGTH: usize = 30;

/// Generate an auth token
pub fn gen_auth_token() -> String {
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
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Outcome {
    /// Contains authorization token
    Authorized(String),
    Unauthorized,
    UserNotFound,
}
