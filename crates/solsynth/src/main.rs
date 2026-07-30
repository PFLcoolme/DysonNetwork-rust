//! SolSynth - DysonNetwork Rust 重构主程序入口
//!
//! 启动核心服务:
//! - Padlock 认证服务 (5101)

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化 Tracing
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_level(true);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive(LevelFilter::INFO.into());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .init();

    info!("正在启动 SolSynth 服务...");

    // ====================
    // Padlock 认证服务 (5101)
    // ====================
    if let Err(e) = run_padlock().await {
        tracing::error!(error = ?e, "Padlock 服务启动失败");
        return Err(e);
    }

    Ok(())
}

async fn run_padlock() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use solsynth_auth::{AuthState, AuthService, JwtConfig};
    
    // 初始化数据库连接
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:@127.0.0.1:3306/solsynth".to_string());
    
    let db = sea_orm::Database::connect(&db_url).await?;
    info!("Padlock: 数据库连接成功");
    
    // 初始化认证服务
    let jwt_config = JwtConfig::default();
    let auth_service = AuthService::new(db, jwt_config.clone());
    
    let auth_state = AuthState {
        auth_service,
        cookie_name: "solsynth_session".to_string(),
    };
    
    // 构建路由
    let app = solsynth_auth::handlers::routes(auth_state);
    
    let addr = "0.0.0.0:5101";
    info!(%addr, "Padlock 认证服务启动");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}