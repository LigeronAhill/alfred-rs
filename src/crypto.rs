use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::{AppError, AppResult};

pub fn hash_password(password: &str) -> AppResult<String> {
    let password = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let res = Argon2::default()
        .hash_password(password, &salt)
        .map_err(|e| AppError::CryptoError(e.to_string()))?
        .to_string();
    Ok(res)
}

pub fn verify_password(hash: &str, password: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| AppError::CryptoError(e.to_string()))?;
    let res = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_success_verify() {
        let pwd = "somePassword".to_string();
        let hashed = hash_password(&pwd).unwrap();

        let result = verify_password(&hashed, &pwd).unwrap();
        assert!(result);
    }
    #[test]
    fn test_success_failed() {
        let pwd = "somePassword".to_string();
        let hashed = hash_password(&pwd).unwrap();

        let result = verify_password(&hashed, "wrongPass").unwrap();
        assert!(!result);
    }
}
