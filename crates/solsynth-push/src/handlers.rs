//! 推送服务 API 处理器

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::models::{
    ApiResponse, PushNotification, PushDevice,
    SendPushRequest, RegisterDeviceRequest,
};
use crate::push_manager::PushManager;
use crate::websocket::WebSocketPushManager;
use crate::fcm::PushService;

/// API 状态
#[derive(Clone)]
pub struct PushState {
    pub push_manager: PushManager,
    pub ws_manager: WebSocketPushManager,
    pub push_service: PushService,
}

/// 注册路由
pub fn routes(state: PushState) -> Router {
    Router::new()
        // 推送路由
        .route("/api/push/send", post(send_push))
        .route("/api/push/stats", get(get_stats))
        // 设备路由
        .route("/api/devices/register", post(register_device))
        .route("/api/devices/:user_id", get(get_device))
        .route("/api/devices/:user_id/unregister", post(unregister_device))
        // WebSocket 路由
        .route("/api/ws", get(ws_connect))
        .with_state(state)
}

// ========== 推送路由 ==========

/// 发送推送通知
async fn send_push(
    State(state): State<PushState>,
    Json(req): Json<SendPushRequest>,
) -> impl axum::response::IntoResponse {
    let notification = PushNotification {
        id: Uuid::new_v4(),
        title: req.title,
        body: req.body,
        data: req.data,
        push_type: crate::models::PushType::Notification,
        target_user_id: req.target_user_id,
        target_device_id: req.target_device_id,
        target_group: req.target_group,
        channel: req.channel.unwrap_or(crate::models::PushChannel::Fcm),
        status: crate::models::PushStatus::Pending,
        error_message: None,
        sent_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let id = state.push_manager.send_notification(notification);
    ApiResponse::success(serde_json::json!({ "notification_id": id }))
}

/// 获取推送统计
async fn get_stats(
    State(state): State<PushState>,
) -> impl axum::response::IntoResponse {
    let stats = state.push_manager.get_stats();
    ApiResponse::success(stats)
}

// ========== 设备路由 ==========

/// 注册推送设备
async fn register_device(
    State(state): State<PushState>,
    Json(req): Json<RegisterDeviceRequest>,
) -> impl axum::response::IntoResponse {
    let device = PushDevice {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        device_id: req.device_id,
        platform: req.platform,
        push_token: req.push_token,
        push_channel: req.push_channel,
        app_version: req.app_version,
        is_active: true,
        last_seen_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.push_manager.register_device(device.clone());
    ApiResponse::success(device)
}

/// 获取用户设备
async fn get_device(
    State(state): State<PushState>,
    Path(user_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    match state.push_manager.get_user_device(user_id) {
        Some(device) => ApiResponse::success(device),
        None => ApiResponse::error("设备未找到".to_string()),
    }
}

/// 注销设备
async fn unregister_device(
    State(state): State<PushState>,
    Path(user_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    if state.push_manager.unregister_device(user_id) {
        ApiResponse::success(serde_json::json!({ "message": "设备已注销" }))
    } else {
        ApiResponse::error("设备未找到".to_string())
    }
}

// ========== WebSocket 路由 ==========

/// WebSocket 连接
async fn ws_connect(
    State(_state): State<PushState>,
    ws: WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|_socket| async move {
        // 实际实现应处理 WebSocket 消息
        // 这里仅作为占位符：接收设备注册、转发推送通知
    })
}