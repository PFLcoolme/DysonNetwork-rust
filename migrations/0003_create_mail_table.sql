-- 邮件表
CREATE TABLE IF NOT EXISTS mail_queue (
    id CHAR(36) PRIMARY KEY COMMENT '邮件ID',
    from_address VARCHAR(255) NOT NULL COMMENT '发件人地址',
    to_addresses TEXT NOT NULL COMMENT '收件人列表 (JSON数组)',
    cc_addresses TEXT DEFAULT NULL COMMENT '抄送列表 (JSON数组)',
    bcc_addresses TEXT DEFAULT NULL COMMENT '密送列表 (JSON数组)',
    subject VARCHAR(500) NOT NULL COMMENT '邮件主题',
    html_body LONGTEXT NOT NULL COMMENT 'HTML 邮件内容',
    text_body TEXT DEFAULT NULL COMMENT '纯文本邮件内容',
    attachments TEXT DEFAULT NULL COMMENT '附件列表 (JSON数组)',
    mail_type TINYINT NOT NULL DEFAULT 99 COMMENT '邮件类型: 0-密码重置, 1-邮箱验证, 2-通知, 3-营销, 99-系统',
    template_name VARCHAR(100) DEFAULT NULL COMMENT '模板名称',
    template_vars JSON DEFAULT NULL COMMENT '模板变量',
    status TINYINT NOT NULL DEFAULT 0 COMMENT '状态: 0-排队中, 1-发送中, 2-已发送, 3-发送失败',
    retry_count INT NOT NULL DEFAULT 0 COMMENT '重试次数',
    error_message TEXT DEFAULT NULL COMMENT '错误信息',
    sent_at TIMESTAMP NULL DEFAULT NULL COMMENT '发送时间',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_status (status),
    INDEX idx_type (mail_type),
    INDEX idx_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='邮件队列表';

-- 邮件发送日志表
CREATE TABLE IF NOT EXISTS mail_send_log (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    mail_id CHAR(36) NOT NULL COMMENT '邮件ID',
    action VARCHAR(50) NOT NULL COMMENT '操作: queued, sending, sent, failed',
    details TEXT DEFAULT NULL COMMENT '详细信息',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    INDEX idx_mail_id (mail_id),
    INDEX idx_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='邮件发送日志表';