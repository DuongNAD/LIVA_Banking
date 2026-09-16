use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::db::ping_db;
use crate::state::AppState;

/// Standardized health check response payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub service: String,
    pub database: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub uptime_seconds: u64,
    pub timestamp: String,
}

/// Handler for `/health` and `/api/health` endpoints.
///
/// Pings the database pool using `SELECT 1`:
/// - Returns HTTP 200 OK with `"status": "ok"`, `"database": "connected"` if healthy.
/// - Returns HTTP 503 Service Unavailable with `"status": "degraded"`, `"database": "disconnected"` if the pool is closed or unreachable.
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.uptime_seconds();
    let timestamp = Utc::now().to_rfc3339();
    let version = env!("CARGO_PKG_VERSION").to_string();
    let service = "liva-server".to_string();

    match ping_db(&state.db).await {
        Ok(_) => {
            let response = HealthResponse {
                status: "ok".to_string(),
                version,
                service,
                database: "connected".to_string(),
                error: None,
                uptime_seconds: uptime,
                timestamp,
            };
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            let response = HealthResponse {
                status: "degraded".to_string(),
                version,
                service,
                database: "disconnected".to_string(),
                error: Some(err.to_string()),
                uptime_seconds: uptime,
                timestamp,
            };
            (StatusCode::SERVICE_UNAVAILABLE, Json(response))
        }
    }
}
