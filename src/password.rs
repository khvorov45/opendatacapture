pub fn hash(password: &str) -> Result<String, argon2::Error> {
    argon2::hash_encoded(
        password.as_bytes(),
        gen_rand_string().as_bytes(),
        &argon2::Config::default(),
    )
}

/// Generates a random string
fn gen_rand_string() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(30)
        .collect()
}
