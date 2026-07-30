-- 文件存储表
CREATE TABLE IF NOT EXISTS files (
    id CHAR(36) PRIMARY KEY COMMENT '文件ID',
    owner_id CHAR(36) NOT NULL COMMENT '文件所有者ID',
    name VARCHAR(255) NOT NULL COMMENT '文件名',
    size BIGINT NOT NULL COMMENT '文件大小 (字节)',
    mime_type VARCHAR(100) DEFAULT NULL COMMENT 'MIME类型',
    file_type TINYINT NOT NULL DEFAULT 99 COMMENT '文件类型: 0-图片, 1-视频, 2-音频, 3-文档, 4-压缩包, 99-其他',
    storage_path VARCHAR(500) NOT NULL COMMENT '存储路径',
    encryption_key_id VARCHAR(100) DEFAULT NULL COMMENT '加密密钥ID',
    checksum VARCHAR(64) DEFAULT NULL COMMENT 'SHA-256校验和',
    folder VARCHAR(200) DEFAULT NULL COMMENT '所属文件夹',
    status TINYINT NOT NULL DEFAULT 1 COMMENT '状态: 0-上传中, 1-已上传, 2-已删除',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_owner (owner_id),
    INDEX idx_status (status),
    INDEX idx_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='文件存储表';

-- 存储空间配额表
CREATE TABLE IF NOT EXISTS storage_quota (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id CHAR(36) NOT NULL COMMENT '用户ID',
    total_space BIGINT NOT NULL DEFAULT 10737418240 COMMENT '总空间 (字节), 默认10GB',
    used_space BIGINT NOT NULL DEFAULT 0 COMMENT '已用空间 (字节)',
    file_count INT NOT NULL DEFAULT 0 COMMENT '文件数量',
    max_file_size BIGINT NOT NULL DEFAULT 524288000 COMMENT '最大文件大小 (字节), 默认500MB',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    UNIQUE KEY unique_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='存储空间配额表';

-- 文件分享表
CREATE TABLE IF NOT EXISTS file_shares (
    id CHAR(36) PRIMARY KEY COMMENT '分享ID',
    file_id CHAR(36) NOT NULL COMMENT '文件ID',
    owner_id CHAR(36) NOT NULL COMMENT '文件所有者',
    share_code VARCHAR(20) NOT NULL COMMENT '分享码',
    password VARCHAR(50) DEFAULT NULL COMMENT '提取码 (可选)',
    expire_at TIMESTAMP NULL DEFAULT NULL COMMENT '过期时间',
    download_count INT NOT NULL DEFAULT 0 COMMENT '下载次数',
    max_downloads INT DEFAULT NULL COMMENT '最大下载次数 (NULL表示无限制)',
    status TINYINT NOT NULL DEFAULT 1 COMMENT '状态: 0-已取消, 1-有效',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    INDEX idx_share_code (share_code),
    INDEX idx_file (file_id),
    INDEX idx_owner (owner_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='文件分享表';