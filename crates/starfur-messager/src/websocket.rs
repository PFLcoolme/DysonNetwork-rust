//! WebSocket 连接管理器
//!
//! 管理所有活跃的 WebSocket 连接，支持消息推送和广播

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use futures::SinkExt;
use tokio::sync::Mutex;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::WsFrame;

/// WebSocket 会话
pub struct WsSession {
    /// 会话 ID
    pub id: Uuid,
    /// 用户 ID
    pub user_id: String,
    /// 发送通道
    pub send: Mutex<SplitSink<WebSocket, Message>>,
}

/// WebSocket 管理器
pub struct WebSocketManager {
    /// 活跃会话
    sessions: Mutex<Vec<Arc<Mutex<WsSession>>>>,
}

impl WebSocketManager {
    /// 全局单例
    static INSTANCE: std::sync::OnceLock<Self> = std::sync::OnceLock::new();

    /// 获取全局实例
    pub fn instance() -> &'static Self {
        Self::INSTANCE.get_or_init(|| Self {
            sessions: Mutex::new(Vec::new()),
        })
    }

    /// 初始化
    pub fn init() {
        tracing::info!("WebSocket 管理器初始化完成");
    }

    /// 创建新的 WebSocket 会话
    pub async fn create_session(user_id: String, ws: WebSocket) -> Uuid {
        let instance = Self::instance();
        let (send, _recv) = ws.split();

        let session = Arc::new(Mutex::new(WsSession {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            send: Mutex::new(send),
        }));

        let session_id = session.id;
        instance.sessions.lock().await.push(session);

        tracing::info!("新用户 WebSocket 连接: user={}, session={}", user_id, session_id);

        session_id
    }

    /// 关闭会话
    pub async fn close_session(&self, session_id: Uuid) {
        let mut sessions = self.sessions.lock().await;
        sessions.retain(|s| s.id != session_id);
        tracing::info!("WebSocket 连接关闭: session={}", session_id);
    }

    /// 向指定用户发送消息
    pub async fn send_to_user(&self, user_id: &str, frame: &WsFrame) {
        let payload = match serde_json::to_string(frame) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("序列化消息失败: {}", e);
                return;
            }
        };

        let sessions = self.sessions.lock().await;
        for session in sessions.iter() {
            let sess = session.lock().await;
            if sess.user_id == user_id {
                if let Err(e) = sess.send.send(Message::Text(payload.clone().into())).await {
                    tracing::error!("发送消息失败: {}", e);
                }
            }
        }
    }

    /// 广播消息给所有连接
    pub async fn broadcast(&self, frame: &WsFrame) {
        let payload = match serde_json::to_string(frame) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("序列化消息失败: {}", e);
                return;
            }
        };

        let sessions = self.sessions.lock().await;
        for session in sessions.iter() {
            let mut send = session.send.lock().await;
            if let Err(e) = send.send(Message::Text(payload.clone().into())).await {
                tracing::error!("广播消息失败: {}", e);
            }
        }
    }

    /// 获取在线用户数量
    pub async fn online_count(&self) -> usize {
        self.sessions.lock().await.len()
    }
}

/// 处理 WebSocket 连接的主要函数
pub async fn handle_ws_upgrade(ws: WebSocket, user_id: String) {
    let session_id = WebSocketManager::create_session(user_id.clone(), ws).await;

    let connected_frame = WsFrame::Connected { user_id };
    let _payload = serde_json::to_string(&connected_frame).unwrap();

    // TODO: 发送确认消息

    tracing::info!("WebSocket 会话 {} 已建立，等待消息...", session_id);
}