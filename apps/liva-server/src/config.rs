use std::net::IpAddr;

/// Default request body limit in bytes (50 MB).
pub const DEFAULT_BODY_LIMIT_BYTES: usize = 50 * 1024 * 1024;

/// Default server port.
pub const DEFAULT_PORT: u16 = 8080;

/// Default host (strictly loopback).
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Default database URL (in-memory SQLite for self-contained execution).
pub const DEFAULT_DATABASE_URL: &str = "sqlite::memory:";

/// Default maximum database pool connections.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// Configuration errors for server startup and zero-egress enforcement.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ConfigError {
    #[error("Zero-Egress violation: attempted to bind to '{0}'. Only loopback addresses are permitted.")]
    ZeroEgressViolation(String),

    #[error("Invalid port '{0}': {1}")]
    InvalidPort(String, String),

    #[error("Invalid max connections '{0}': {1}")]
    InvalidMaxConnections(String, String),

    #[error("Invalid body limit '{0}': {1}")]
    InvalidBodyLimit(String, String),
}

/// Server configuration defining network bindings, database connectivity, and limits.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub max_connections: u32,
    pub body_limit_bytes: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
            database_url: DEFAULT_DATABASE_URL.to_string(),
            max_connections: DEFAULT_MAX_CONNECTIONS,
            body_limit_bytes: DEFAULT_BODY_LIMIT_BYTES,
        }
    }
}

/// Strictly validates that the provided host is a loopback address.
///
/// Under Zero-Egress requirements (R5), the server must never bind to external interfaces
/// (such as 0.0.0.0 or LAN IP addresses).
pub fn validate_bind_address(host: &str) -> Result<(), ConfigError> {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::ZeroEgressViolation(
            "Empty host address is not allowed".to_string(),
        ));
    }

    if trimmed.eq_ignore_ascii_case("localhost") {
        return Ok(());
    }

    // Strip optional IPv6 square brackets, e.g. "[::1]"
    let unbracketed = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(trimmed);

    match unbracketed.parse::<IpAddr>() {
        Ok(ip) => {
            if ip.is_loopback() {
                Ok(())
            } else {
                Err(ConfigError::ZeroEgressViolation(format!(
                    "Zero-Egress violation: attempted to bind to '{host}'. Only loopback addresses are permitted."
                )))
            }
        }
        Err(_) => Err(ConfigError::ZeroEgressViolation(format!(
            "Zero-Egress violation: attempted to bind to '{host}'. Only loopback addresses are permitted."
        ))),
    }
}

impl ServerConfig {
    /// Loads configuration from environment variables, falling back to secure defaults.
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = std::env::var("LIVA_SERVER_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
        validate_bind_address(&host)?;

        let port = match std::env::var("LIVA_SERVER_PORT") {
            Ok(p) => p.parse::<u16>().map_err(|e| {
                ConfigError::InvalidPort(p, e.to_string())
            })?,
            Err(_) => DEFAULT_PORT,
        };

        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

        let max_connections = match std::env::var("LIVA_MAX_CONNECTIONS") {
            Ok(m) => m.parse::<u32>().map_err(|e| {
                ConfigError::InvalidMaxConnections(m, e.to_string())
            })?,
            Err(_) => DEFAULT_MAX_CONNECTIONS,
        };

        let body_limit_bytes = match std::env::var("LIVA_BODY_LIMIT_BYTES") {
            Ok(b) => b.parse::<usize>().map_err(|e| {
                ConfigError::InvalidBodyLimit(b, e.to_string())
            })?,
            Err(_) => DEFAULT_BODY_LIMIT_BYTES,
        };

        Ok(Self {
            host,
            port,
            database_url,
            max_connections,
            body_limit_bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bind_address_valid() {
        assert!(validate_bind_address("127.0.0.1").is_ok());
        assert!(validate_bind_address("::1").is_ok());
        assert!(validate_bind_address("[::1]").is_ok());
        assert!(validate_bind_address("localhost").is_ok());
        assert!(validate_bind_address("LOCALHOST").is_ok());
    }

    #[test]
    fn test_validate_bind_address_rejects_external() {
        assert!(validate_bind_address("0.0.0.0").is_err());
        assert!(validate_bind_address("::").is_err());
        assert!(validate_bind_address("192.168.1.1").is_err());
        assert!(validate_bind_address("10.0.0.1").is_err());
        assert!(validate_bind_address("8.8.8.8").is_err());
        assert!(validate_bind_address("example.com").is_err());
        assert!(validate_bind_address("").is_err());
    }

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.body_limit_bytes, 50 * 1024 * 1024);
        assert!(validate_bind_address(&config.host).is_ok());
    }
}
