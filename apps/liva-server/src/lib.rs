pub mod config;
pub mod db;
pub mod error;
pub mod routes;
pub mod state;

pub use config::{validate_bind_address, ServerConfig};
pub use db::{ping_db, DbError, DbPool};
pub use error::ApiError;
pub use routes::health::HealthResponse;
pub use state::AppState;

/// Creates the Axum application router configured with shared state, middleware, and routes.
pub fn create_app(state: AppState) -> axum::Router {
    routes::create_router(state)
}
