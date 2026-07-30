//! 消息系统 API 处理器
//!
//! 提供消息发送、接收、已读回执等 HTTP API

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{
    CreateMessageRequest, Message, MessageStatus, MessageType, UnreadCount, WsFrame,
};
use crate::websocket::WebSocketManager;

/// API 状态
#[derive(Clone)]
pub struct MessagerState {
    pub ws_manager: WebSocketManager,
}

/// 响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(msg),
        }
    }
}

/// 查询参数 - 获取消息历史
#[derive(Debug, Deserialize)]
pub struct MessageHistoryQuery {
    pub conversation_id: Uuid,
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub cursor: Option<String>,
}

fn default_limit() -> i32 {
    50
}

/// 注册路由
pub fn routes(state: MessagerState) -> Router {
    Router::new()
        .route("/messages", post(send_message))
        .route("/messages/history", get(message_history))
        .route("/messages/{id}/read", post(mark_as_read))
        .route("/unread/count", get(unread_count))
        .route("/ws", get(ws_upgrade))
        .with_state(state)
}

/// 发送消息
async fn send_message(
    State(state): State<MessagerState>,
    Json(req): Json<CreateMessageRequest>,
) -> impl IntoResponse {
    tracing::info!("收到发送消息请求: receiver={}", req.receiver_id);

    // TODO: 验证请求
    // TODO: 存储到数据库

    let message = Message {
        id: Uuid::new_v4(),
        sender_id: Uuid::new_v4(), // TODO: 从认证上下文获取
        receiver_id: req.receiver_id,
        message_type: req.message_type,
        content: req.content,
        content_type: req.content_type,
        status: MessageStatus::Sent,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        reply_to: req.reply_to,
        attachments: req.attachments,

    };

    // 通过 WebSocket 推送给接收者
    state.ws_manager
        .send_to_user(&message.receiver_id.to_string(), &WsFrame::Message { message })
        .await;

    ApiResponse::success("消息发送成功")
}

/// 获取消息历史
async fn message_history(
    State(_state): State<MessagerState>,
    Query(query): Query<MessageHistoryQuery>,
) -> impl IntoResponse {
    tracing::info!("获取消息历史: conversation={}, limit={}", query.conversation_id, query.limit);

    // TODO: 从数据库查询消息历史

    ApiResponse::success(vec![])
}

/// 标记为已读
async fn mark_as_read(
    State(state): State<MessagerState>,
    Path(message_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!("标记消息为已读: {}", message_id);

    // TODO: 更新数据库

    ApiResponse::success("已标记为已读")
}

/// 获取未读消息数
async fn unread_count(
    State(_state): State<MessagerState>,
) -> impl IntoResponse {
    // TODO: 从数据库获取未读数

    ApiResponse::success(UnreadCount {
        user_id: Uuid::new_v4(),
        total_unread: 0,
        conversations: vec![],
    })
}

/// WebSocket 升级
async fn ws_upgrade(
    State(state): State<MessagerState>,
    request: WebSocketUpgrade,
) -> impl IntoResponse {
    // TODO: 从认证上下文获取 user_id
    let user_id = "demo-user".to_string();

    request.on_upgrade(move |socket| handle_ws(socket, user_id, state))
}

async fn handle_ws(socket: axum::extract::WebSocket, user_id: String, state: MessagerState) {
    let _session_id = WebSocketManager::create_session(user_id.clone(), socket).await;

    tracing::info!("用户 {} 的 WebSocket 连接已建立", user_id);

    // TODO: 处理 WebSocket 消息
}