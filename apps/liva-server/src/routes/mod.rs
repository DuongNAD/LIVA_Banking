pub mod health;

use axum::{
    body::Bytes,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};

use crate::state::AppState;
use health::health_handler;

/// Test endpoint to exercise request body size limits and payload handling.
async fn test_payload_handler(bytes: Bytes) -> (StatusCode, String) {
    (StatusCode::OK, format!("Received {} bytes", bytes.len()))
}

/// Constructs the primary Axum router with health endpoints, payload limits, and tracing middleware.
pub fn create_router(state: AppState) -> Router {
    let limit_layer = RequestBodyLimitLayer::new(state.config.body_limit_bytes);

    Router::new()
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        .route("/api/v1/test/payload", post(test_payload_handler))
        .layer(TraceLayer::new_for_http())
        .layer(limit_layer)
        .with_state(state)
}
