//! 消息系统数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::DateTime;

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 私聊消息
    Direct,
    /// 群组消息
    Group,
    /// 系统通知
    System,
}

impl Default for MessageType {
    fn default() -> Self {
        Self::Direct
    }
}

/// 消息状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    /// 发送中
    Sending,
    /// 已发送
    Sent,
    /// 已送达
    Delivered,
    /// 已读
    Read,
    /// 发送失败
    Failed,
}

impl Default for MessageStatus {
    fn default() -> Self {
        Self::Sending
    }
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息 ID
    pub id: Uuid,
    /// 发送者 ID
    pub sender_id: Uuid,
    /// 接收者 ID (私聊) 或群组 ID (群聊)
    pub receiver_id: Uuid,
    /// 消息类型
    #[serde(default)]
    pub message_type: MessageType,
    /// 消息内容
    pub content: String,
    /// 消息类型 (text, image, video, file 等)
    #[serde(default = "default_content_type")]
    pub content_type: String,
    /// 消息状态
    #[serde(default)]
    pub status: MessageStatus,
    /// 创建时间
    pub created_at: DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: DateTime<chrono::Utc>,
    /// 回复的消息 ID (可选)
    pub reply_to: Option<Uuid>,
    /// 附件 URL 列表
    #[serde(default)]
    pub attachments: Vec<String>,
}

/// 消息请求 (用于创建消息)
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMessageRequest {
    /// 接收者 ID
    pub receiver_id: Uuid,
    /// 消息内容
    pub content: String,
    /// 消息类型
    #[serde(default = "default_message_type")]
    pub message_type: MessageType,
    /// 消息内容类型
    #[serde(default = "default_content_type")]
    pub content_type: String,
    /// 回复的消息 ID
    pub reply_to: Option<Uuid>,
    /// 附件 URL 列表
    #[serde(default)]
    pub attachments: Vec<String>,
}

/// 群组信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// 群组 ID
    pub id: Uuid,
    /// 群组名称
    pub name: String,
    /// 群组描述
    pub description: String,
    /// 群主 ID
    pub owner_id: Uuid,
    /// 成员列表
    #[serde(default)]
    pub members: Vec<Uuid>,
    /// 创建时间
    pub created_at: DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: DateTime<chrono::Utc>,
}

/// 消息已读状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReadStatus {
    /// 消息 ID
    pub message_id: Uuid,
    /// 已读者 ID 列表
    #[serde(default)]
    pub read_by: Vec<Uuid>,
    /// 已读数量
    #[serde(default)]
    pub read_count: i32,
    /// 总数量
    #[serde(default)]
    pub total_count: i32,
}

/// 未读消息计数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadCount {
    /// 用户 ID
    pub user_id: Uuid,
    /// 未读消息总数
    pub total_unread: i32,
    /// 按对话分组的未读数
    #[serde(default)]
    pub conversations: Vec<ConversationUnread>,
}

/// 单个对话的未读数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationUnread {
    /// 对话 ID (用户 ID 或群组 ID)
    pub conversation_id: Uuid,
    /// 未读消息数
    pub unread_count: i32,
}

/// WebSocket 消息帧
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsFrame {
    /// 消息帧
    Message {
        message: Message,
    },
    /// 已读回执
    ReadReceipt {
        message_id: Uuid,
        user_id: Uuid,
    },
    /// 打字指示器
    Typing {
        user_id: Uuid,
        conversation_id: Uuid,
    },
    /// 连接确认
    Connected {
        user_id: Uuid,
    },
    /// 错误
    Error {
        code: String,
        message: String,
    },
}

fn default_message_type() -> MessageType {
    MessageType::Direct
}

fn default_content_type() -> String {
    "text".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = Message {
            id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            receiver_id: Uuid::new_v4(),
            message_type: MessageType::Direct,
            content: "Hello, World!".to_string(),
            content_type: "text".to_string(),
            status: MessageStatus::Sent,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            reply_to: None,
            attachments: vec![],
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content, "Hello, World!");
    }
}