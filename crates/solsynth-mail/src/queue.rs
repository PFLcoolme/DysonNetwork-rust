//! 邮件队列管理
//!
//! 负责邮件队列的优先级管理和发送调度

use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::models::{MailMessage, MailStatus, MailStats, QueueItem};

/// 邮件队列管理器
#[derive(Clone)]
pub struct MailQueue {
    queue: Arc<DashMap<Uuid, QueueItem>>,
    stats: Arc<MailQueueStats>,
}

struct MailQueueStats {
    queued: std::sync::atomic::AtomicU64,
    sending: std::sync::atomic::AtomicU64,
    sent: std::sync::atomic::AtomicU64,
    failed: std::sync::atomic::AtomicU64,
}

impl MailQueue {
    /// 创建新的邮件队列
    pub fn new() -> Self {
        Self {
            queue: Arc::new(DashMap::new()),
            stats: Arc::new(MailQueueStats {
                queued: std::sync::atomic::AtomicU64::new(0),
                sending: std::sync::atomic::AtomicU64::new(0),
                sent: std::sync::atomic::AtomicU64::new(0),
                failed: std::sync::atomic::AtomicU64::new(0),
            }),
        }
    }

    /// 添加邮件到队列
    pub fn enqueue(&self, mut mail_message: MailMessage) -> Uuid {
        let mail_id = mail_message.id;
        let priority = match mail_message.mail_type {
            crate::models::MailType::PasswordReset => 10,
            crate::models::MailType::EmailVerification => 9,
            crate::models::MailType::Notification => 5,
            crate::models::MailType::Marketing => 1,
            crate::models::MailType::System => 8,
        };

        mail_message.status = MailStatus::Queued;
        let queue_item = QueueItem {
            mail_id,
            priority,
            scheduled_at: None,
            mail_message,
        };

        self.queue.insert(mail_id, queue_item);
        self.stats.queued.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        info!("邮件已加入队列: {}", mail_id);
        mail_id
    }

    /// 获取下一个要发送的邮件
    pub fn dequeue(&self) -> Option<(Uuid, MailMessage)> {
        // 找到优先级最高的邮件
        let max_priority = self.queue.iter().max_by_key(|item| item.value().priority);

        if let Some(item) = max_priority {
            let mail_id = item.key().clone();
            if let Some(entry) = self.queue.remove(&mail_id) {
                let mut mail = entry.1.mail_message;
                mail.status = MailStatus::Sending;
                self.stats.sending.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Some((mail_id, mail))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 标记邮件发送成功
    pub fn mark_sent(&self, mail_id: &Uuid) {
        if let Some(mut item) = self.queue.get_mut(mail_id) {
            item.mail_message.status = MailStatus::Sent;
            item.mail_message.sent_at = Some(chrono::Utc::now());
        }
        self.stats.sent.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.stats.sending.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        info!("邮件发送成功: {}", mail_id);
    }

    /// 标记邮件发送失败
    pub fn mark_failed(&self, mail_id: &Uuid, error_msg: Option<String>) {
        if let Some(mut item) = self.queue.get_mut(mail_id) {
            item.mail_message.status = MailStatus::Failed;
            item.mail_message.error_message = error_msg;
            item.mail_message.retry_count += 1;
        }
        self.stats.failed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.stats.sending.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        error!("邮件发送失败: {}", mail_id);
    }

    /// 重试失败的邮件
    pub fn retry_failed(&self, mail_id: &Uuid, max_retries: i32) -> bool {
        if let Some(mut item) = self.queue.get_mut(mail_id) {
            if item.mail_message.retry_count < max_retries {
                item.mail_message.status = MailStatus::Queued;
                info!("重试邮件: {} (第 {} 次)", mail_id, item.mail_message.retry_count + 1);
                true
            } else {
                warn!("邮件达到最大重试次数: {}", mail_id);
                false
            }
        } else {
            false
        }
    }

    /// 获取队列统计
    pub fn get_stats(&self) -> MailStats {
        MailStats {
            total_queued: self.stats.queued.load(std::sync::atomic::Ordering::SeqCst),
            total_sending: self.stats.sending.load(std::sync::atomic::Ordering::SeqCst),
            total_sent: self.stats.sent.load(std::sync::atomic::Ordering::SeqCst),
            total_failed: self.stats.failed.load(std::sync::atomic::Ordering::SeqCst),
        }
    }

    /// 获取队列大小
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for MailQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// 后台邮件发送任务
pub async fn process_queue(queue: MailQueue, send_fn: impl Fn(MailMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        while let Some((mail_id, mail)) = queue.dequeue() {
            match send_fn(mail).await {
                Ok(_) => queue.mark_sent(&mail_id),
                Err(e) => queue.mark_failed(&mail_id, Some(e.to_string())),
            }
        }
    }
}