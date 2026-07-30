//! SolSynth - DysonNetwork Rust 重构主程序入口
//!
//! 启动所有核心服务:
//! - Padlock 认证服务 (5101)
//! - Passport 用户资料 (5102)
//! - Sphere 联邦内容 (5103)
//! - Messager 实时消息 (5104)
//! - Ring 音视频信令 (5105)
//! - Wallet 支付订阅 (5106)

use std::sync::Arc;

use tokio::select;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
    // Padlock 认证服务
    // ====================
    let auth_handle = tokio::spawn(async {
        if let Err(e) = run_padlock().await {
            tracing::error!(error = ?e, "Padlock 服务启动失败");
        }
    });

    // ====================
    // Sphere 联邦内容服务
    // ====================
    let sphere_handle = tokio::spawn(async {
        if let Err(e) = run_sphere().await {
            tracing::error!(error = ?e, "Sphere 服务启动失败");
        }
    });

    // 等待服务退出
    select! {
        result = auth_handle => {
            if let Err(e) = result {
                tracing::error!(error = ?e, "Padlock 任务panic");
            }
        }
        result = sphere_handle => {
            if let Err(e) = result {
                tracing::error!(error = ?e, "Sphere 任务panic");
            }
        }
    }

    Ok(())
}

async fn run_padlock() -> anyhow::Result<()> {
    use solsynth_auth::{AuthState, JwtConfig};
    
    // 初始化数据库连接
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:@127.0.0.1:3306/solsynth".to_string());
    
    let pool = sqlx::MySqlPool::connect(&db_url).await?;
    info!("Padlock: 数据库连接成功");
    
    // 初始化认证服务
    let jwt_config = JwtConfig::default();
    let auth_service = solsynth_auth::models::service::AuthService::new(
        pool,
        jwt_config,
    );
    
    let auth_state = AuthState {
        auth_service,
        jwt_config,
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

async fn run_sphere() -> anyhow::Result<()> {
    use solsynth_sphere::{SphereConfig, SphereContext};
    
    let config = SphereConfig::default();
    let context = SphereContext::new(config.clone());
    
    let app = solsynth_sphere::handlers::routes(context);
    
    let addr = format!("0.0.0.0:{}", config.port);
    info!(%addr, "Sphere 联邦内容服务启动");
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}