pub fn hash(password: &str) -> Result<String, argon2::Error> {
    argon2::hash_encoded(
        password.as_bytes(),
        super::gen_rand_string().as_bytes(),
        &argon2::Config::default(),
    )
}
