//! Actor 数据模型

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Insertable, Identifiable, AsChangeset, Deserialize, Serialize)]
#[diesel(table_name = crate::db::schema::actors)]
pub struct ActorModel {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub summary: Option<String>,
    pub public_key_pem: String,
    pub private_key_pem: String,
    pub inbox_url: String,
    pub outbox_url: String,
    pub shared_inbox_url: Option<String>,
    pub endpoint_url: String,
    pub actor_type: String,
    pub is_instance: bool,
    pub avatar_url: Option<String>,
    pub header_image_url: Option<String>,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}