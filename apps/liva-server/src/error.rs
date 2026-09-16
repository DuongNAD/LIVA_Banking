use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// Standardized JSON error response envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Standardized API errors mapped to HTTP status codes.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ApiError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "ERR_BAD_REQUEST",
            Self::NotFound(_) => "ERR_NOT_FOUND",
            Self::PayloadTooLarge(_) => "ERR_PAYLOAD_TOO_LARGE",
            Self::Database(_) => "ERR_DATABASE_ERROR",
            Self::ServiceUnavailable(_) => "ERR_SERVICE_UNAVAILABLE",
            Self::Config(_) => "ERR_CONFIG_ERROR",
            Self::Internal(_) => "ERR_INTERNAL_SERVER_ERROR",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error_code: self.error_code().to_string(),
            message: self.to_string(),
            details: None,
        };
        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<crate::config::ConfigError> for ApiError {
    fn from(err: crate::config::ConfigError) -> Self {
        Self::Config(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ApiError::BadRequest("invalid query".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::NotFound("item".into()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::PayloadTooLarge("too big".into()).status_code(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
        assert_eq!(
            ApiError::Database("connection pool closed".into()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            ApiError::ServiceUnavailable("degraded".into()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            ApiError::Internal("crash".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
