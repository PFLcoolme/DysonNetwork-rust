//! NATS 消息队列客户端
//!
//! 使用 NATS 实现消息的异步推送和广播

use async_nats::Client;
use tokio::sync::Mutex;
use std::sync::OnceLock;

use crate::models::{Message, WsFrame};

/// NATS 主题
pub mod topics {
    /// 消息发送主题
    pub const MESSAGE_SEND: &str = "messager.send";
    /// 消息推送主题
    pub const MESSAGE_PUSH: &str = "messager.push";
    /// 已读回执主题
    pub const READ_RECEIPT: &str = "messager.read";
    /// 打字指示器主题
    pub const TYPING: &str = "messager.typing";
    /// 系统通知主题
    pub const SYSTEM_NOTIFY: &str = "messager.system";
}

/// NATS 客户端封装
pub struct NatsClient {
    client: Mutex<Option<Client>>,
}

impl NatsClient {
    /// 全局单例
    static INSTANCE: OnceLock<Self> = OnceLock::new();

    /// 获取全局实例
    pub fn instance() -> &'static Self {
        Self::INSTANCE.get_or_init(|| Self {
            client: Mutex::new(None),
        })
    }

    /// 初始化 NATS 连接
    pub async fn init() -> anyhow::Result<()> {
        let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());

        let client = async_nats::connect(nats_url).await?;

        tracing::info!("NATS 连接成功: {}", nats_url);

        // 订阅消息推送主题
        let mut subscriber = client.subscribe(topics::MESSAGE_PUSH.to_string()).await?;

        // 在后台任务中处理消息
        tokio::spawn(async move {
            while let Ok(msg) = subscriber.next().await {
                tracing::info!("收到 NATS 消息: {:?}", msg.subject);
                // TODO: 处理消息转发逻辑
            }
        });

        let instance = Self::instance();
        *instance.client.lock().await = Some(client);

        Ok(())
    }

    /// 获取 NATS 客户端
    pub async fn client(&self) -> Option<Client> {
        self.client.lock().await.clone()
    }

    /// 发送消息到 NATS
    pub async fn send_message(message: &Message) -> anyhow::Result<()> {
        let instance = Self::instance();
        let client = instance.client().await;

        match client {
            Some(ref cx) => {
                let payload = serde_json::to_string(message)?;
                cx.publish(topics::MESSAGE_SEND.to_string(), payload.into()).await?;
                Ok(())
            }
            None => {
                anyhow::bail!("NATS 客户端未初始化")
            }
        }
    }

    /// 推送消息到用户
    pub async fn push_to_user(user_id: &str, frame: &WsFrame) -> anyhow::Result<()> {
        let instance = Self::instance();
        let client = instance.client().await;

        match client {
            Some(ref cx) => {
                let payload = serde_json::to_string(frame)?;
                let subject = format!("messager.push.{}", user_id);
                cx.publish(subject, payload.into()).await?;
                Ok(())
            }
            None => {
                anyhow::bail!("NATS 客户端未初始化")
            }
        }
    }

    /// 发布系统通知
    pub async fn publish_system_notify(content: &str) -> anyhow::Result<()> {
        let instance = Self::instance();
        let client = instance.client().await;

        match client {
            Some(ref cx) => {
                let payload = serde_json::json!({
                    "content": content,
                    "timestamp": chrono::Utc::now(),
                });
                cx.publish(topics::SYSTEM_NOTIFY.to_string(), payload.to_string().into()).await?;
                Ok(())
            }
            None => {
                anyhow::bail!("NATS 客户端未初始化")
            }
        }
    }

    /// 检查 NATS 连接状态
    pub async fn check_connection() -> bool {
        let instance = Self::instance();
        let client = instance.client().await;

        match client {
            Some(ref cx) => cx.is_connected(),
            None => false,
        }
    }
}