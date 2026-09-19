//! 市场服务 API 处理器

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;

use crate::models::{
    ApiResponse, CreateOrderRequest, CreateProductRequest, UpdateProductRequest,
};
use crate::orders::OrderManager;
use crate::payment::PaymentManager;
use crate::products::ProductManager;

/// API 状态
#[derive(Clone)]
pub struct MarketplaceState {
    pub product_manager: ProductManager,
    pub order_manager: OrderManager,
    pub payment_manager: PaymentManager,
}

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(20),
            offset: Some(0),
        }
    }
}

/// 注册路由
pub fn routes(state: MarketplaceState) -> Router {
    Router::new()
        // 商品路由
        .route("/api/products", post(create_product))
        .route("/api/products/list", get(list_products))
        .route("/api/products/search", get(search_products))
        .route("/api/products/:id", get(get_product))
        .route("/api/products/:id", put(update_product))
        .route("/api/products/:id", delete(delete_product))
        // 订单路由
        .route("/api/orders", post(create_order))
        .route("/api/orders/list", get(list_orders))
        .route("/api/orders/:id", get(get_order))
        .route("/api/orders/:id/cancel", post(cancel_order))
        // 支付路由
        .route("/api/payments/create-intent", post(create_payment_intent))
        .route("/api/payments/webhook", post(payment_webhook))
        .route("/api/payments/:intent_id/refund", post(refund_payment))
        .with_state(state)
}

// ========== 商品路由 ==========

/// 创建商品
async fn create_product(
    State(state): State<MarketplaceState>,
    Json(req): Json<CreateProductRequest>,
) -> impl axum::response::IntoResponse {
    // 从 session 获取 seller_id (实际应使用认证中间件)
    let seller_id = Uuid::new_v4();
    let product = state.product_manager.create_product(req, seller_id).await;
    ApiResponse::success(product)
}

/// 获取商品列表
async fn list_products(
    State(state): State<MarketplaceState>,
    Query(params): Query<PaginationParams>,
) -> impl axum::response::IntoResponse {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);
    let products = state.product_manager.list_products(limit, offset).await;
    ApiResponse::success(products)
}

/// 搜索商品
async fn search_products(
    State(state): State<MarketplaceState>,
    Query(params): Query<SearchQuery>,
) -> impl axum::response::IntoResponse {
    let query = params.q;
    let limit = params.limit.unwrap_or(20);
    let products = state.product_manager.search_products(&query, limit).await;
    ApiResponse::success(products)
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<u32>,
}

/// 获取商品详情
async fn get_product(
    State(state): State<MarketplaceState>,
    Path(id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    match state.product_manager.get_product(id).await {
        Some(product) => ApiResponse::success(product),
        None => ApiResponse::error("商品未找到".to_string()),
    }
}

/// 更新商品
async fn update_product(
    State(state): State<MarketplaceState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> impl axum::response::IntoResponse {
    match state.product_manager.update_product(id, req).await {
        Ok(product) => ApiResponse::success(product),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

/// 删除商品
async fn delete_product(
    State(state): State<MarketplaceState>,
    Path(id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    match state.product_manager.delete_product(id).await {
        Ok(_) => ApiResponse::success(()),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

// ========== 订单路由 ==========

/// 创建订单
async fn create_order(
    State(state): State<MarketplaceState>,
    Json(req): Json<CreateOrderRequest>,
) -> impl axum::response::IntoResponse {
    let buyer_id = Uuid::new_v4();
    match state.order_manager.create_order(req, buyer_id).await {
        Ok(order) => ApiResponse::success(order),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

/// 获取订单列表
async fn list_orders(
    State(state): State<MarketplaceState>,
    Query(params): Query<PaginationParams>,
) -> impl axum::response::IntoResponse {
    let user_id = Uuid::new_v4();
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);
    let orders = state.order_manager.list_user_orders(user_id, limit, offset).await;
    ApiResponse::success(orders)
}

/// 获取订单详情
async fn get_order(
    State(state): State<MarketplaceState>,
    Path(id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    match state.order_manager.get_order(id).await {
        Some(order) => ApiResponse::success(order),
        None => ApiResponse::error("订单未找到".to_string()),
    }
}

/// 取消订单
async fn cancel_order(
    State(state): State<MarketplaceState>,
    Path(id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    match state.order_manager.cancel_order(id).await {
        Ok(order) => ApiResponse::success(order),
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

// ========== 支付路由 ==========

/// 创建支付意图
async fn create_payment_intent(
    State(state): State<MarketplaceState>,
    Json(order_id_json): Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let order_id = order_id_json["order_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok());

    match order_id {
        Some(id) => match state.order_manager.get_order(id).await {
            Some(order) => match state.payment_manager.create_payment_intent(&order).await {
                Ok(response) => ApiResponse::success(response),
                Err(e) => ApiResponse::error(e.to_string()),
            },
            None => ApiResponse::error("订单未找到".to_string()),
        },
        None => ApiResponse::error("无效订单 ID".to_string()),
    }
}

/// 支付回调 Webhook
async fn payment_webhook(
    State(state): State<MarketplaceState>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> impl axum::response::IntoResponse {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    match state
        .payment_manager
        .handle_payment_webhook(&body, signature)
        .await
    {
        Ok(event) => {
            // 处理事件
            match event.event_type {
                crate::payment::PaymentEventType::PaymentSucceeded => {
                    info!("支付成功");
                }
                crate::payment::PaymentEventType::PaymentFailed => {
                    error!("支付失败");
                }
                _ => {}
            }
            ApiResponse::success(serde_json::json!({ "event_id": event.event_id }))
        }
        Err(e) => ApiResponse::error(e.to_string()),
    }
}

/// 退款
async fn refund_payment(
    State(state): State<MarketplaceState>,
    Path(intent_id): Path<String>,
) -> impl axum::response::IntoResponse {
    match state.payment_manager.refund_payment(&intent_id, None).await {
        Ok(_) => ApiResponse::success(()),
        Err(e) => ApiResponse::error(e.to_string()),

    }
}