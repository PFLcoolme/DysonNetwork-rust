//! WebSocket 连接管理器
//!
//! 管理所有活跃的 WebSocket 连接，支持消息推送和广播

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::WsFrame;

/// 全局单例
static INSTANCE: OnceLock<WebSocketManager> = OnceLock::new();

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
#[derive(Clone)]
pub struct WebSocketManager {
    /// 活跃会话
    sessions: Arc<Mutex<Vec<Arc<WsSession>>>>,
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 获取全局实例
    pub fn instance() -> &'static Self {
        INSTANCE.get_or_init(Self::new)
    }

    /// 初始化
    pub fn init() {
        tracing::info!("WebSocket 管理器初始化完成");
    }

    /// 创建新的 WebSocket 会话
    pub async fn create_session(&self, user_id: String, ws: WebSocket) -> Uuid {
        let (send, _recv) = ws.split();

        let session_id = Uuid::new_v4();
        let session = Arc::new(WsSession {
            id: session_id,
            user_id: user_id.clone(),
            send: Mutex::new(send),
        });

        self.sessions.lock().await.push(session);

        tracing::info!("新用户 WebSocket 连接: user={}, session={}", user_id, session_id);

        session_id
    }

    /// 关闭会话
    pub async fn close_session(&self, session_id: Uuid) {
        let mut sessions = self.sessions.lock().await;
        let mut retained = Vec::with_capacity(sessions.len());
        for session in sessions.drain(..) {
            if session.id != session_id {
                retained.push(session);
            }
        }
        *sessions = retained;
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
            if session.user_id == user_id {
                let mut send = session.send.lock().await;
                if let Err(e) = send.send(Message::Text(payload.clone().into())).await {
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
    let session_id = WebSocketManager::instance()
        .create_session(user_id.clone(), ws)
        .await;

    let user_uid = Uuid::parse_str(&user_id).unwrap_or(Uuid::nil());
    let connected_frame = WsFrame::Connected { user_id: user_uid };
    let _payload = serde_json::to_string(&connected_frame).unwrap();

    // TODO: 发送确认消息

    tracing::info!("WebSocket 会话 {} 已建立，等待消息...", session_id);
}
