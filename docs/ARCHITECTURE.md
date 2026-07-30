# SolSynth 架构文档

## 📋 目录

- [系统架构](#系统架构)
- [核心服务](#核心服务)
- [数据流](#数据流)
- [部署架构](#部署架构)

## 
系统架构

SolSynth 采用微服务架构，每个服务独立部署、独立扩展。

### 架构图

```
                        ┌──────────────────────┐
                        │     Load Balancer    │
                        └──────────┬───────────┘
                                   │
                          ┌──────────────┴──────────────┐
                    │     API Gateway           │
                    │ (starfur-gateway)       │
                    └──────────────┬──────────────┘
                                   │
         ┌─────────────────────────┼─────────────────────────┐
         │                         │                         │
         ▼                         ▼                         ▼
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
│   Auth Service  │      │  Content Service │      │  Message Service│
│   (solsynth-    │      │   (solsynth-     │      │  (starfur-      │
│    auth)        │      │    sphere)       │      │   messager)     │
└────────┬────────┘      └────────┬────────┘      └────────┬────────┘
         │                         │                         │
                                  ▼                         ▼▼
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────    ┐
│ Storage        │      │   Mail          │      │ Marketplace │
│  (solsynth-     │      │  (solsynth-     │      │  (solsynth-    │
│   storage)      │      │    mail)        │      │   marketplace) │
└────────┬────────┘      └────────┬────────┘      └────────┬────────┘
         │                         │                         │
         ▼                         ▼                         ▼
┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
    │    Push         │      │     DB          │      │ gRPC        │
│  (solsynth-     │      │  (solsynth-     │      │  (solsynth-    │
│    push)        │      │    db)          │      │    grpc)        │
└─────────────────┘      └─────────────────┘      └─────────────────┘
```

## 核心服务

### 1. API Gateway (starfur-gateway)

统一入口，负责路由、限流、认证验证。

### 2. Auth Service (solsynth-auth)

用户认证与授权，支持：
- OIDC 协议
- WebAuthn (生物识别)
- JWT Token
- Argon2 密码哈希

### 3. Content Service (solsynth-sphere)

联邦内容服务，基于 ActivityPub 协议：
- 内容发布
- 内容订阅
- 联邦通信

### 4. Message Service (starfur-messager)

实时消息服务：
- WebSocket 连接
- NATS 消息队列
- MySQL 消息持久化

### 5. Storage Service (solsynth-storage)

文件存储服务：
- 文件上传/下载
- 端到端加密 (E2EE)
- 分片上传

### 6. Mail Service (solsynth-mail)

邮件服务：
- SMTP 发送邮件
- 邮件模板引擎
- 邮件队列

### 7. Marketplace (solsynth-marketplace)

市场服务：
- 商品管理
- 订单处理
- Stripe 支付集成

### 8. Push Service (solsynth-push)

推送服务：
- FCM (Firebase Cloud Messaging)
- WebSocket 实时推送
- 跨节点消息分发

## 数据流

### 用户认证流程

```
Client
                          → API Gateway → Auth Service ↓
                    (OIDC/WebAuthn)
                          ↓
                    JWT Token
```

### 消息发送流程

```
Client → WebSocket → Message Service
                          ↓
                    (NATS Broadcast)
                          ↓
                    MySQL Persistence
```

### 文件上传流程

```
Client → API Gateway → Storage Service
                          ↓
                    (E2EE Encrypt)
                          ↓
                   
                    MySQL Metadata File Data
```

## 部署架构

### 生产环境

```
┌──────────────────────────────────────────────────── ─┐
│                   Kubernetes Cluster                │
├─────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐         │
│  │ Gateway  │  │  App Pod │  │  App Pod │         │
│  └──────────┘  └──────────┘  └──────────┘         │
                                                           │ │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │ Auth Pod │  │Message Pod│  │Storage Pod│        │
│  └──────────┘  └──────────┘  └──────────┘        │
│                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │ Mail Pod │  │Market Pod │  │ Push Pod │        │
│  └──────────┘  └──────────┘  └──────────┘        │
│                                                     │
│  ┌──────────────────────────────────┐             │
│  │        MySQL (Primary/Replica)   │             │
│  └──────────────────────────────────┘             │
│                                                     │
│  ┌──────────────────────────────────┐             │
│  │          Redis Cluster           │             │
│  └──────────────────────────────────┘             │
│                                                     │
│  ┌──────────────────────────────────┐             │
│  │            NATS Cluster          │             │             
│  └──────────────────────────────────┘│
└─────────────────────────────────────────────────────┘
```

### 配置文件

主配置文件：`config/default.yaml`

```yaml
server:
  host: "0.0.0.0"
  port: 8080

database:
  host: "localhost"
  port: 3306
  username: "solsynth"
  password: ""
  database: "solsynth"

redis:
  url: "redis://localhost:6379"

nats:
  urls: ["nats://localhost:4222"]
```

## 📝 许可证

AGPL-3.0