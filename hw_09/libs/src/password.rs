use pbkdf2::{
    password_hash::{Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand_core::OsRng;

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);

    Ok(Pbkdf2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(&password_hash)?;

    Ok(Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
