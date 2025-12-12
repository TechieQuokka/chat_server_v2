//! Password hashing and verification utilities
//!
//! Uses Argon2id for secure password hashing (OWASP recommended).

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::AppError;

/// Hash a password using Argon2id
///
/// # Errors
/// Returns an error if hashing fails
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hashing failed: {e}")))
}

/// Verify a password against a hash
///
/// # Errors
/// Returns an error if verification fails or the hash is invalid
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid password hash format: {e}")))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Password service for dependency injection
#[derive(Debug, Clone, Default)]
pub struct PasswordService;

impl PasswordService {
    /// Create a new password service
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Hash a password
    ///
    /// # Errors
    /// Returns an error if hashing fails
    pub fn hash(&self, password: &str) -> Result<String, AppError> {
        hash_password(password)
    }

    /// Verify a password against a hash
    ///
    /// # Errors
    /// Returns an error if verification fails
    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        verify_password(password, hash)
    }

    /// Verify a password and return an error if invalid
    ///
    /// # Errors
    /// Returns `AppError::InvalidCredentials` if the password doesn't match
    pub fn verify_or_error(&self, password: &str, hash: &str) -> Result<(), AppError> {
        if self.verify(password, hash)? {
            Ok(())
        } else {
            Err(AppError::InvalidCredentials)
        }
    }
}

/// Validate password strength
///
/// Returns `Ok(())` if the password meets requirements:
/// - At least 8 characters
/// - Contains at least one uppercase letter
/// - Contains at least one lowercase letter
/// - Contains at least one digit
///
/// # Errors
/// Returns a validation error if the password doesn't meet requirements
pub fn validate_password_strength(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if !password.chars().any(char::is_uppercase) {
        return Err(AppError::Validation(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if !password.chars().any(char::is_lowercase) {
        return Err(AppError::Validation(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation(
            "Password must contain at least one digit".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        // Hash should start with argon2 identifier
        assert!(hash.starts_with("$argon2"));
        // Hash should be different each time (different salt)
        let hash2 = hash_password(password).unwrap();
        assert_ne!(hash, hash2);
    }

    #[test]
    fn test_verify_password_success() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_failure() {
        let password = "SecurePassword123!";
        let wrong_password = "WrongPassword123!";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password(wrong_password, &hash).unwrap());
    }

    #[test]
    fn test_password_service() {
        let service = PasswordService::new();
        let password = "SecurePassword123!";

        let hash = service.hash(password).unwrap();
        assert!(service.verify(password, &hash).unwrap());
        assert!(!service.verify("wrong", &hash).unwrap());
    }

    #[test]
    fn test_verify_or_error_success() {
        let service = PasswordService::new();
        let password = "SecurePassword123!";
        let hash = service.hash(password).unwrap();

        assert!(service.verify_or_error(password, &hash).is_ok());
    }

    #[test]
    fn test_verify_or_error_failure() {
        let service = PasswordService::new();
        let password = "SecurePassword123!";
        let hash = service.hash(password).unwrap();

        let result = service.verify_or_error("wrong", &hash);
        assert!(matches!(result, Err(AppError::InvalidCredentials)));
    }

    #[test]
    fn test_validate_password_strength_valid() {
        assert!(validate_password_strength("SecurePass1").is_ok());
        assert!(validate_password_strength("Abcdefg1").is_ok());
        assert!(validate_password_strength("MyP@ssw0rd!").is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let result = validate_password_strength("Short1");
        assert!(result.is_err());
        if let Err(AppError::Validation(msg)) = result {
            assert!(msg.contains("8 characters"));
        }
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let result = validate_password_strength("lowercase123");
        assert!(result.is_err());
        if let Err(AppError::Validation(msg)) = result {
            assert!(msg.contains("uppercase"));
        }
    }

    #[test]
    fn test_validate_password_strength_no_lowercase() {
        let result = validate_password_strength("UPPERCASE123");
        assert!(result.is_err());
        if let Err(AppError::Validation(msg)) = result {
            assert!(msg.contains("lowercase"));
        }
    }

    #[test]
    fn test_validate_password_strength_no_digit() {
        let result = validate_password_strength("NoDigitsHere");
        assert!(result.is_err());
        if let Err(AppError::Validation(msg)) = result {
            assert!(msg.contains("digit"));
        }
    }
}
