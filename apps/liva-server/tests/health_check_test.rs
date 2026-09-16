use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use liva_server::{
    config::validate_bind_address, create_app, db::DbPool, AppState, HealthResponse, ServerConfig,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_1_health_check_connected_db_returns_200() {
    let pool = DbPool::in_memory_sqlite()
        .await
        .expect("in-memory sqlite initialization");
    let config = ServerConfig::default();
    let state = AppState::new(pool, config);
    let app = create_app(state);

    let req = Request::builder()
        .uri("/health")
        .method(Method::GET)
        .body(Body::empty())
        .expect("request builder");

    let res = app.oneshot(req).await.expect("service execution");

    assert_eq!(res.status(), StatusCode::OK);

    let bytes = res.into_body().collect().await.expect("collect body").to_bytes();
    let health: HealthResponse = serde_json::from_slice(&bytes).expect("parse health json");

    assert_eq!(health.status, "ok");
    assert_eq!(health.database, "connected");
    assert_eq!(health.service, "liva-server");
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    assert!(health.error.is_none());
    assert!(!health.timestamp.is_empty());
}

#[tokio::test]
async fn test_2_api_health_returns_consistent_schema() {
    let pool = DbPool::in_memory_sqlite()
        .await
        .expect("in-memory sqlite initialization");
    let config = ServerConfig::default();
    let state = AppState::new(pool, config);
    let app = create_app(state);

    let req = Request::builder()
        .uri("/api/health")
        .method(Method::GET)
        .body(Body::empty())
        .expect("request builder");

    let res = app.oneshot(req).await.expect("service execution");

    assert_eq!(res.status(), StatusCode::OK);

    let bytes = res.into_body().collect().await.expect("collect body").to_bytes();
    let health: HealthResponse = serde_json::from_slice(&bytes).expect("parse health json");

    assert_eq!(health.status, "ok");
    assert_eq!(health.database, "connected");
    assert_eq!(health.service, "liva-server");
    assert!(health.error.is_none());
}

#[tokio::test]
async fn test_3_health_check_disconnected_db_returns_503() {
    let pool = DbPool::in_memory_sqlite()
        .await
        .expect("in-memory sqlite initialization");
    let config = ServerConfig::default();
    let state = AppState::new(pool.clone(), config);
    let app = create_app(state);

    // Explicitly close the database connection pool to simulate failure
    pool.close().await;

    let req = Request::builder()
        .uri("/health")
        .method(Method::GET)
        .body(Body::empty())
        .expect("request builder");

    let res = app.oneshot(req).await.expect("service execution");

    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);

    let bytes = res.into_body().collect().await.expect("collect body").to_bytes();
    let health: HealthResponse = serde_json::from_slice(&bytes).expect("parse health json");

    assert_eq!(health.status, "degraded");
    assert_eq!(health.database, "disconnected");
    assert_eq!(health.service, "liva-server");
    assert!(health.error.is_some(), "error message must be present when degraded");
}

#[tokio::test]
async fn test_4_zero_egress_bind_validator() {
    // Valid loopback bindings must succeed
    assert!(validate_bind_address("127.0.0.1").is_ok());
    assert!(validate_bind_address("127.0.0.2").is_ok());
    assert!(validate_bind_address("::1").is_ok());
    assert!(validate_bind_address("[::1]").is_ok());
    assert!(validate_bind_address("localhost").is_ok());
    assert!(validate_bind_address("LOCALHOST").is_ok());

    // External / wildcard interfaces must be strictly rejected
    let external_hosts = [
        "0.0.0.0",
        "::",
        "192.168.1.100",
        "10.0.0.1",
        "172.16.0.1",
        "8.8.8.8",
        "example.com",
        "evil.attacker.com",
        "",
        " ",
    ];

    for host in external_hosts {
        let result = validate_bind_address(host);
        assert!(
            result.is_err(),
            "Host '{}' should have been rejected by zero-egress validator",
            host
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Zero-Egress violation") || err_msg.contains("Empty host"),
            "Error message should explain zero-egress violation: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_5_request_body_limit_enforcement() {
    // Sub-test A: Valid payload under limit succeeds
    {
        let pool = DbPool::in_memory_sqlite()
            .await
            .expect("in-memory sqlite");
        let config = ServerConfig::default(); // 50 MB limit
        let state = AppState::new(pool, config);
        let app = create_app(state);

        let small_body = vec![65u8; 1024]; // 1 KB
        let req = Request::builder()
            .uri("/api/v1/test/payload")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(small_body))
            .expect("request builder");

        let res = app.oneshot(req).await.expect("service execution");
        assert_eq!(res.status(), StatusCode::OK);
    }

    // Sub-test B: Payload exceeding configured limit is rejected with HTTP 413
    {
        let pool = DbPool::in_memory_sqlite()
            .await
            .expect("in-memory sqlite");
        let config = ServerConfig {
            body_limit_bytes: 1024, // 1 KB limit for precise verification
            ..Default::default()
        };
        let state = AppState::new(pool, config);
        let app = create_app(state);

        let oversized_body = vec![65u8; 2048]; // 2 KB (exceeds 1 KB limit)
        let req = Request::builder()
            .uri("/api/v1/test/payload")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(oversized_body))
            .expect("request builder");

        let res = app.oneshot(req).await.expect("service execution");
        assert_eq!(
            res.status(),
            StatusCode::PAYLOAD_TOO_LARGE,
            "Oversized payload should be rejected with 413 Payload Too Large"
        );
    }

    // Sub-test C: 50MB default limit rejection using streamed chunks without high memory consumption
    {
        let pool = DbPool::in_memory_sqlite()
            .await
            .expect("in-memory sqlite");
        let config = ServerConfig::default(); // 50 MB default limit (52,428,800 bytes)
        let state = AppState::new(pool, config);
        let app = create_app(state);

        // Stream 820 chunks of 64KB = 53,739,520 bytes (> 50 MB), bounded in flight memory
        let chunk = axum::body::Bytes::from(vec![0u8; 64 * 1024]);
        let stream = futures_util::stream::iter((0..820).map(move |_| {
            Ok::<_, std::io::Error>(axum::body::Bytes::clone(&chunk))
        }));
        let body = Body::from_stream(stream);

        let req = Request::builder()
            .uri("/api/v1/test/payload")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(body)
            .expect("request builder");

        let res = app.oneshot(req).await.expect("service execution");
        assert_eq!(
            res.status(),
            StatusCode::PAYLOAD_TOO_LARGE,
            "Stream exceeding 50MB should be rejected with 413 Payload Too Large"
        );
    }
}
