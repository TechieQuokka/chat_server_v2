//! JWT utilities for authentication
//!
//! Provides token encoding, decoding, and validation using the `jsonwebtoken` crate.

use chat_core::Snowflake;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Token type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Token type (access or refresh)
    pub token_type: TokenType,
    /// Optional session ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl Claims {
    /// Get the user ID as a Snowflake
    ///
    /// # Errors
    /// Returns an error if the subject cannot be parsed as a Snowflake
    pub fn user_id(&self) -> Result<Snowflake, AppError> {
        self.sub
            .parse::<i64>()
            .map(Snowflake::new)
            .map_err(|_| AppError::InvalidToken)
    }

    /// Check if the token is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Check if this is an access token
    #[must_use]
    pub fn is_access_token(&self) -> bool {
        self.token_type == TokenType::Access
    }

    /// Check if this is a refresh token
    #[must_use]
    pub fn is_refresh_token(&self) -> bool {
        self.token_type == TokenType::Refresh
    }
}

/// Token pair containing access and refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// JWT service for encoding and decoding tokens
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl JwtService {
    /// Create a new JWT service with the given secret and expiry times
    #[must_use]
    pub fn new(secret: &str, access_token_expiry: i64, refresh_token_expiry: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    /// Generate a token pair for a user
    ///
    /// # Errors
    /// Returns an error if token encoding fails
    pub fn generate_token_pair(&self, user_id: Snowflake) -> Result<TokenPair, AppError> {
        self.generate_token_pair_with_session(user_id, None)
    }

    /// Generate a token pair for a user with a session ID
    ///
    /// # Errors
    /// Returns an error if token encoding fails
    pub fn generate_token_pair_with_session(
        &self,
        user_id: Snowflake,
        session_id: Option<String>,
    ) -> Result<TokenPair, AppError> {
        let access_token = self.encode_token(user_id, TokenType::Access, session_id.clone())?;
        let refresh_token = self.encode_token(user_id, TokenType::Refresh, session_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry,
        })
    }

    /// Encode a JWT token
    fn encode_token(
        &self,
        user_id: Snowflake,
        token_type: TokenType,
        session_id: Option<String>,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let expiry = match token_type {
            TokenType::Access => self.access_token_expiry,
            TokenType::Refresh => self.refresh_token_expiry,
        };

        let claims = Claims {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(expiry)).timestamp(),
            token_type,
            session_id,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to encode JWT")))
    }

    /// Decode and validate a JWT token
    ///
    /// # Errors
    /// Returns an error if the token is invalid or expired
    pub fn decode_token(&self, token: &str) -> Result<Claims, AppError> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
                _ => AppError::InvalidToken,
            }
        })?;

        Ok(token_data.claims)
    }

    /// Validate an access token and return the claims
    ///
    /// # Errors
    /// Returns an error if the token is invalid, expired, or not an access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, AppError> {
        let claims = self.decode_token(token)?;

        if !claims.is_access_token() {
            return Err(AppError::InvalidToken);
        }

        Ok(claims)
    }

    /// Validate a refresh token and return the claims
    ///
    /// # Errors
    /// Returns an error if the token is invalid, expired, or not a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, AppError> {
        let claims = self.decode_token(token)?;

        if !claims.is_refresh_token() {
            return Err(AppError::InvalidToken);
        }

        Ok(claims)
    }

    /// Refresh tokens using a valid refresh token
    ///
    /// # Errors
    /// Returns an error if the refresh token is invalid or expired
    pub fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair, AppError> {
        let claims = self.validate_refresh_token(refresh_token)?;
        let user_id = claims.user_id()?;

        self.generate_token_pair_with_session(user_id, claims.session_id)
    }
}

impl std::fmt::Debug for JwtService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtService")
            .field("access_token_expiry", &self.access_token_expiry)
            .field("refresh_token_expiry", &self.refresh_token_expiry)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> JwtService {
        JwtService::new("test-secret-key-that-is-long-enough", 900, 604800)
    }

    #[test]
    fn test_generate_token_pair() {
        let service = create_test_service();
        let user_id = Snowflake::new(12345);

        let pair = service.generate_token_pair(user_id).unwrap();

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
        assert_eq!(pair.expires_in, 900);
    }

    #[test]
    fn test_decode_access_token() {
        let service = create_test_service();
        let user_id = Snowflake::new(12345);

        let pair = service.generate_token_pair(user_id).unwrap();
        let claims = service.decode_token(&pair.access_token).unwrap();

        assert_eq!(claims.sub, "12345");
        assert!(claims.is_access_token());
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_decode_refresh_token() {
        let service = create_test_service();
        let user_id = Snowflake::new(12345);

        let pair = service.generate_token_pair(user_id).unwrap();
        let claims = service.decode_token(&pair.refresh_token).unwrap();

        assert_eq!(claims.sub, "12345");
        assert!(claims.is_refresh_token());
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_validate_access_token() {
        let service = create_test_service();
        let user_id = Snowflake::new(12345);

        let pair = service.generate_token_pair(user_id).unwrap();

        // Should succeed with access token
        let claims = service.validate_access_token(&pair.access_token).unwrap();
        assert_eq!(claims.user_id().unwrap(), user_id);

        // Should fail with refresh token
        let result = service.validate_access_token(&pair.refresh_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_refresh_token() {
        let service = create_test_service();
        let user_id = Snowflake::new(12345);

        let pair = service.generate_token_pair(user_id).unwrap();

        // Should succeed with refresh token
        let claims = service.validate_refresh_token(&pair.refresh_token).unwrap();
        assert_eq!(claims.user_id().unwrap(), user_id);

        // Should fail with access token
        let result = service.validate_refresh_token(&pair.access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_tokens() {
        let service = create_test_service();
        let user_id = Snowflake::new(12345);

        let pair1 = service.generate_token_pair(user_id).unwrap();
        let pair2 = service.refresh_tokens(&pair1.refresh_token).unwrap();

        // New tokens should be valid (they may be identical if generated in same second)
        let claims = service.validate_access_token(&pair2.access_token).unwrap();
        assert_eq!(claims.user_id().unwrap(), user_id);

        let claims = service.validate_refresh_token(&pair2.refresh_token).unwrap();
        assert_eq!(claims.user_id().unwrap(), user_id);
    }

    #[test]
    fn test_invalid_token() {
        let service = create_test_service();

        let result = service.decode_token("invalid.token.here");
        assert!(matches!(result, Err(AppError::InvalidToken)));
    }

    #[test]
    fn test_token_with_session_id() {
        let service = create_test_service();
        let user_id = Snowflake::new(12345);
        let session_id = "session-123".to_string();

        let pair = service
            .generate_token_pair_with_session(user_id, Some(session_id.clone()))
            .unwrap();

        let claims = service.decode_token(&pair.access_token).unwrap();
        assert_eq!(claims.session_id, Some(session_id));
    }

    #[test]
    fn test_claims_user_id() {
        let claims = Claims {
            sub: "12345".to_string(),
            iat: 0,
            exp: i64::MAX,
            token_type: TokenType::Access,
            session_id: None,
        };

        let user_id = claims.user_id().unwrap();
        assert_eq!(user_id, Snowflake::new(12345));
    }
}
