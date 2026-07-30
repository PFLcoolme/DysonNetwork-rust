//! # StarFurr Messager - 实时消息系统
//!
//! Messager 是 StarFurr 平台的实时消息推送服务，支持：
//! - 一对一私聊
//! - 群组消息
//! - 消息已读/未读状态
//! - 消息推送 (NATS + WebSocket)
//! - 消息历史存储 (MySQL)

pub mod handlers;
pub mod models;
pub mod nats_client;
pub mod websocket;

pub use handlers::*;
pub use models::*;
pub use nats_client::NatsClient;
pub use websocket::WebSocketManager;

/// 消息系统初始化
pub async fn init() -> anyhow::Result<()> {
    tracing::info!("初始化 Messager 消息系统...");

    // 初始化 WebSocket 管理器
    WebSocketManager::init();

    tracing::info!("Messager 消息系统初始化完成");
    Ok(())
}

/// 获取消息系统健康状态
pub async fn health_check() -> bool {
    NatsClient::check_connection().await
}