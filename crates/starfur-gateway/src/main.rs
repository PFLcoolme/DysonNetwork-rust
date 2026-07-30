mod config;
mod limiter;
mod proxy;
mod routing;

use std::{sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use config::Config;
use limiter::RateLimiter;
use proxy::AppState;
use routing::RouteTable;
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Serialize)]
struct HealthResponse<'a> {
    status: &'static str,
    service: &'static str,
    routes: &'a [&'a str],
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let config = Config::from_env()?;
    let bind_addr = config.bind_addr;
    let external_scheme = HeaderValue::from_str(&config.external_scheme)?;
    let routes = RouteTable::new(config.upstreams);
    let limiter = Arc::new(RateLimiter::per_minute(config.rate_limit_per_minute));
    let state = AppState::new(routes, limiter, config.upstream_timeout, external_scheme);

    let app = Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .fallback(proxy::proxy)
        .with_state(state);
    let listener = TcpListener::bind(bind_addr).await?;
    info!(%bind_addr, "Starfur Gateway listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

async fn live() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: "starfur-gateway",
        routes: state.public_routes(),
    })
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("starfur_gateway=info,tower_http=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .init();
}

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
        () = ctrl_c => {},
        () = terminate => {},
        () = tokio::time::sleep(Duration::from_secs(u64::MAX)) => {},
    }
}
