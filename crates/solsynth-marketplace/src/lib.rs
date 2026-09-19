//!
//! Marketplace 市场服务
//!
//! 功能:
//! - 商品管理
//! - 订单处理
//! - 支付集成 (Stripe)
//! - 评价系统

pub mod models;
pub mod products;
pub mod orders;
pub mod payment;
pub mod handlers;

pub use models::*;
pub use products::*;
pub use orders::*;
pub use payment::*;
pub use handlers::*;