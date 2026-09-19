//! 消息系统 API 处理器
//!
//! 提供消息发送、接收、已读回执等 HTTP API

use axum::{
    extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{
    CreateMessageRequest, Message, MessageType, UnreadCount, WsFrame,
};
use crate::websocket::WebSocketManager;
use solsynth_db::message::{
    MessageService, MessageStatus as DbMessageStatus, MessageType as DbMessageType,
};

/// API 状态
#[derive(Clone)]
pub struct MessagerState {
    pub ws_manager: WebSocketManager,
    pub db: DatabaseConnection,
}

/// 响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
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

/// 将数据库的 NaiveDateTime (UTC) 转换为 DateTime<Utc>
fn to_utc(naive: NaiveDateTime) -> DateTime<Utc> {
    DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
}

/// 查询参数 - 获取消息历史
#[derive(Debug, Deserialize)]
pub struct MessageHistoryQuery {
    pub conversation_id: String,
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

/// 将数据库消息模型转换为 API 模型
fn convert_message(m: &solsynth_db::message::Model) -> Message {
    Message {
        id: Uuid::parse_str(&m.id).unwrap_or(Uuid::new_v4()),
        sender_id: Uuid::parse_str(&m.sender_id).unwrap_or(Uuid::new_v4()),
        receiver_id: Uuid::parse_str(&m.receiver_id).unwrap_or(Uuid::new_v4()),
        message_type: m.message_type.into(),
        content: m.content.clone(),
        content_type: m.content_type.clone(),
        status: m.status.into(),
        created_at: to_utc(m.created_at),
        updated_at: to_utc(m.updated_at),
        reply_to: m.reply_to.as_ref().and_then(|s| Uuid::parse_str(s).ok()),
        attachments: m
            .attachments
            .as_ref()
            .and_then(|v| {
                v.as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            })
            .unwrap_or_default(),
    }
}

/// 发送消息
async fn send_message(
    State(state): State<MessagerState>,
    Json(req): Json<CreateMessageRequest>,
) -> impl IntoResponse {
    tracing::info!("收到发送消息请求: receiver={}", req.receiver_id);

    // 从认证上下文获取 sender_id (TODO: 实现认证中间件)
    let sender_id = "demo-sender".to_string();

    // 创建消息服务
    let message_service = MessageService::new(state.db.clone());

    // 转换消息类型
    let db_message_type = match req.message_type {
        MessageType::Direct => DbMessageType::Direct,
        MessageType::Group => DbMessageType::Group,
        MessageType::System => DbMessageType::System,
    };

    // 存储消息到数据库
    match message_service
        .create_message(
            &sender_id,
            &req.receiver_id.to_string(),
            &req.content,
            db_message_type,
            &req.content_type,
            None, // group_id (私聊时)
            req.reply_to.as_ref().map(|id| id.to_string()).as_deref(),
            if req.attachments.is_empty() {
                None
            } else {
                Some(req.attachments)
            },
        )
        .await
    {
        Ok(db_message) => {
            tracing::info!("消息存储成功: {}", db_message.id);

            // 构造 WebSocket 消息帧
            let message = convert_message(&db_message);

            // 通过 WebSocket 推送给接收者
            state
                .ws_manager
                .send_to_user(&db_message.receiver_id, &WsFrame::Message { message })
                .await;

            ApiResponse::success(db_message.id)
        }
        Err(e) => {
            tracing::error!("消息存储失败: {}", e);
            ApiResponse::error(format!("消息存储失败: {}", e))
        }
    }
}

/// 获取消息历史
async fn message_history(
    State(state): State<MessagerState>,
    Query(query): Query<MessageHistoryQuery>,
) -> impl IntoResponse {
    tracing::info!(
        "获取消息历史: conversation={}, limit={}",
        query.conversation_id,
        query.limit
    );

    let message_service = MessageService::new(state.db.clone());

    // 获取 sender_id (TODO: 从认证上下文获取)
    let user_id = "demo-user".to_string();

    match message_service
        .get_message_history(&user_id, &query.conversation_id, query.limit)
        .await
    {
        Ok(messages) => {
            let message_list: Vec<Message> = messages.iter().map(convert_message).collect();

            ApiResponse::success(message_list)
        }
        Err(e) => {
            tracing::error!("获取消息历史失败: {}", e);
            ApiResponse::error(format!("获取消息历史失败: {}", e))
        }
    }
}

/// 标记为已读
async fn mark_as_read(
    State(state): State<MessagerState>,
    Path(message_id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("标记消息为已读: {}", message_id);

    // TODO: 从认证上下文获取 user_id 并校验消息归属
    let message_service = MessageService::new(state.db.clone());

    match message_service
        .update_message_status(&message_id, DbMessageStatus::Read)
        .await
    {
        Ok(()) => ApiResponse::success("已标记为已读"),
        Err(e) => {
            tracing::error!("标记已读失败: {}", e);
            ApiResponse::error(format!("标记已读失败: {}", e))
        }
    }
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

async fn handle_ws(socket: WebSocket, user_id: String, state: MessagerState) {
    let _session_id = state
        .ws_manager
        .create_session(user_id.clone(), socket)
        .await;

    tracing::info!("用户 {} 的 WebSocket 连接已建立", user_id);

    // TODO: 处理 WebSocket 消息
}

// ==================== 类型转换 ====================

impl From<i8> for MessageType {
    fn from(value: i8) -> Self {
        match value {
            0 => Self::Direct,
            1 => Self::Group,
            2 => Self::System,
            _ => Self::Direct,
        }
    }
}

impl From<MessageType> for i8 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::Direct => 0,
            MessageType::Group => 1,
            MessageType::System => 2,
        }
    }
}

impl From<i8> for crate::models::MessageStatus {
    fn from(value: i8) -> Self {
        match value {
            0 => Self::Sending,
            1 => Self::Sent,
            2 => Self::Delivered,
            3 => Self::Read,
            4 => Self::Failed,
            _ => Self::Sending,
        }
    }
}
