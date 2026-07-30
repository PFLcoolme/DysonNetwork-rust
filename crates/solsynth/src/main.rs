//! SolSynth - DyonNetwork Rust 重构主程序入口
//!
//! 启动所有核心服务:
//! - Padlock 认证服务
//! - 未来可扩展其他服务

use std::sync::Arc;

use axum::Router;
use tokio::select;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::Layer;

use solsynth_auth::{AuthState, JwtConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 Tracing
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_level(true);

    let env_filter = tracing_subscriber::filter::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive(LevelFilter::INFO.into());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();

    info!("正在启动 SolSynth 服务...");

    // 初始化数据库连接
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:@127.0.0.1:3306/solsynth".to_string());

    let pool = sqlx::MySqlPool::connect(&db_url).await?;
    info!("数据库连接成功");

    // 初始化认证服务
    let jwt_config = JwtConfig::default();
    let auth_service = models::service::AuthService::new(pool, jwt_config);
    let auth_state = AuthState {
        auth_service,
        jwt_config,
        cookie_name: "solsynth_session".to_string(),
    };

    // 构建认证服务路由
    let auth_router = Router::new()
        .nest("/auth", handlers::routes(auth_state.clone()))
        .into_make_service();

    // 启动认证服务 (Padlock)
    let auth_addr = "0.0.0.0:5101";
    info!(%auth_addr, "Padlock 认证服务启动");

    let listener = tokio::net::TcpListener::bind(auth_addr).await?;
    axum::serve(listener, auth_router).await?;

    Ok(())
}