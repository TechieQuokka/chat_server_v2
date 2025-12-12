//! Validated JSON extractor
//!
//! Extracts and validates JSON request bodies using the validator crate.

use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::response::ApiError;

/// Validated JSON extractor
///
/// Extracts a JSON body and validates it using the `validator` crate.
/// The inner type must implement both `Deserialize` and `Validate`.
#[derive(Debug, Clone)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Extract JSON
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|e| {
            match e {
                JsonRejection::JsonDataError(e) => ApiError::invalid_query(e.to_string()),
                JsonRejection::JsonSyntaxError(e) => ApiError::invalid_query(e.to_string()),
                JsonRejection::MissingJsonContentType(e) => ApiError::invalid_query(e.to_string()),
                JsonRejection::BytesRejection(e) => ApiError::invalid_query(e.to_string()),
                _ => ApiError::invalid_query("Invalid JSON body"),
            }
        })?;

        // Validate
        value.validate()?;

        Ok(ValidatedJson(value))
    }
}

/// Optional validated JSON extractor
///
/// Similar to ValidatedJson but returns Ok(None) for empty bodies.
#[derive(Debug, Clone)]
pub struct OptionalValidatedJson<T>(pub Option<T>);

#[async_trait]
impl<S, T> FromRequest<S> for OptionalValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Check if there's a content-length header with non-zero value
        let has_body = req
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            .is_some_and(|len| len > 0);

        if !has_body {
            return Ok(OptionalValidatedJson(None));
        }

        // Extract and validate JSON
        let ValidatedJson(value) = ValidatedJson::from_request(req, state).await?;
        Ok(OptionalValidatedJson(Some(value)))
    }
}
