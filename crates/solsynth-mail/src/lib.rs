//!
//! Mail 邮件服务
//!
//! 功能:
//! - SMTP/IMAP 集成
//! - 邮件模板引擎
//! - 邮件队列管理
//! - 密码重置、验证邮件等

pub mod models;
pub mod smtp;
pub mod templates;
pub mod queue;
pub mod handlers;

pub use models::*;
pub use smtp::*;
pub use templates::*;
pub use queue::*;
pub use handlers::*;