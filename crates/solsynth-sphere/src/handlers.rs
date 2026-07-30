//! HTTP 处理器 - WebFinger, NodeInfo, ActivityPub

use activitypub_federation::config::AuthorityExt;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::federation::{SphereConfig, SphereContext};

/// WebFinger 响应
#[derive(Debug, Serialize, Deserialize)]
struct WebFingerResponse {
    subject: String,
    aliases: Vec<String>,
    links: Vec<WebFingerLink>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebFingerLink {
    rel: String,
    type_: String,
    href: String,
}

/// NodeInfo 响应
#[derive(Debug, Serialize, Deserialize)]
struct NodeInfoResponse {
    #[serde(rename = "software")]
    software_info: NodeInfoSoftware,
    #[serde(rename = "protocols")]
    protocols: Vec<String>,
    #[serde(rename = "open_registrations")]
    open_registrations: bool,
    usage: NodeInfoUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeInfoSoftware {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeInfoUsage {
    users: NodeInfoUsers,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeInfoUsers {
    total: i32,
}

/// 健康检查
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// WebFinger 端点
pub async fn webfinger(
    State(context): State<SphereContext>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let resource = match query.get("resource") {
        Some(r) => r.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, "Missing resource parameter");
        }
    };

    let domain = context.config.domain.clone();
    
    // 解析 resource: "acct:user@domain"
    if resource.starts_with("acct:") {
        let acct = &resource[5..];
        if let Some(at_pos) = acct.find('@') {
            let username = &acct[..at_pos];
            let domain_part = &acct[at_pos + 1..];
            
            if domain_part == domain || domain_part == &domain {
                let response = WebFingerResponse {
                    subject: format!("acct:{}", username),
                    aliases: vec![format!("https://{}/u/{}", domain, username)],
                    links: vec![
                        WebFingerLink {
                            rel: "http://webfinger.net/rel/profile-page".to_string(),
                            type_: "text/html".to_string(),
                            href: format!("https://{}/u/{}", domain, username),
                        },
                        WebFingerLink {
                            rel: "self".to_string(),
                            type_: "application/activity+json".to_string(),
                            href: format!("https://{}/u/{}", domain, username),
                        },
                    ],
                };
                return (StatusCode::OK, Json(response));
            }
        }
    }

    (StatusCode::NOT_FOUND, "Not found")
}

/// NodeInfo 2.0 端点
pub async fn nodeinfo_2_0(
    State(context): State<SphereContext>,
) -> impl IntoResponse {
    let domain = context.config.domain.clone();
    
    let response = NodeInfoResponse {
        software_info: NodeInfoSoftware {
            name: "solsynth-sphere".to_string(),
            version: "0.1.0".to_string(),
        },
        protocols: vec!["activitypub".to_string()],
        open_registrations: context.config.enable_registration,
        usage: NodeInfoUsage {
            users: NodeInfoUsers { total: 0 },
        },
    };
    
    (StatusCode::OK, Json(response))
}

/// NodeInfo 发现端点
pub async fn nodeinfo_discovery() -> impl IntoResponse {
    serde_json::json!({
        "version": "2.1",
        "links": [{
            "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
            "href": "https://api.starfur.test/sphere/nodeinfo/2.0"
        }]
    })

}

/// ActivityPub Inboxpub async fn inbox(
    State(_context): State<SphereContext>,
    _request: axum::extract::Request,
) -> impl IntoResponse {
    // TODO: 实现 ActivityPub Inbox 处理
    (StatusCode::OK, "Inbox received")
}

/// ActivityPub Outbox
pub async fn outbox(
    State(_context): State<SphereContext>,
) -> impl IntoResponse {
    // TODO: 实现 ActivityPub Outbox
    serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": "https://api.starfur.test/sphere/outbox",
        "type": "Outbox",
        "first": "https://api.starfur.test/sphere/outbox?page=1"
    })
}

/// ActivityPub Shared Inbox
pub async fn shared_inbox(
    _request: axum::extract::Request,
) -> impl IntoResponse {
    // TODO: 实现 Shared Inbox
    (StatusCode::OK, "Shared inbox received")
}

/// 公开时间线
pub async fn public_timeline(
    State(context): State<SphereContext>,
) -> impl IntoResponse {
    // TODO: 实现联邦时间线
    serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": "https://api.starfur.test/sphere/public",
        "type": "OrderedCollection",
        "totalItems": 0,
        "first": "https://api.starfur.test/sphere/public?page=1"
    })
}

/// 配置路由
pub fn routes(context: SphereContext) -> axum::Router {
    use axum::routing::{get, post};
    
    axum::Router::new()
        // WebFinger
        .route("/.well-known/webfinger", get(webfinger))
        // NodeInfo
        .route("/.well-known/nodeinfo", get(nodeinfo_discovery))
        .route("/nodeinfo/2.0", get(nodeinfo_2_0))
        // ActivityPub
        .route("/inbox", post(inbox))
        .route("/outbox", get(outbox))
        .route("/shared-inbox", post(shared_inbox))
        .route("/public", get(public_timeline))
        // 健康检查
        .route("/health", get(health_check))
        .with_state(context)
}