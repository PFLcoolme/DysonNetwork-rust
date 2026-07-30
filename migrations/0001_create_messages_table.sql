-- 消息表
CREATE TABLE IF NOT EXISTS messages (
    id CHAR(36) PRIMARY KEY COMMENT '消息ID (UUID)',
    sender_id CHAR(36) NOT NULL COMMENT '发送者ID',
    receiver_id CHAR(36) NOT NULL COMMENT '接收者ID',
    message_type TINYINT NOT NULL DEFAULT 0 COMMENT '消息类型: 0-私聊, 1-群聊, 2-系统通知',
    group_id CHAR(36) DEFAULT NULL COMMENT '群组ID (群聊时)',
    content TEXT NOT NULL COMMENT '消息内容',
    content_type VARCHAR(50) NOT NULL DEFAULT 'text' COMMENT '消息内容类型: text, image, video, file',
    reply_to CHAR(36) DEFAULT NULL COMMENT '回复的消息ID',
    attachments JSON DEFAULT NULL COMMENT '附件URL列表',
    status TINYINT NOT NULL DEFAULT 0 COMMENT '消息状态: 0-发送中, 1-已发送, 2-已送达, 3-已读, 4-发送失败',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_sender (sender_id),
    INDEX idx_receiver (receiver_id),
    INDEX idx_group (group_id),
    INDEX idx_created (created_at),
    INDEX idx_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='消息表';

-- 消息已读状态表
CREATE TABLE IF NOT EXISTS message_read_status (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    message_id CHAR(36) NOT NULL COMMENT '消息ID',
    user_id CHAR(36) NOT NULL COMMENT '已读者ID',
    read_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '已读时间',
    UNIQUE KEY unique_message_user (message_id, user_id),
    INDEX idx_message (message_id),
    INDEX idx_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='消息已读状态表';

-- 未读消息计数表
CREATE TABLE IF NOT EXISTS unread_count (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id CHAR(36) NOT NULL COMMENT '用户ID',
    conversation_id CHAR(36) NOT NULL COMMENT '对话ID (用户ID或群组ID)',
    unread_count INT NOT NULL DEFAULT 0 COMMENT '未读消息数',
    last_read_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '最后已读时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    UNIQUE KEY unique_user_conversation (user_id, conversation_id),
    INDEX idx_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='未读消息计数表';

-- 群组表
CREATE TABLE IF NOT EXISTS `groups` (
    id CHAR(36) PRIMARY KEY COMMENT '群组ID',
    name VARCHAR(100) NOT NULL COMMENT '群组名称',
    description TEXT DEFAULT NULL COMMENT '群组描述',
    owner_id CHAR(36) NOT NULL COMMENT '群主ID',
    avatar_url VARCHAR(500) DEFAULT NULL COMMENT '群组头像',
    status TINYINT NOT NULL DEFAULT 1 COMMENT '群组状态: 0-禁用, 1-正常',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_owner (owner_id),
    INDEX idx_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='群组表';

-- 群组成员表
CREATE TABLE IF NOT EXISTS group_members (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    group_id CHAR(36) NOT NULL COMMENT '群组ID',
    user_id CHAR(36) NOT NULL COMMENT '用户ID',
    role TINYINT NOT NULL DEFAULT 2 COMMENT '成员角色: 0-群主, 1-管理员, 2-普通成员',
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '加入时间',
    UNIQUE KEY unique_group_user (group_id, user_id),
    INDEX idx_group (group_id),
    INDEX idx_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='群组成员表';