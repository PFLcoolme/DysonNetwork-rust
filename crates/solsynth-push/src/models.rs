//! 推送服务数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 推送类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PushType {
    Notification = 0,
    Message = 1,
    System = 2,
    Marketing = 3,
}

impl PushType {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Notification,
            1 => Self::Message,
            2 => Self::System,
            3 => Self::Marketing,
            _ => Self::Notification,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Notification => 0,
            Self::Message => 1,
            Self::System => 2,
            Self::Marketing => 3,
        }
    }
}

/// 推送渠道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PushChannel {
    Fcm = 0,
    Apns = 1,
    Websocket = 2,
    Web = 3,
}

impl PushChannel {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Fcm,
            1 => Self::Apns,
            2 => Self::Websocket,
            3 => Self::Web,
            _ => Self::Fcm,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Fcm => 0,
            Self::Apns => 1,
            Self::Websocket => 2,
            Self::Web => 3,
        }
    }
}

/// 推送状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PushStatus {
    Pending = 0,
    Sending = 1,
    Sent = 2,
    Failed = 3,
}

impl PushStatus {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Pending,
            1 => Self::Sending,
            2 => Self::Sent,
            3 => Self::Failed,
            _ => Self::Pending,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Pending => 0,
            Self::Sending => 1,
            Self::Sent => 2,
            Self::Failed => 3,
        }
    }
}

/// 推送设备
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushDevice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: String,
    pub platform: String,
    pub push_token: String,
    pub push_channel: PushChannel,
    pub app_version: String,
    pub is_active: bool,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 推送通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotification {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub data: Option<serde_json::Value>,
    pub push_type: PushType,
    pub target_user_id: Option<Uuid>,
    pub target_device_id: Option<Uuid>,
    pub target_group: Option<String>,
    pub channel: PushChannel,
    pub status: PushStatus,
    pub error_message: Option<String>,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 发送推送请求
#[derive(Debug, Deserialize)]
pub struct SendPushRequest {
    pub title: String,
    pub body: String,
    pub data: Option<serde_json::Value>,
    pub target_user_id: Option<Uuid>,
    pub target_device_id: Option<Uuid>,
    pub target_group: Option<String>,
    pub channel: Option<PushChannel>,
}

/// 注册设备请求
#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_id: String,
    pub platform: String,
    pub push_token: String,
    pub push_channel: PushChannel,
    pub app_version: String,
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

/// 推送统计
#[derive(Debug, Serialize)]
pub struct PushStats {
    pub total_pending: u64,
    pub total_sending: u64,
    pub total_sent: u64,
    pub total_failed: u64,
    pub total_devices: u64,
}