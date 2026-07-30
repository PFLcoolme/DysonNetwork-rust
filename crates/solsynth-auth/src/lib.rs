//!认证模块 - 对应 Padlock 服务
//!
//! 功能:
//! - 用户登录/注册
//! - OIDC 第三方登录 (Google, Apple, GitHub, Microsoft, Discord, Afdian, Steam)
//! - WebAuthn/Passkey 认证
//! - 双因素认证 (2FA)
//! - 会话管理
//! - Token 签发与验证

pub mod handlers;
pub mod models;
pub mod oidc;
pub mod services;

pub use models::*;
pub use services::*;
