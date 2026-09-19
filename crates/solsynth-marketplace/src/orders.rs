//! 订单处理模块
//!
//! 负责订单的创建、更新和状态管理

use anyhow::Result;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::{
    Order, OrderStatus, Product, CreateOrderRequest,
};

/// 订单管理器
#[derive(Clone)]
pub struct OrderManager {
    orders: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Order>>>,
    products: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Product>>>,
}

impl OrderManager {
    /// 创建新的订单管理器
    pub fn new(
        products: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Product>>>,
    ) -> Self {
        Self {
            orders: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            products,
        }
    }

    /// 创建订单
    pub async fn create_order(&self, req: CreateOrderRequest, buyer_id: Uuid) -> Result<Order> {
        // 获取商品信息
        let products = self.products.read().await;
        let product = products
            .get(&req.product_id)
            .ok_or_else(|| anyhow::anyhow!("商品未找到"))?
            .clone();

        // 检查库存
        if product.stock_quantity < req.quantity {
            anyhow::bail!("库存不足");
        }

        // 检查商品状态
        if product.status != crate::models::ProductStatus::Active {
            anyhow::bail!("商品不可销售");
        }

        let total_amount = product.price * req.quantity as f64;

        // 创建订单
        let order = Order {
            id: Uuid::new_v4(),
            buyer_id,
            seller_id: product.seller_id,
            product_id: product.id,
            quantity: req.quantity,
            unit_price: product.price,
            total_amount,
            currency: product.currency.clone(),
            status: OrderStatus::Pending,
            payment_method: req.payment_method,
            payment_intent_id: None,
            shipping_address: req.shipping_address,
            notes: req.notes,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut orders = self.orders.write().await;
        orders.insert(order.id, order.clone());

        info!("订单已创建: {} (商品: {}, 数量: {})", order.id, product.name, req.quantity);

        Ok(order)
    }

    /// 获取订单
    pub async fn get_order(&self, order_id: Uuid) -> Option<Order> {
        let orders = self.orders.read().await;
        orders.get(&order_id).cloned()
    }

    /// 获取用户订单
    pub async fn list_user_orders(&self, user_id: Uuid, limit: u32, offset: u32) -> Vec<Order> {
        let orders = self.orders.read().await;
        let mut user_orders: Vec<Order> = orders
            .values()
            .filter(|o| o.buyer_id == user_id || o.seller_id == user_id)
            .cloned()
            .collect();
        user_orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        user_orders
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect()
    }

    /// 更新订单状态
    pub async fn update_order_status(&self, order_id: Uuid, status: OrderStatus) -> Result<Order> {
        let mut orders = self.orders.write().await;

        if let Some(order) = orders.get_mut(&order_id) {
            let old_status = order.status.clone();
            order.status = status.clone();
            order.updated_at = chrono::Utc::now();

            info!(
                "订单状态变更: {} ({:?} -> {:?})",
                order_id, old_status, status
            );

            Ok(order.clone())
        } else {
            anyhow::bail!("订单未找到");
        }
    }

    /// 确认付款
    pub async fn confirm_payment(
        &self,
        order_id: Uuid,
        payment_intent_id: String,
    ) -> Result<Order> {
        let mut orders = self.orders.write().await;

        if let Some(order) = orders.get_mut(&order_id) {
            order.status = OrderStatus::Paid;
            order.payment_intent_id = Some(payment_intent_id);
            order.updated_at = chrono::Utc::now();

            info!("订单付款已确认: {}", order_id);

            Ok(order.clone())
        } else {
            anyhow::bail!("订单未找到");
        }
    }

    /// 完成订单
    pub async fn complete_order(&self, order_id: Uuid) -> Result<Order> {
        self.update_order_status(order_id, OrderStatus::Completed).await
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<Order> {
        // 退款逻辑
        let orders = self.orders.read().await;
        if let Some(order) = orders.get(&order_id) {
            if order.status == OrderStatus::Paid {
                warn!("订单 {} 已付款，需要退款", order_id);
            }
        }

        self.update_order_status(order_id, OrderStatus::Cancelled).await

    }

    /// 退款
    pub async fn refund_order(&self, order_id: Uuid) -> Result<()> {
        let mut orders = self.orders.write().await;

        if let Some(order) = orders.get_mut(&order_id) {
            if order.status != OrderStatus::Paid {
                anyhow::bail!("只能退款已付款的订单");
            }

            order.status = OrderStatus::Refunded;
            order.updated_at = chrono::Utc::now();

            info!("订单已退款: {}", order_id);
            Ok(())
        } else {
            anyhow::bail!("订单未找到");
        }
    }
}

impl Default for OrderManager {
    fn default() -> Self {
        Self {
            orders: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            products: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }
}