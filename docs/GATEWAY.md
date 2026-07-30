# Starfur Gateway

`starfur-gateway` 是 DysonNetwork 向 Rust 逐步迁移期间的兼容入口。它保留现有客户端 URL，并允许各上游服务独立切换到新实现。

## 能力

- 流式 HTTP 反向代理，不缓冲请求体或响应体
- `/{service}/...` 到上游 `/api/...` 的路径重写
- OIDC、WebAuthn、ActivityPub、WebFinger 和 TUS 固定路径透传
- `/pass`、`/drive`、`/id` 旧路径兼容
- WebSocket HTTP Upgrade 双向透传
- 按客户端 IP 的固定窗口限流
- 存活检查、就绪检查、JSON 日志与优雅停机

## 运行

```bash
cargo run --package starfur-gateway
```

默认监听 `0.0.0.0:8080`。配置项见仓库根目录 `.env.example`；程序读取环境变量，不会自动加载 `.env`。

```bash
curl http://127.0.0.1:8080/health/ready
```

## 路由

| 外部路径 | 上游 | 上游路径 |
| --- | --- | --- |
| `/padlock/*` | Padlock | `/api/*` |
| `/passport/*`、`/pass/*` | Passport | `/api/*` |
| `/sphere/*` | Sphere | `/api/*` |
| `/messager/*` | Messager | `/api/*` |
| `/ring/*` | Ring | `/api/*` |
| `/wallet/*` | Wallet | `/api/*` |
| `/develop/*` | Develop | `/api/*` |
| `/storage/*`、`/drive/*` | Storage | `/api/*` |
| `/push/*`、`/mail/*`、`/workspace/*` | 对应新服务 | `/api/*` |
| `/.well-known/openid-configuration` 等 | Padlock | 原路径 |
| `/.well-known/webfinger`、`/activitypub/*` | Sphere | 原路径 |
| `/api/tus/*` | Storage | 原路径 |
| `/ws` | WebSocket upstream | 原路径 |

上游仅接受内部 `http://` 地址。若公网 TLS 在 Nginx、Caddy 或负载均衡器终止，把 `STARFUR_EXTERNAL_SCHEME` 设置为 `https`。
