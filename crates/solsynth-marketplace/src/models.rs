//! 市场服务数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 商品类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductType {
    Digital = 0,
    Physical = 1,
    Service = 2,
    Subscription = 3,
}

impl ProductType {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Digital,
            1 => Self::Physical,
            2 => Self::Service,
            3 => Self::Subscription,
            _ => Self::Digital,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Digital => 0,
            Self::Physical => 1,
            Self::Service => 2,
            Self::Subscription => 3,
        }
    }
}

/// 商品状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductStatus {
    Draft = 0,
    Active = 1,
    OutOfStock = 2,
    Disabled = 3,
}

impl ProductStatus {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Draft,
            1 => Self::Active,
            2 => Self::OutOfStock,
            3 => Self::Disabled,
            _ => Self::Draft,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Draft => 0,
            Self::Active => 1,
            Self::OutOfStock => 2,
            Self::Disabled => 3,
        }
    }
}

/// 订单状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending = 0,
    Paid = 1,
    Processing = 2,
    Completed = 3,
    Refunded = 4,
    Cancelled = 5,
}

impl OrderStatus {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Pending,
            1 => Self::Paid,
            2 => Self::Processing,
            3 => Self::Completed,
            4 => Self::Refunded,
            5 => Self::Cancelled,
            _ => Self::Pending,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Pending => 0,
            Self::Paid => 1,
            Self::Processing => 2,
            Self::Completed => 3,
            Self::Refunded => 4,
            Self::Cancelled => 5,
        }
    }
}

/// 支付方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    Stripe = 0,
    PayPal = 1,
    Crypto = 2,
    Balance = 3,
}

impl PaymentMethod {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Stripe,
            1 => Self::PayPal,
            2 => Self::Crypto,
            3 => Self::Balance,
            _ => Self::Stripe,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Stripe => 0,
            Self::PayPal => 1,
            Self::Crypto => 2,
            Self::Balance => 3,
        }
    }
}

/// 商品
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub currency: String,
    pub product_type: ProductType,
    pub status: ProductStatus,
    pub stock_quantity: i32,
    pub digital_file_id: Option<Uuid>,
    pub images: Vec<String>,
    pub tags: Vec<String>,
    pub is_featured: bool,
    pub rating_avg: f64,
    pub rating_count: i32,
    pub sales_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_amount: f64,
    pub currency: String,
    pub status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub payment_intent_id: Option<String>,
    pub shipping_address: Option<String>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 评价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub order_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub product_id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 创建商品请求
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub currency: Option<String>,
    pub product_type: ProductType,
    pub stock_quantity: Option<i32>,
    pub tags: Option<Vec<String>>,
}

/// 更新商品请求
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub stock_quantity: Option<i32>,
    pub status: Option<ProductStatus>,
    pub tags: Option<Vec<String>>,
}

/// 创建订单请求
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub product_id: Uuid,
    pub quantity: i32,
    pub payment_method: PaymentMethod,
    pub shipping_address: Option<String>,
    pub notes: Option<String>,
}

/// 支付响应
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub order_id: String,
    pub payment_url: Option<String>,
    pub client_secret: Option<String>,
}

/// 响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> axum::response::IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
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