# SolSynth

 基于 Rust 重构的 DysonNetwork 服务端

##📋 项目概述

SolSynth 是对 [DysonNetwork](https://github.com/Solsynth/DysonNetwork) 的全面 Rust 重构，采用模块化架构设计，提供完整的 后端服务套件。

##🏗️ 项目结构

```
starfur/
├── Cargo.tom l              # Workspace配置
├── config/                 # 配置文件
│    └──default.yaml
├── migrations/             # 数据库迁移
│   ├── 0001_create_messages_table.sql
│   ├── 0002_create_storage_table.sql
│   ├── 0003_create_mail_table.sql
│   ├── 0004_create_marketplace_tables.sql
│   └── 0005_create_push_tables.sql
└── crates/
    ├── solsynth            # 主程序 (服务编排)
    ├── solsynth-auth       # Padlock 认证模块
    ├── solsynth-db         # 数据库模块
    ├── solsynth-grpc       # gRPC 服务模块
    ├── solsynth-utils      # 通用工具
    ├── solsynth-sphere     # Sphere 联邦内容服务
    ├── solsynth-storage    # 文件存储服务
    ├── solsynth-mail       # 邮件服务
    ├── solsynth-marketplace # 市场服务
    ├── solsynth-push       # 推送服务
    ├── star
    fur-gateway     # API 网关└── starfur-messager    # 消息服务 
```

##🛠️ 技术栈

| 类别 | 技术 |
|------|------|
| **运行时** | Tokio |
| **Web 框架** | Axum |
| **gRPC** | Tonic |
| **数据库** | SQLx + MySQL |
| **缓存** | Redis |
| **消息队列** | NATS |
| **认证** | argon2 + jsonwebtoken + WebAuthn |
| **推送** | FCM (Firebase Cloud Messaging) |
| **支付** | Stripe |

## 📦 服务模块

| 模块 | 功能 | 对应原服务 |
|------|------|-----------|
| solsynth-auth | 认证与授权 (OIDC, WebAuthn, JWT) | Padlock |
| solsynth-sphere | 联邦内容服务 (ActivityPub) | - |
| starfur-messager | 消息系统 (WebSocket, NATS) | DysonChat |
| solsynth-storage | 文件存储 (上传/下载, E2EE) | DysonFS |
| solsynth-mail | 邮件服务 (SMTP, 模板引擎) | ElecPostal |
| solsynth-marketplace | 市场服务 (商品、订单、支付) | Matrix.Nucleus |
| solsynth-push | 推送服务 (FCM, WebSocket) | DysonNoti |
| solsynth-db | 数据库连接与管理 | - |
| solsynth-grpc | gRPC 服务通信 | - |
| solsynth-utils | 通用工具 | - |
| starfur-gateway | API 网关 | - |

## 🚀 快速开始

### 前置要求

- Rust 1.70+
- MySQL 8.0+
- Redis 7.0+
- NATS Server

### 安装与运行

```bash
# 克隆仓库
git clone https://cnb.cool/starfurr/starfur.git
cd starfur

# 构建项目
cargo build

# 运行测试
cargo test

# 运行服务
cargo run -p solsynth

# 运行 API Gateway
cargo run -p starfur-gateway
```

### 配置

复制默认配置并根据需要修改：

```bash
cp config/default.yaml config/local.yaml
```

### 数据库迁移

```bash
# 运行所有迁移
cargo run -p solsynth-db migrate

# 回滚最后一次迁移
cargo run -p solsynth-db migrate-down
```

##  📚文档

- [API Gateway 文档](docs/GATEWAY.md)
- [认证模块文档](docs/AUTH.md)
- [存储服务文档](docs/STORAGE.md)
- [消息服务文档](docs/MESSAGER.md)
- [市场服务文档](docs/MARKETPLACE.md)
- [推送服务文档](docs/PUSH.md)

 ## 📊架构图

```
                    ┌─────────────────┐
                    │   API Gateway   │
                    │ (
                    starfur-gateway)│└────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
               │                    │                    │
  ▼                    ▼                   ▼
┌────────────  ───┐ ┌───────────────┐   ┌───────────────┐
│   Auth        │   │   Sphere      │   │  Messager     │
│   (Padlock)   │   │ (Federated)   │   │  (Chat )      │
└───────────────┘   └───────────────┘   └───────────────┘
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐   ┌───────────────┐   ┌──────── ───────┐
│   Storage     │   │   Mail        │   │ Marketplace  │
│   (DysonFS)   │   │  (Email)      │   │  (Market)     │
└───────────────┘   └───────────────┘   └───────────────┘
        │                    │                    │
                            ▼                    ▼▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│       Push       │   │    DB         │   │ gRPC       │
│  (Noti)       │   │  (MySQL)      │   │  (RPC)        │
└───────────────┘   └───────────────┘   └──────────── ───┘
```

##🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定 crate 测试
cargo test -p solsynth-auth

# 运行集成测试
cargo test --test integration
```

## 📝 许可证

AGPL-3.0

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📧 联系方式

- Repository: https://cnb.cool/starfurr/starfur