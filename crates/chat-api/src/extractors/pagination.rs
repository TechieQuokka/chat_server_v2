//! Pagination extractor
//!
//! Extracts cursor-based pagination parameters from query strings.

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use chat_core::Snowflake;
use serde::Deserialize;

use crate::response::ApiError;

/// Default page size
const DEFAULT_LIMIT: i32 = 50;
/// Maximum page size
const MAX_LIMIT: i32 = 100;

/// Raw pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    /// Get items before this ID
    #[serde(default)]
    pub before: Option<String>,
    /// Get items after this ID
    #[serde(default)]
    pub after: Option<String>,
    /// Maximum number of items to return
    #[serde(default)]
    pub limit: Option<i32>,
}

/// Validated pagination parameters
#[derive(Debug, Clone)]
pub struct Pagination {
    /// Get items before this ID
    pub before: Option<Snowflake>,
    /// Get items after this ID
    pub after: Option<Snowflake>,
    /// Maximum number of items to return (validated to 1-100)
    pub limit: i32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            before: None,
            after: None,
            limit: DEFAULT_LIMIT,
        }
    }
}

impl Pagination {
    /// Create pagination with only a limit
    pub fn with_limit(limit: i32) -> Self {
        Self {
            before: None,
            after: None,
            limit: limit.clamp(1, MAX_LIMIT),
        }
    }

    /// Check if we should fetch forward (after cursor)
    pub fn is_forward(&self) -> bool {
        self.after.is_some() || self.before.is_none()
    }
}

impl TryFrom<PaginationParams> for Pagination {
    type Error = ApiError;

    fn try_from(params: PaginationParams) -> Result<Self, Self::Error> {
        // Parse before cursor
        let before = params
            .before
            .map(|s| {
                s.parse::<Snowflake>()
                    .map_err(|_| ApiError::invalid_query("Invalid 'before' cursor format"))
            })
            .transpose()?;

        // Parse after cursor
        let after = params
            .after
            .map(|s| {
                s.parse::<Snowflake>()
                    .map_err(|_| ApiError::invalid_query("Invalid 'after' cursor format"))
            })
            .transpose()?;

        // Validate and clamp limit
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);

        Ok(Pagination {
            before,
            after,
            limit,
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<PaginationParams>::from_request_parts(parts, state)
            .await
            .map_err(|e| ApiError::invalid_query(e.to_string()))?;

        Pagination::try_from(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination() {
        let pagination = Pagination::default();
        assert_eq!(pagination.limit, DEFAULT_LIMIT);
        assert!(pagination.before.is_none());
        assert!(pagination.after.is_none());
    }

    #[test]
    fn test_limit_clamping() {
        let pagination = Pagination::with_limit(200);
        assert_eq!(pagination.limit, MAX_LIMIT);

        let pagination = Pagination::with_limit(0);
        assert_eq!(pagination.limit, 1);
    }

    #[test]
    fn test_pagination_from_params() {
        let params = PaginationParams {
            before: Some("123456789".to_string()),
            after: None,
            limit: Some(25),
        };

        let pagination = Pagination::try_from(params).unwrap();
        assert!(pagination.before.is_some());
        assert!(pagination.after.is_none());
        assert_eq!(pagination.limit, 25);
    }
}
