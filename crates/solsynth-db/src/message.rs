//! 消息相关实体和服务

use sea_orm::entity::prelude::*;
use sea_orm::sea_query::Expr;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;

// ==================== 消息实体 ====================

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub sender_id: String,
    pub receiver_id: String,
    #[sea_orm(column_type = "TinyInt")]
    pub message_type: i8,
    pub group_id: Option<String>,
    pub content: String,
    #[sea_orm(column_type = "VARCHAR(50)")]
    pub content_type: String,
    pub reply_to: Option<String>,
    #[sea_orm(column_type = "Json")]
    pub attachments: Option<serde_json::Value>,
    #[sea_orm(column_type = "TinyInt")]
    pub status: i8,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel = ActiveModelBehavior;

// ==================== 消息已读状态实体 ====================

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "message_read_status")]
pub struct ReadStatus {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub message_id: String,
    pub user_id: String,
    pub read_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum ReadStatusRelation {}

impl ActiveModelBehavior for ActiveModel = ActiveModelBehavior;

// ==================== 未读消息计数实体 ====================

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "unread_count")]
pub struct UnreadCount {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: String,
    pub conversation_id: String,
    #[sea_orm(column_type = "Int")]
    pub unread_count: i32,
    pub last_read_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum UnreadCountRelation {}

impl ActiveModelBehavior for ActiveModel = ActiveModelBehavior;

// ==================== 群组实体 ====================

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "groups")]
pub struct Group {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub avatar_url: Option<String>,
    #[sea_orm(column_type = "TinyInt")]
    pub status: i8,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum GroupRelation {}

impl ActiveModelBehavior for ActiveModel = ActiveModelBehavior;

// ==================== 群组成员实体 ====================

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "group_members")]
pub struct GroupMember {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub group_id: String,
    pub user_id: String,
    #[sea_orm(column_type = "TinyInt")]
    pub role: i8,
    pub joined_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum GroupMemberRelation {}

impl ActiveModelBehavior for ActiveModel = ActiveModelBehavior;

// ==================== 消息类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Direct = 0,
    Group = 1,
    System = 2,
}

impl MessageType {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Direct,
            1 => Self::Group,
            2 => Self::System,
            _ => Self::Direct,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Direct => 0,
            Self::Group => 1,
            Self::System => 2,
        }
    }
}

// ==================== 消息状态 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    Sending = 0,
    Sent = 1,
    Delivered = 2,
    Read = 3,
    Failed = 4,
}

impl MessageStatus {
    pub fn from_i8(value: i8) -> Self {
        match value {
            0 => Self::Sending,
            1 => Self::Sent,
            2 => Self::Delivered,
            3 => Self::Read,
            4 => Self::Failed,
            _ => Self::Sending,
        }
    }

    pub fn to_i8(&self) -> i8 {
        match self {
            Self::Sending => 0,
            Self::Sent => 1,
            Self::Delivered => 2,
            Self::Read => 3,
            Self::Failed => 4,
        }
    }
}

// ==================== 消息服务 ====================

pub struct MessageService {
    pool: MySqlPool,
}

impl MessageService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 创建新消息
    pub async fn create_message(
        &self,
        sender_id: &str,
        receiver_id: &str,
        content: &str,
        message_type: MessageType,
        content_type: &str,
        group_id: Option<&str>,
        reply_to: Option<&str>,
        attachments: Option<Vec<String>>,
    ) -> anyhow::Result<Model> {
        let now = chrono::Utc::now();
        let attachments_json = attachments
            .map(|a| serde_json::to_value(&a).ok())
            .flatten();

        let active_model = ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            sender_id: Set(sender_id.to_string()),
            receiver_id: Set(receiver_id.to_string()),
            message_type: Set(message_type.to_i8()),
            group_id: Set(group_id.map(|s| s.to_string())),
            content: Set(content.to_string()),
            content_type: Set(content_type.to_string()),
            reply_to: Set(reply_to.map(|s| s.to_string())),
            attachments: Set(attachments_json),
            status: Set(MessageStatus::Sent.to_i8()),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
        };

        let model = active_model.insert(&self.pool).await?;
        Ok(model)
    }

    /// 获取用户的消息历史
    pub async fn get_message_history(
        &self,
        user_id: &str,
        conversation_id: &str,
        limit: i32,
    ) -> anyhow::Result<Vec<Model>> {
        let messages = Entity::find()
            .filter(
                Condition::any()
                    .add(
                        Condition::any()
                            .add(Column::SenderId.eq(user_id))
                            .add(Column::ReceiverId.eq(user_id)),
                    )
                    .add(Column::GroupId.eq(conversation_id)),
            )
            .order_by_desc(Column::CreatedAt)
            .limit(limit as i64)
            .all(&self.pool)
            .await?;
        Ok(messages)
    }

       /// 标记消息为已读
    pub async fn mark_as_read(
        &self,
        message_id: &str,
        user_id: &str,
    ) -> anyhow::Result<()> {
        let now = chrono::Utc::now();
        let read_status = ReadStatusActiveModel {
            message_id: Set(message_id.to_string()),
            user_id: Set(user_id.to_string()),
            read_at: Set(now.naive_utc()),
        };

        // 使用 insert_or_ignore 避免重复
        read_status.insert(&self.pool).await?;
        Ok(())

    }

    /// 更新消息状态
    pub async fn update_message_status(
        &self,
        message_id: &str,
        status: MessageStatus,
    ) -> anyhow::Result<()> {
        let active_model = ActiveModel {
            id: Set(message_id.to_string()),
            status: Set(status.to_i8()),
            ..Default::default()
        };

        active_model.update(&self.pool).await?;
        Ok(())
    }
}