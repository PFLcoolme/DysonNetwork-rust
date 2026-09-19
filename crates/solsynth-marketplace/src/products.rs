//! 商品管理模块
//!
//! 负责商品的创建、更新、查询和管理

use anyhow::Result;
use tracing::{info, error};
use uuid::Uuid;

use crate::models::{Product, ProductStatus, CreateProductRequest, UpdateProductRequest};

/// 商品管理器
#[derive(Clone)]
pub struct ProductManager {
    products: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Product>>>,
}

impl ProductManager {
    /// 创建新的商品管理器
    pub fn new() -> Self {
        Self {
            products: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 创建商品
    pub async fn create_product(&self, req: CreateProductRequest, seller_id: Uuid) -> Product {
        let product = Product {
            id: Uuid::new_v4(),
            seller_id,
            name: req.name,
            description: req.description,
            price: req.price,
            currency: req.currency.unwrap_or_else(|| "USD".to_string()),
            product_type: req.product_type,
            status: ProductStatus::Draft,
            stock_quantity: req.stock_quantity.unwrap_or(0),
            digital_file_id: None,
            images: Vec::new(),
            tags: req.tags.unwrap_or_default(),
            is_featured: false,
            rating_avg: 0.0,
            rating_count: 0,
            sales_count: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut products = self.products.write().await;
        products.insert(product.id, product.clone());
        info!("商品已创建: {} (ID: {})", product.name, product.id);
        product
    }

    /// 更新商品
    pub async fn update_product(&self, product_id: Uuid, req: UpdateProductRequest) -> Result<Product> {
        let mut products = self.products.write().await;
        
        if let Some(product) = products.get_mut(&product_id) {
            if let Some(name) = req.name {
                product.name = name;
            }
            if let Some(description) = req.description {
                product.description = description;
            }
            if let Some(price) = req.price {
                product.price = price;
            }
            if let Some(stock) = req.stock_quantity {
                product.stock_quantity = stock;
            }
            if let Some(status) = req.status {
                product.status = status;
            }
            if let Some(tags) = req.tags {
                product.tags = tags;
            }
            product.updated_at = chrono::Utc::now();
            
            let updated = product.clone();
            info!("商品已更新: {}", product_id);
            Ok(updated)
        } else {
            error!("商品未找到: {}", product_id);
            anyhow::bail!("商品未找到");
        }
    }

    /// 获取商品
    pub async fn get_product(&self, product_id: Uuid) -> Option<Product> {
        let products = self.products.read().await;
        products.get(&product_id).cloned()
    }

    /// 获取所有商品
    pub async fn list_products(&self, limit: u32, offset: u32) -> Vec<Product> {
        let products = self.products.read().await;
        let mut all_products: Vec<Product> = products.values().cloned().collect();
        all_products.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        all_products.into_iter().skip(offset as usize).take(limit as usize).collect()
    }

    /// 按卖家获取商品
    pub async fn list_seller_products(&self, seller_id: Uuid, limit: u32, offset: u32) -> Vec<Product> {
        let products = self.products.read().await;
        let mut seller_products: Vec<Product> = products.values()
            .filter(|p| p.seller_id == seller_id)
            .cloned()
            .collect();
        seller_products.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        seller_products.into_iter().skip(offset as usize).take(limit as usize).collect()
    }

    /// 搜索商品
    pub async fn search_products(&self, query: &str, limit: u32) -> Vec<Product> {
        let products = self.products.read().await;
        let query_lower = query.to_lowercase();
        let mut results: Vec<Product> = products.values()
            .filter(|p| p.name.to_lowercase().contains(& 
                      query_lower) || p.description.to_lowercase().contains(&query_lower) ||
                       p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)))
            .cloned()
            .collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results.into_iter().take(limit as usize).collect()
    }

    /// 删除商品
    pub async fn delete_product(&self, product_id: Uuid) -> Result<()> {
        let mut products = self.products.write().await;
        if products.remove(&product_id).is_some() {
            info!("商品已删除: {}", product_id);
            Ok(())
        } else {
            anyhow::bail!("商品未找到");
        }
    }
}

impl Default for ProductManager {
    fn default() -> Self {
        Self::new()
    }
}