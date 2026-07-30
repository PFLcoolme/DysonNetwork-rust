//! 消息相关实体和服务

use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, QueryOrder, QuerySelect, Set};
use serde::{Deserialize, Serialize};

// ==================== 消息实体 ====================

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub sender_id: String,
    pub receiver_id: String,
    pub message_type: i8,
    pub group_id: Option<String>,
    pub content: String,
    pub content_type: String,
    pub reply_to: Option<String>,
    pub attachments: Option<serde_json::Value>,
    pub status: i8,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

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
    db: DatabaseConnection,
}

impl MessageService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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

        let model = active_model.insert(&self.db).await?;
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
            .limit(Some(limit as u64))
            .all(&self.db)
            .await?;
        Ok(messages)
    }

    /// 更新消息状态
    pub async fn update_message_status(
        &self,
        message_id: &str,
        status: MessageStatus,
    ) -> anyhow::Result<()> {
        let model = Entity::find_by_id(message_id.to_string())
            .one(&self.db)
            .await?;

        match model {
            Some(m) => {
                let active_model: ActiveModel = m.into();
                let mut updated = active_model;
                updated.status = Set(status.to_i8());
                updated.update(&self.db).await?;
                Ok(())
            }
            None => Err(DbErr::RecordNotFound("Message not found".to_string()).into()),
        }
    }
}