//! Health check handlers
//!
//! Endpoints for liveness and readiness probes.

use axum::{extract::State, http::StatusCode, Json};
use chat_service::{HealthResponse, ReadinessResponse};

use crate::state::AppState;

/// Basic health check (liveness probe)
///
/// GET /health
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse::healthy())
}

/// Readiness check with dependency health
///
/// GET /health/ready
pub async fn readiness_check(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    // Check database connectivity
    let db_healthy = state
        .service_context()
        .pool()
        .acquire()
        .await
        .map(|_| true)
        .unwrap_or(false);

    // Check Redis connectivity
    let redis_healthy = state
        .service_context()
        .redis_pool()
        .health_check()
        .await
        .is_ok();

    let response = ReadinessResponse::ready(db_healthy, redis_healthy);
    let status = if db_healthy && redis_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
}
