//! WebSocket 推送模块
//!
//! 负责 WebSocket 连接管理和实时推送

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error};
use uuid::Uuid;

use crate::models::{PushDevice, PushNotification};

/// WebSocket 连接
pub struct WebSocketConnection {
    pub connection_id: Uuid,
    pub user_id: Uuid,
    pub device: PushDevice,
    pub sender: mpsc::Sender<PushNotification>,
}

/// WebSocket 推送管理器
#[derive(Clone)]
pub struct WebSocketPushManager {
    connections: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, WebSocketConnection>>>,
}

impl WebSocketPushManager {
    /// 创建新的 WebSocket 推送管理器
    pub fn new() -> Self {
        Self {
            connections: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// 建立 WebSocket 连接
    pub async fn connect(&self, user_id: Uuid, device: PushDevice) -> Result<mpsc::Receiver<PushNotification>> {
        let connection_id = Uuid::new_v4();
        let (sender, receiver) = mpsc::channel::<PushNotification>(100);

        let ws_conn = WebSocketConnection {
            connection_id,
            user_id,
            device,
            sender,
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id, ws_conn);

        info!("WebSocket 连接已建立: {} (用户: {})", connection_id, user_id);

        Ok(receiver)
    }

    /// 断开 WebSocket 连接
    pub async fn disconnect(&self, connection_id: Uuid) {
        let mut connections = self.connections.write().await;
        if connections.remove(&connection_id).is_some() {
            info!("WebSocket 连接已断开: {}", connection_id);
        }
    }

    /// 向用户发送推送
    pub async fn send_to_user(&self, user_id: Uuid, notification: PushNotification) -> Result<()> {
        let connections = self.connections.read().await;
        for conn in connections.values() {
            if conn.user_id == user_id {
                if conn.sender.send(notification.clone()).await.is_err() {
                    error!("向用户 {} 发送推送失败", user_id);
                }
            }
        }
        Ok(())
    }

    /// 获取在线用户数量
    pub async fn get_online_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

impl Default for WebSocketPushManager {
    fn default() -> Self {
        Self::new()
    }
}