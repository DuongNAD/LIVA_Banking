use std::time::Duration;

/// Maximum duration allowed for a database ping before timing out.
pub const DB_PING_TIMEOUT: Duration = Duration::from_millis(2000);

/// Database-related errors.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Unsupported database URL scheme: '{0}'. Supported schemes: 'sqlite:', 'postgres:', 'postgresql:'")]
    UnsupportedDatabaseUrl(String),

    #[error("Database connection pool is closed")]
    PoolClosed,

    #[error("Database ping timed out after {0:?}")]
    Timeout(Duration),
}

/// Managed database connection pool supporting SQLite and PostgreSQL.
#[derive(Clone, Debug)]
pub enum DbPool {
    Sqlite(sqlx::SqlitePool),
    Postgres(sqlx::PgPool),
}

impl DbPool {
    /// Connects to the database specified by `url` with the given connection limit.
    pub async fn connect(url: &str, max_connections: u32) -> Result<Self, DbError> {
        let trimmed = url.trim();
        if trimmed.starts_with("sqlite:") {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(max_connections)
                .connect(trimmed)
                .await?;
            Ok(Self::Sqlite(pool))
        } else if trimmed.starts_with("postgres:") || trimmed.starts_with("postgresql:") {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(max_connections)
                .connect(trimmed)
                .await?;
            Ok(Self::Postgres(pool))
        } else {
            Err(DbError::UnsupportedDatabaseUrl(url.to_string()))
        }
    }

    /// Creates an in-memory SQLite pool for fast testing and standalone operation.
    pub async fn in_memory_sqlite() -> Result<Self, DbError> {
        Self::connect("sqlite::memory:", 5).await
    }

    /// Checks if the underlying pool has been closed.
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Sqlite(pool) => pool.is_closed(),
            Self::Postgres(pool) => pool.is_closed(),
        }
    }

    /// Closes the connection pool. Subsequent queries will fail.
    pub async fn close(&self) {
        match self {
            Self::Sqlite(pool) => pool.close().await,
            Self::Postgres(pool) => pool.close().await,
        }
    }

    /// Executes `SELECT 1` against the pool with a 2000ms timeout, returning ping duration.
    pub async fn ping(&self) -> Result<Duration, DbError> {
        if self.is_closed() {
            return Err(DbError::PoolClosed);
        }

        let start = std::time::Instant::now();
        let ping_fut = async {
            match self {
                Self::Sqlite(pool) => {
                    sqlx::query("SELECT 1").execute(pool).await?;
                    Ok::<(), sqlx::Error>(())
                }
                Self::Postgres(pool) => {
                    sqlx::query("SELECT 1").execute(pool).await?;
                    Ok::<(), sqlx::Error>(())
                }
            }
        };

        match tokio::time::timeout(DB_PING_TIMEOUT, ping_fut).await {
            Ok(Ok(())) => Ok(start.elapsed()),
            Ok(Err(err)) => Err(DbError::Sqlx(err)),
            Err(_) => Err(DbError::Timeout(DB_PING_TIMEOUT)),
        }
    }
}

/// Standalone helper to ping a database pool with `SELECT 1`.
pub async fn ping_db(pool: &DbPool) -> Result<Duration, DbError> {
    pool.ping().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_in_memory_pool_ping() {
        let pool = DbPool::in_memory_sqlite().await.expect("connect sqlite");
        let duration = ping_db(&pool).await.expect("ping should succeed");
        assert!(duration.as_millis() < 2000);
        assert!(!pool.is_closed());

        pool.close().await;
        assert!(pool.is_closed());
        let ping_after_close = ping_db(&pool).await;
        assert!(ping_after_close.is_err());
    }

    #[tokio::test]
    async fn test_unsupported_url_scheme() {
        let err = DbPool::connect("mysql://localhost/test", 5).await;
        assert!(err.is_err());
    }
}
