-- 推送设备表
CREATE TABLE IF NOT EXISTS push_devices (
    id CHAR(36) PRIMARY KEY COMMENT '设备ID',
    user_id CHAR(36) NOT NULL COMMENT '用户ID',
    device_id VARCHAR(255) NOT NULL COMMENT '设备标识',
    platform VARCHAR(50) NOT NULL COMMENT '平台 (iOS/Android/Web)',
    push_token TEXT NOT NULL COMMENT '推送令牌',
    push_channel TINYINT NOT NULL DEFAULT 0 COMMENT '渠道: 0-FCM, 1-APNS, 2-WebSocket, 3-Web',
    app_version VARCHAR(50) DEFAULT NULL COMMENT '应用版本',
    is_active BOOLEAN DEFAULT TRUE COMMENT '是否激活',
    last_seen_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '最后在线时间',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_user (user_id),
    INDEX idx_device (device_id),
    INDEX idx_active (is_active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='推送设备表';

-- 推送通知表
CREATE TABLE IF NOT EXISTS push_notifications (
    id CHAR(36) PRIMARY KEY COMMENT '通知ID',
    title VARCHAR(255) NOT NULL COMMENT '通知标题',
    body TEXT NOT NULL COMMENT '通知内容',
    data JSON DEFAULT NULL COMMENT '附加数据',
    push_type TINYINT NOT NULL DEFAULT 0 COMMENT '类型: 0-通知, 1-消息, 2-系统, 3-营销',
    target_user_id CHAR(36) DEFAULT NULL COMMENT '目标用户ID',
    target_device_id CHAR(36) DEFAULT NULL COMMENT '目标设备ID',
    target_group VARCHAR(255) DEFAULT NULL COMMENT '目标群组',
    channel TINYINT NOT NULL DEFAULT 0 COMMENT '渠道: 0-FCM, 1-APNS, 2-WebSocket, 3-Web',
    status TINYINT NOT NULL DEFAULT 0 COMMENT '状态: 0-待发送, 1-发送中, 2-已发送, 3-失败',
    error_message TEXT DEFAULT NULL COMMENT '错误信息',
    sent_at TIMESTAMP NULL DEFAULT NULL COMMENT '发送时间',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_target_user (target_user_id),
    INDEX idx_status (status),
    INDEX idx_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='推送通知表';