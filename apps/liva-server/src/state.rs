use crate::config::ServerConfig;
use crate::db::DbPool;
use std::time::Instant;

/// Shared application state passed across Axum handlers and middleware.
#[derive(Clone, Debug)]
pub struct AppState {
    pub db: DbPool,
    pub config: ServerConfig,
    pub start_time: Instant,
}

impl AppState {
    /// Creates a new `AppState` instance with the specified database pool and server configuration.
    pub fn new(db: DbPool, config: ServerConfig) -> Self {
        Self {
            db,
            config,
            start_time: Instant::now(),
        }
    }

    /// Computes current uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
