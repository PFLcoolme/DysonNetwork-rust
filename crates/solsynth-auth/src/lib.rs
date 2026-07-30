//! SolSynth Auth - 认证与授权模块 (对应 Padlock 服务)
//!
//! 功能:
//! - 用户注册/登录/登出
//! - JWT Access Token 和 Refresh Token
//! - OIDC 第三方登录 (Google, Apple, GitHub 等)
//! - WebAuthn/Passkey 认证
//! - 双因素认证 (2FA)
//! - 会话管理

pub mod handlers;
pub mod models;
pub mod oidc;

pub use handlers::*;
pub use models::*;