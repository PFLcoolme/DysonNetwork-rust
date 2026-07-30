//! Activity 数据模型

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Insertable, Identifiable, AsChangeset, Deserialize, Serialize)]
#[diesel(table_name = crate::db::schema::activities)]
pub struct ActivityModel {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub activity_type: String,
    pub raw_json: String,
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub to_vec: Vec<String>,
    pub cc_vec: Vec<String>,
    pub in_reply_to: Option<Uuid>,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Identifiable, AsChangeset, Deserialize, Serialize)]
#[diesel(table_name = crate::db::schema::posts)]
pub struct PostModel {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub content_type: String,
    pub visibility: String,
    pub in_reply_to: Option<Uuid>,
    pub repost_of: Option<Uuid>,
    pub url: String,
    pub ap_id: String,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}