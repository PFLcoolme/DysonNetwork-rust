-- 商品表
CREATE TABLE IF NOT EXISTS products (
    id CHAR(36) PRIMARY KEY COMMENT '商品ID',
    seller_id CHAR(36) NOT NULL COMMENT '卖家ID',
    name VARCHAR(255) NOT NULL COMMENT '商品名称',
    description TEXT NOT NULL COMMENT '商品描述',
    price DECIMAL(10, 2) NOT NULL COMMENT '商品价格',
    currency VARCHAR(3) DEFAULT 'USD' COMMENT '货币类型',
    product_type TINYINT NOT NULL DEFAULT 0 COMMENT '类型: 0-数字, 1-实体, 2-服务, 3-订阅',
    status TINYINT NOT NULL DEFAULT 0 COMMENT '状态: 0-草稿, 1-在售, 2-缺货, 3-禁用',
    stock_quantity INT NOT NULL DEFAULT 0 COMMENT '库存数量',
    digital_file_id CHAR(36) DEFAULT NULL COMMENT '数字文件ID',
    images TEXT DEFAULT NULL COMMENT '图片URL列表 (JSON数组)',
    tags TEXT DEFAULT NULL COMMENT '标签列表 (JSON数组)',
    is_featured BOOLEAN DEFAULT FALSE COMMENT '是否推荐',
    rating_avg DECIMAL(2, 1) DEFAULT 0.0 COMMENT '平均评分',
    rating_count INT DEFAULT 0 COMMENT '评分数量',
    sales_count INT DEFAULT 0 COMMENT '销售数量',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_seller (seller_id),
    INDEX idx_status (status),
    INDEX idx_type (product_type),
    INDEX idx_featured (is_featured),
    INDEX idx_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='商品表';

-- 订单表
CREATE TABLE IF NOT EXISTS orders (
    id CHAR(36) PRIMARY KEY COMMENT '订单ID',
    buyer_id CHAR(36) NOT NULL COMMENT '买家ID',
    seller_id CHAR(36) NOT NULL COMMENT '卖家ID',
    product_id CHAR(36) NOT NULL COMMENT '商品ID',
    quantity INT NOT NULL DEFAULT 1 COMMENT '购买数量',
    unit_price DECIMAL(10, 2) NOT NULL COMMENT '单价',
    total_amount DECIMAL(10, 2) NOT NULL COMMENT '总金额',
    currency VARCHAR(3) DEFAULT 'USD' COMMENT '货币类型',
    status TINYINT NOT NULL DEFAULT 0 COMMENT '状态: 0-待支付, 1-已支付, 2-处理中, 3-已完成, 4-已退款, 5-已取消',
    payment_method TINYINT NOT NULL DEFAULT 0 COMMENT '支付方式: 0-Stripe, 1-PayPal, 2-加密货币, 3-余额',
    payment_intent_id VARCHAR(255) DEFAULT NULL COMMENT '支付意图ID',
    shipping_address TEXT DEFAULT NULL COMMENT '收货地址',
    notes TEXT DEFAULT NULL COMMENT '备注',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_buyer (buyer_id),
    INDEX idx_seller (seller_id),
    INDEX idx_product (product_id),
    INDEX idx_status (status),
    INDEX idx_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='订单表';

-- 评价表
CREATE TABLE IF NOT EXISTS reviews (
    id CHAR(36) PRIMARY KEY COMMENT '评价ID',
    order_id CHAR(36) NOT NULL COMMENT '订单ID',
    buyer_id CHAR(36) NOT NULL COMMENT '买家ID',
    seller_id CHAR(36) NOT NULL COMMENT '卖家ID',
    product_id CHAR(36) NOT NULL COMMENT '商品ID',
    rating TINYINT NOT NULL COMMENT '评分 (1-5)',
    comment TEXT DEFAULT NULL COMMENT '评价内容',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    INDEX idx_order (order_id),
    INDEX idx_buyer (buyer_id),
    INDEX idx_seller (seller_id),
    INDEX idx_product (product_id),
    INDEX idx_rating (rating)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='评价表';