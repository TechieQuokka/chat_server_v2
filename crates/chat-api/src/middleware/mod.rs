//! Middleware stack for the API server
//!
//! Provides logging, request ID generation, CORS, and other middleware.

use axum::{
    body::Body,
    http::{header, HeaderValue, Method, Request, StatusCode},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use std::time::Duration;

use crate::state::AppState;

/// Header name for request ID
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Apply middleware stack to the router
pub fn apply_middleware(router: Router<AppState>) -> Router<AppState> {
    router.layer(
        ServiceBuilder::new()
            // Request ID
            .layer(SetRequestIdLayer::new(
                header::HeaderName::from_static(REQUEST_ID_HEADER),
                MakeRequestUuid,
            ))
            .layer(PropagateRequestIdLayer::new(header::HeaderName::from_static(
                REQUEST_ID_HEADER,
            )))
            // Tracing
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<Body>| {
                        let request_id = request
                            .headers()
                            .get(REQUEST_ID_HEADER)
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("unknown");

                        tracing::info_span!(
                            "http_request",
                            method = %request.method(),
                            uri = %request.uri(),
                            request_id = %request_id,
                        )
                    })
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            // Timeout (returns 503 Service Unavailable on timeout)
            .layer(TimeoutLayer::with_status_code(
                StatusCode::SERVICE_UNAVAILABLE,
                Duration::from_secs(30),
            ))
            // CORS (permissive for development)
            .layer(create_cors_layer()),
    )
}

/// Create CORS layer
fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::HeaderName::from_static(REQUEST_ID_HEADER),
        ])
        .allow_origin(Any) // TODO: Configure from AppConfig in production
        .expose_headers([
            header::HeaderName::from_static(REQUEST_ID_HEADER),
            header::HeaderName::from_static("x-ratelimit-limit"),
            header::HeaderName::from_static("x-ratelimit-remaining"),
            header::HeaderName::from_static("x-ratelimit-reset"),
        ])
}

/// Create CORS layer with specific origin for production
#[allow(dead_code)]
fn create_cors_layer_with_origin(origin: &str) -> CorsLayer {
    CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::HeaderName::from_static(REQUEST_ID_HEADER),
        ])
        .allow_origin(origin.parse::<HeaderValue>().unwrap())
        .expose_headers([
            header::HeaderName::from_static(REQUEST_ID_HEADER),
            header::HeaderName::from_static("x-ratelimit-limit"),
            header::HeaderName::from_static("x-ratelimit-remaining"),
            header::HeaderName::from_static("x-ratelimit-reset"),
        ])
}
