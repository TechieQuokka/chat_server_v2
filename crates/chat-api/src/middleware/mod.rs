//! Middleware stack for the API server
//!
//! Provides logging, request ID generation, CORS, rate limiting, and other middleware.

use axum::{
    body::Body,
    http::{header, HeaderValue, Method, Request, StatusCode},
    Router,
};
use chat_common::{CorsConfig, RateLimitConfig};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::GlobalKeyExtractor, GovernorLayer};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

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
            )),
    )
}

/// Apply middleware stack with rate limiting and configured CORS
pub fn apply_middleware_with_config(
    router: Router<AppState>,
    rate_limit_config: &RateLimitConfig,
    cors_config: &CorsConfig,
    is_production: bool,
) -> Router<AppState> {
    // Create rate limiter configuration with GlobalKeyExtractor
    // This applies rate limit globally (not per-IP) - suitable for local testing
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(rate_limit_config.requests_per_second.into())
            .burst_size(rate_limit_config.burst)
            .key_extractor(GlobalKeyExtractor)
            .finish()
            .expect("Failed to create rate limiter configuration"),
    );

    // Apply layers in order (note: layers are applied in reverse order in tower)
    // So we want: Request -> RateLimit -> RequestID -> Trace -> Timeout -> CORS -> Handler
    // Which means we add them in this order: CORS, Timeout, Trace, RequestID, RateLimit
    router
        // CORS (innermost - applied last to outgoing responses)
        .layer(create_cors_layer_from_config(cors_config, is_production))
        // Timeout (returns 503 Service Unavailable on timeout)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::SERVICE_UNAVAILABLE,
            Duration::from_secs(30),
        ))
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
        // Request ID propagation
        .layer(PropagateRequestIdLayer::new(header::HeaderName::from_static(
            REQUEST_ID_HEADER,
        )))
        // Request ID generation
        .layer(SetRequestIdLayer::new(
            header::HeaderName::from_static(REQUEST_ID_HEADER),
            MakeRequestUuid,
        ))
        // Rate limiting (outermost - applied first to incoming requests)
        .layer(GovernorLayer {
            config: governor_conf,
        })
}

/// Create CORS layer from configuration
fn create_cors_layer_from_config(config: &CorsConfig, is_production: bool) -> CorsLayer {
    let base_layer = CorsLayer::new()
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
        .expose_headers([
            header::HeaderName::from_static(REQUEST_ID_HEADER),
            header::HeaderName::from_static("x-ratelimit-limit"),
            header::HeaderName::from_static("x-ratelimit-remaining"),
            header::HeaderName::from_static("x-ratelimit-reset"),
        ]);

    // In production, only allow configured origins
    // In development, allow any origin if no origins are configured
    if is_production || !config.allowed_origins.is_empty() {
        if config.allowed_origins.is_empty() {
            tracing::warn!(
                "CORS: No allowed origins configured in production mode. \
                 Requests from browsers will be blocked."
            );
            // Return a restrictive CORS layer that blocks all cross-origin requests
            base_layer.allow_origin(AllowOrigin::list(Vec::<HeaderValue>::new()))
        } else {
            let origins: Vec<HeaderValue> = config
                .allowed_origins
                .iter()
                .filter_map(|origin| {
                    origin.parse::<HeaderValue>().ok().or_else(|| {
                        tracing::warn!("Invalid CORS origin: {}", origin);
                        None
                    })
                })
                .collect();

            tracing::info!("CORS: Allowing {} configured origins", origins.len());
            base_layer.allow_origin(AllowOrigin::list(origins))
        }
    } else {
        tracing::warn!(
            "CORS: Allowing any origin (development mode). \
             Configure CORS_ALLOWED_ORIGINS for production."
        );
        base_layer.allow_origin(Any)
    }
}

/// Create a default permissive CORS layer (for development only)
#[allow(dead_code)]
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
        .allow_origin(Any)
        .expose_headers([
            header::HeaderName::from_static(REQUEST_ID_HEADER),
            header::HeaderName::from_static("x-ratelimit-limit"),
            header::HeaderName::from_static("x-ratelimit-remaining"),
            header::HeaderName::from_static("x-ratelimit-reset"),
        ])
}
