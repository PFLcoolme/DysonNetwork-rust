//! 认证模块数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 用户账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,          // 用户名
    pub nick: String,          // 昵称
    pub language: String,      // 语言 (en-US, zh-Hans)
    pub region: String,        // 地区
    pub is_superuser: bool,
    pub activated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 认证会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: Uuid,
    pub account_id: Uuid,
    pub device_id: String,
    pub device_name: Option<String>,
    pub platform: ClientPlatform,
    pub access_token: String,
    pub access_expires_at: DateTime<Utc>,
    pub refresh_token: String,
    pub refresh_expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub expired_at: DateTime<Utc>,
}

/// 认证挑战
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub id: Uuid,
    pub account_id: Uuid,
    pub device_id: String,
    pub device_name: Option<String>,
    pub platform: ClientPlatform,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub step_total: i32,
    pub step_remain: i32,
    pub failed_attempts: i32,
    pub expired_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 认证因子类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountAuthFactorType {
    Password,
    Passkey,
    WebAuthn,
    Email,
    Phone,
    RecoveryCode,
    QrLogin,
}

/// 认证因子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAuthFactor {
    pub id: Uuid,
    pub account_id: Uuid,
    pub factor_type: AccountAuthFactorType,
    pub enabled_at: Option<DateTime<Utc>>,
    pub trustworthy: i32,
    pub created_at: DateTime<Utc>,
}

/// 客户端平台
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClientPlatform {
    Unidentified,
    Web,
    IOS,
    Android,
    Desktop,
}

impl std::fmt::Display for ClientPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientPlatform::Unidentified => write!(f, "Unidentified"),
            ClientPlatform::Web => write!(f, "Web"),
            ClientPlatform::IOS => write!(f, "IOS"),
            ClientPlatform::Android => write!(f, "Android"),
            ClientPlatform::Desktop => write!(f, "Desktop"),
        }
    }
}

/// Token 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: i64,
       pub refresh_expires_in: i64,
 pub token_type: String,
}