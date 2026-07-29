# SolSynth

基于 Rust 重构的 Solar Network (DysonNetwork) 服务端

## 项目结构

```
solsynth/
├──  Cargo.toml              # Workspace配置
├── config/                 # 配置文件
│   └── default.yaml
├── migrations/             # 数据库迁移
├──
 crates/
│   ├── solsynth            # 主程序│   ├── solsynth-auth       # 认证模块 (Padlock)
│   ├── solsynth-db         # 数据库模块
│   ├── solsynth-grpc       # gRPC 服务模块
│   └── solsynth-utils      # 通用工具
```

##
 技术栈

- **运行时**: Tokio- **Web 框架**: Axum
- **gRPC**: Tonic
- **数据库**: SQLx + MySQL
- **缓存**: Redis
- **消息队列**: NATS
- **认证**: argon2 + jsonwebtoken

## 服务模块

| 模块 | 功能 | 对应原服务 |
|------|------|-----------|
| solsynth-auth | 认证与授权 | Padlock |
| solsynth-db | 数据库连接与管理 | - |
| solsynth-grpc | gRPC 服务通信 | - |
| solsynth-utils | 通用工具 | - |

## 快速开始

```bash
# 克隆仓库
git clone https://cnb.cool/starfurr/starfur.git
cd starfur

# 构建项目
cargo build

# 运行测试
cargo test

# 运行
服务
cargo run```

## 配置

复制默认配置并根据需要修改：

```bash
cp config/default.yaml config/local.yaml
```

## 许可证

AGPL-3.0