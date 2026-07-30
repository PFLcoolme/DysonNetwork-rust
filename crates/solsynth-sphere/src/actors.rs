//! ActivityPub Actor (代理) 管理
 //!
//! 包含: 实例 Actor与用户 Actor

use activitypub_federation::packages::actor::{Actor, PrivateKey, PublicKey};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::federation::{SphereConfig, SphereContext};

/// Actor 数据库模型
#[derive(Debug, Clone, Serialize, Deserialize, Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crate::models::actors::table)]
pub struct ActorModel {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub summary: Option<String>,
    pub public_key: String,
    pub private_key: String,
    pub inbox_url: String,
    pub outbox_url: String,
    pub shared_inbox_url: Option<String>,
    pub endpoint_url: String,
    pub actor_type: ActorType,
    pub is_instance: bool,
    pub avatar_url: Option<String>,
    pub header_image_url: Option<String>,
    pub manipulated: bool,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Actor 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorType {
    Person,
    Service,
    Organization,
    Application,
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorType::Person => write!(f, "Person"),
            ActorType::Service => write!(f, "Service"),
            ActorType::Organization => write!(f, "Organization"),
            ActorType::Application => write!(f, "Application"),
        }
    }
}

/// ActivityPub Actor 实现
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SphereActor {
    #[serde(skip)]
    pub database_model: ActorModel,
    #[serde(skip)]
    pub context: SphereContext,
}

#[async_trait::async_trait]
impl Actor for SphereActor {
    type Context = SphereContext;
    type Error = anyhow::Error;
    type Model = ActorModel;

    async fn read(
        id: url::Url,
        context: &Self::Context,
    ) -> Result<Option<Self::Model>, activitypub_federation::Error> {
        use crate::models::actors;
        
        let db = crate::get_pool();
        let mut conn = db.get().map_err(|e| activitypub_federation::Error::BadRequest(e.to_string()))?;
        
        let actor = actors::table
            .filter(actors::endpoint_url.eq(id.to_string()))
            .first::<ActorModel>(&mut conn)
            .optional()
            .map_err(|e| activitypub_federation::Error::BadRequest(e.to_string()))?;
        
        Ok(actor)
    }

    async fn create(
        model: Self::Model,
        _context: &Self::Context,
    ) -> Result<Self::Model, Self::Error> {
        use crate::models::actors;
        
        let db = crate::get_pool();
        let mut conn = db.get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        
        diesel::insert_into(actors::table)
            .values(&model)
            .execute(&mut conn)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        
        Ok(model)
    }

    async fn update(
        model: Self::Model,
        context: &Self::Context,
    ) -> Result<Self::Model, Self::Error> {
        use crate::models::actors;
        
        let db = crate::get_pool();
        let mut conn = db.get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        
        diesel::update(actors::table)
            .filter(actors::id.eq(model.id))
            .set((
                actors::name.eq(model.name),
                actors::display_name.eq(model.display_name),
                actors::summary.eq(model.summary),
                actors::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        
        Ok(model)
    }

    async fn delete(
        id: url::Url,
        context: &Self::Context,
    ) -> Result<(), Self::Error> {
        use crate::models::actors;
        
        let db = crate::get_pool();
        let mut conn = db.get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        
        diesel::update(actors::table)
            .filter(actors::endpoint_url.eq(id.to_string()))

            .set((                actors::deleted.eq(true),
                actors::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        
        Ok(())
    }

    fn database_model(&self) -> &Self::Model {
        &self.database_model
    }

    fn context(&self) -> &Self::Context {
        &self.context
    }
}

/// 创建实例 Actor
pub fn create_instance_actor(config: &SphereConfig) -> ActorModel {
    use uuid::Uuid;
    
    let now = Utc::now();
    
    ActorModel {
        id: Uuid::now_v7(),
        name: "sphere".to_string(),
        display_name: Some(config.instance_name.clone()),
        summary: Some(config.instance_description.clone()),
        public_key: generate_dummy_public_key(),
        private_key: generate_dummy_private_key(),
        inbox_url: format!("https://{}/inbox", config.domain),
        outbox_url: format!("https://{}/outbox", config.domain),
        shared_inbox_url: Some(format!("https://{}/shared-inbox", config.domain)),
        endpoint_url: format!("https://{}/u/sphere", config.domain),
        actor_type: ActorType::Application,
        is_instance: true,
        avatar_url: None,
        header_image_url: None,
        manipulated: false,
        deleted: false,
        created_at: now,
        updated_at: now,
    }
}

/// 生成模拟私钥 (仅用于开发)
fn generate_dummy_private_key() -> String {
    r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCz
RltC30Z3VS5JJcds3xfn/ygWyF+ZOMPiBqelVi7d0g3JFNmC6z5q7OBtbtwMZHh
dRlFy00YNuBrJzl2S3hT5QqEeC+GkXY5mJrFaX6QkNnYRfGpQAeRBxNM3R
3dJTK3Ri8zq3l2B2VZfFhGZ3VZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5lBp4Xa5
V5WZkVmZJYqXZ2XZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5lBp4Xa5V5WZkVmZJYq
XZ2XZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5lBp4Xa5V5WZkVmZJYqXZ2XZ5lBp4X
a5V5WZkVmZJYqXZ2XZ5lBp4QIDAQABAoIBAC3Rm7Lq8f5E3b5Y5Z5f5Z5f5Z
5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z
5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z
5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z
5f5
Z5fECgYEA7qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq
qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq
qqECgYEA2d0Y5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f
5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f
5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f5Z5f
5f5Z5fECgYApBqElMTC3RltC30Z3VS5JJcds3xfn/ygWyF+ZOMPiBqelVi7d0
g3JFNmC6z5q7OBtbtwMZHhdRlFy00YNuBrJzl2S3hT5QqEeC+GkXY5mJrFaX
6QkNnYRfGpQAeRBxNM3R3dJTK3Ri8zq3l2B2VZfFhGZ3VZ5lBp4CQKBgQCke5
qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq
qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq
qqqqqqqqqqqqqqqqqqqqqqqqqqqqECgYEA4qElMTC3RltC30Z3VS5JJcds3xfn/y
gWyF+ZOMPiBqelVi7d0g3JFNmC6z5q7OBtbtwMZHhdRlFy00YNuBrJzl2S3hT
5QqEeC+GkXY5mJrFaX6QkNnYRfGpQAeRBxNM3R3dJTK3Ri8zq3l2B2VZfFhG
-----END PRIVATE KEY-----"#.to_string()
}

/// Actor API 处理器
pub mod actor_handler {
    use activitypub_federation::config::AuthorityExt;
    use
 axum::{        extract::{Request, State},
        http::StatusCode,
        response::IntoResponse,
        Json,
    };
    use tracing::info;

    use crate::federation::SphereContext;
    use crate::models::actors;

    /// 获取实例 Actor
    pub async fn instance_actor(
        State(context): State<SphereContext>,
    ) -> impl IntoResponse {
        let config = context.config().clone();
        
        let actor = crate::actors::create_instance_actor(&config);
        
        Json(actor)
    }

    /// 获取用户 Actor
    pub async fn user_actor(
        State(context): State<SphereContext>,
        axum::extract::Path(username): axum::extract::Path<String>,
    ) -> impl IntoResponse {
        use diesel::prelude::*;
        
        let db = crate::get_pool();
        let mut conn = match db.get() {
            Ok(c) => c,
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e));
            }


        };        let actor = match actors::table
            .filter(actors::name.eq(&username))
            .first::<actors::Model>(&mut conn)
            .optional()
        {
            Ok(Some(a)) => a,
            Ok(None) => {
                return (StatusCode::NOT_FOUND, "User not found");
            }
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e));
            }
        };

        Json(actor)
    }
}