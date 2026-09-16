use liva_server::{config::validate_bind_address, create_app, db::DbPool, AppState, ServerConfig};
use tracing::info;

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "liva_server=info,tower_http=info".into()),
        )
        .init();

    info!("Starting LIVA Banking Server (Milestone M3)...");

    let config = ServerConfig::from_env()?;
    validate_bind_address(&config.host)?;

    info!(
        host = %config.host,
        port = config.port,
        "Zero-Egress validated: host strictly bound to loopback"
    );

    let db = DbPool::connect(&config.database_url, config.max_connections).await?;
    info!("Database connection established");

    let state = AppState::new(db, config.clone());
    let app = create_app(state);

    let bind_addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("LIVA Banking Server listening on http://{}", bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("LIVA Banking Server shut down gracefully");
    Ok(())
}
