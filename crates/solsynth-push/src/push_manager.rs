//! 推送管理器
//!
//! 负责推送通知的调度、重试和状态管理

use dashmap::DashMap;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::models::{PushDevice, PushNotification, PushStats, PushStatus};

/// 推送管理器
#[derive(Clone)]
pub struct PushManager {
    notifications: Arc<DashMap<Uuid, PushNotification>>,
    devices: Arc<DashMap<Uuid, PushDevice>>,
    stats: Arc<PushManagerStats>,
}

struct PushManagerStats {
    pending: std::sync::atomic::AtomicU64,
    sending: std::sync::atomic::AtomicU64,
    sent: std::sync::atomic::AtomicU64,
    failed: std::sync::atomic::AtomicU64,
}

impl PushManager {
    /// 创建新的推送管理器
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(DashMap::new()),
            devices: Arc::new(DashMap::new()),
            stats: Arc::new(PushManagerStats {
                pending: std::sync::atomic::AtomicU64::new(0),
                sending: std::sync::atomic::AtomicU64::new(0),
                sent: std::sync::atomic::AtomicU64::new(0),
                failed: std::sync::atomic::AtomicU64::new(0),
            }),
        }
    }

    /// 注册推送设备
    pub fn register_device(&self, device: PushDevice) {
        info!("注册推送设备: {} (用户: {})", device.device_id, device.user_id);
        self.devices.insert(device.user_id, device);
    }

    /// 注销设备
    pub fn unregister_device(&self, user_id: Uuid) -> bool {
        if self.devices.remove(&user_id).is_some() {
            info!("注销推送设备: {}", user_id);
            true
        } else {
            false
        }
    }

    /// 获取用户设备
    pub fn get_user_device(&self, user_id: Uuid) -> Option<PushDevice> {
        self.devices.get(&user_id).map(|d| d.value().clone())
    }

    /// 发送推送通知
    pub fn send_notification(&self, mut notification: PushNotification) -> Uuid {
        notification.status = PushStatus::Pending;
        self.notifications.insert(notification.id, notification.clone());
        self.stats.pending.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        info!("推送通知已创建: {}", notification.id);
        notification.id
    }

    /// 获取待发送的推送
    pub fn dequeue_notification(&self) -> Option<(Uuid, PushNotification)> {
        for entry in self.notifications.iter() {
            if entry.value().status == PushStatus::Pending {
                let id = *entry.key();
                if let Some(mut notification) = self.notifications.get_mut(&id) {
                    notification.status = PushStatus::Sending;
                    self.stats.pending.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    self.stats.sending.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    return Some((id, notification.clone()));
                }
            }
        }
        None
    }

    /// 标记推送成功
    pub fn mark_sent(&self, notification_id: &Uuid) {
        if let Some(mut notification) = self.notifications.get_mut(notification_id) {
            notification.status = PushStatus::Sent;
            notification.sent_at = Some(chrono::Utc::now());
        }
        self.stats.sent.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.stats.sending.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        info!("推送发送成功: {}", notification_id);
    }

    /// 标记推送失败
    pub fn mark_failed(&self, notification_id: &Uuid, error_msg: Option<String>) {
        if let Some(mut notification) = self.notifications.get_mut(notification_id) {
            notification.status = PushStatus::Failed;
            notification.error_message = error_msg;
        }
        self.stats.failed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.stats.sending.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        error!("推送发送失败: {}", notification_id);
    }

    /// 获取推送统计
    pub fn get_stats(&self) -> PushStats {
        PushStats {
            total_pending: self.stats.pending.load(std::sync::atomic::Ordering::SeqCst),
            total_sending: self.stats.sending.load(std::sync::atomic::Ordering::SeqCst),
            total_sent: self.stats.sent.load(std::sync::atomic::Ordering::SeqCst),
            total_failed: self.stats.failed.load(std::sync::atomic::Ordering::SeqCst),
            total_devices: self.devices.len() as u64,
        }
    }
}

impl Default for PushManager {
    fn default() -> Self {
        Self::new()
    }
}