//! 邮件服务数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 邮件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MailType {
    PasswordReset = 0,
    EmailVerification = 1,
    Notification = 2,
    Marketing = 3,
    System = 99,
}

impl MailType {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::PasswordReset,
            1 => Self::EmailVerification,
            2 => Self::Notification,
            3 => Self::Marketing,
            99 => Self::System,
            _ => Self::System,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::PasswordReset => 0,
            Self::EmailVerification => 1,
            Self::Notification => 2,
            Self::Marketing => 3,
            Self::System => 99,
        }
    }
}

/// 邮件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MailStatus {
    Queued = 0,
    Sending = 1,
    Sent = 2,
    Failed = 3,
}

impl MailStatus {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Queued,
            1 => Self::Sending,
            2 => Self::Sent,
            3 => Self::Failed,
            _ => Self::Queued,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Queued => 0,
            Self::Sending => 1,
            Self::Sent => 2,
            Self::Failed => 3,
        }
    }
}

/// 邮件模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailTemplate {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
    pub variables: Vec<String>,
}

/// 邮件消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailMessage {
    pub id: Uuid,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
    pub attachments: Vec<String>,
    pub mail_type: MailType,
    pub template_name: Option<String>,
    pub template_vars: Option<serde_json::Value>,
    pub status: MailStatus,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// SMTP 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub tls: bool,
    pub from_address: String,
    pub from_name: String,
    pub max_connections: u32,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.example.com".to_string()),
            port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            username: std::env::var("SMTP_USER").unwrap_or_else(|_| "".to_string()),
            password: std::env::var("SMTP_PASS").unwrap_or_else(|_| "".to_string()),
            tls: true,
            from_address: std::env::var("SMTP_FROM_ADDRESS").unwrap_or_else(|_| "noreply@solsynth.com".to_string()),
            from_name: std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Solsynth".to_string()),
            max_connections: 10,
        }
    }
}

/// 发送邮件请求
#[derive(Debug, Deserialize)]
pub struct SendMailRequest {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
    pub attachments: Option<Vec<String>>,
    pub template_name: Option<String>,
    pub template_vars: Option<serde_json::Value>,
}

/// 发送响应
#[derive(Debug, Serialize)]
pub struct SendMailResponse {
    pub mail_id: String,
    pub status: String,
}

/// 邮件队列项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub mail_id: Uuid,
    pub priority: u8,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub mail_message: MailMessage,
}

/// 邮件服务统计
#[derive(Debug, Serialize)]
pub struct MailStats {
    pub total_queued: u64,
    pub total_sending: u64,
    pub total_sent: u64,
    pub total_failed: u64,
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