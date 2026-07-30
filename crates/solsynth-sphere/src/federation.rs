//! ActivityPub Federation 配置与初始化

use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use axum::Router;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Sphere 服务配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SphereConfig {
    /// 实例域名
    pub domain: String,
    /// 实例显示名称
    pub instance_name: String,
    /// 实例描述
    pub instance_description: String,
    /// 管理员信息
    pub admin_email: String,
    /// 是否启用注册
    pub enable_registration: bool,
    /// API 端口
    pub port: u16,
}

impl Default for SphereConfig {
    fn default() -> Self {
        Self {
            domain: std::env::var("SPHERE_DOMAIN").unwrap_or_else(|_| "localhost".to_string()),
            instance_name: std::env::var("SPHERE_INSTANCE_NAME").unwrap_or_else(|_| "Starfur Sphere".to_string()),
            instance_description: std::env::var("SPHERE_INSTANCE_DESC").unwrap_or_else(|_| "A federated content platform".to_string()),
            admin_email: std::env::var("SPHERE_ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string()),
            enable_registration: true,
            port: 5103,
        }
    }
}

impl SphereConfig {
    /// 创建 ActivityPub Federation 配置
    pub async fn federation_config(&self) -> FederationConfig<SphereContext> {
        FederationConfig::builder()
            .app_data(self.clone())
            .domain(self.domain.clone())
            .actor_name("sphere".to_string())
            .public_keys(vec![activitypub_federation::config::PublicKey {
                key_id: "https://".to_string() + &self.domain + "/.well-known/public-key".to_string(),
                public_key_pem: generate_dummy_public_key(),
            }])
            .manual_start(true)
            .build()
            .await
            .unwrap()
    }

    /// 构建 Router
    pub fn router(&self) -> Router {
        Router::new()
    }
} 

/// 生成模拟 RSA公钥 (仅用于开发)
fn generate_dummy_public_key() -> String {
    // 在实际生产中，这里应该生成真实的 RSA 密钥对
    r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3VS5JJcds3xfn/ygWyF
+ZOMPiBqelVi7d0g3JFNmC6z5q7OBtbtwMZHhdRlFy00YNuBrJzl
2S3hT5QqEeC+GkXY5mJrFaX6QkNnYRfGpQAeRBxNM3R3dJTK3Ri8zq3l2B2VZfFhGZ3VZ5lBp4
Xa5V5WZkVmZJYqXZ2XZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5lBp4Xa5V5WZkVmZJYqX
Z2XZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5lBp4Xa5V5W
ZkVmZJYqXZ2XZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5l
Bp4Xa5V5WZkVmZJYqXZ2XZ5lBp4QIDAQAB
-----END PUBLIC KEY-----"#.to_string()
}

/// Sphere 应用上下文
#[derive(Clone)]
pub struct SphereContext {
    config: SphereConfig,
}

impl SphereContext {
    pub fn new(config: SphereConfig) -> Self {
        Self { config }
    }

    pub async fn get_activitypub_context(
        &self,
    ) -> activitypub_federation::context::ActivityPubContext<Self> {
        activitypub_federation::context::ActivityPubContext::new(self.clone())
    }
}

impl activitypub_federation::config::Context for SphereContext {
    fn new_ctx(
        _app_data: &SphereConfig,
        request: &axum::extract::Request,
    ) -> Self {
        Self {
            config: _app_data.clone(),
        }
    }

    fn get_activitypub_config(
        &self,
    ) -> async_std::sync::Arc<activitypub_federation::config::FederationConfig<Self>> {
        unimplemented!("Use FederationConfig::builder() to build")
    }
}