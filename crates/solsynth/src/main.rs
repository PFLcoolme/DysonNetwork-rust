//! SolSynth - Rust 重构的 Solar Network 服务端
//!
//! 基于 DysonNetwork 项目的 Rust 重写实现

use tracing::info;

#[tokio::main]
async fn main() {
    //初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("solsynth=info".parse().unwrap()),
        )
        .init();

    info!("Sol   Synth 正在启动...");
    info!("服务: 认证 | 用户 | 内容 | 消息 | 通话 | 支付 | 开发者");

    // TODO: 加载配置
    // TODO: 初始化数据库连接
    // TODO: 启动 gRPC 服务
    // TODO: 启动 HTTP 服务

    info!("SolSynth 启动完成!"); // 保持运行
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");

    info!("SolSynth 正在关闭...");
}
