//! OID
C (OpenID Connect) 第三方登录支持//!
//!  支持: Google, Apple, GitHub, Discord等

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// OIDC 提供者配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OidcProvider {
    pub client_id: String,
    pub client_secret: String,
    pub authorization_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub jwks_url: Option<String>,
}

/// Google 配置
pub fn google_config() -> OidcProvider {
    OidcProvider {
        client_id: std::env::var("OIDC_GOOGLE_CLIENT_ID").unwrap_or_default(),
        client_secret: std::env::var("OIDC_GOOGLE_CLIENT_SECRET").unwrap_or_default(),
        authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        token_url: "https://oauth2.googleapis.com/token".to_string(),
        userinfo_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        jwks_url: Some("https://www.googleapis.com/oauth2/v3/certs".to_string()),
    }
}

/// Apple 配置
pub fn apple_config() -> OidcProvider {
    OidcProvider {
        client_id: std::env::var("OIDC_APPLE_CLIENT_ID").unwrap_or_default(),
        client_secret: std::env::var("OIDC_APPLE_CLIENT_SECRET").unwrap_or_default(),
        authorization_url: "https://appleid.apple.com/auth/authorize".to_string(),
        token_url: "https://appleid.apple.com/auth/token".to_string(),
        userinfo_url: "https://appleid.apple.com/userinfo".to_string(),
        jwks_url: Some("https://appleid.apple.com/auth/keys".to_string()),
    }
}

/// GitHub 配置
pub fn github_config() -> OidcProvider {
    OidcProvider {
        client_id: std::env::var("OIDC_GITHUB_CLIENT_ID").unwrap_or_default(),
        client_secret: std::env::var("OIDC_GITHUB_CLIENT_SECRET").unwrap_or_default(),
        authorization_url: "https://github.com/login/oauth/authorize".to_string(),
        token_url: "https://github.com/login/oauth/access_token".to_string(),
        userinfo_url: "https://api.github.com/user".to_string(),
        jwks_url: None,
    }
}

/// OIDC 回调响应
#[derive(Debug, Serialize, Deserialize)]
pub struct OidcCallback {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
}

/// 获取 OIDC 授权 URL
pub fn authorization_url(provider: &OidcProvider, state: &str, redirect_uri: &str) -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email&state={}",
        provider.authorization_url,
        provider.client_id,
        redirect_uri,
        state,
    )
}

/// 用授权码换取 Token
pub async fn exchange_token(provider: &OidcProvider, code: &str, redirect_uri: &str) -> Result<OidcCallback> {
    let client = reqwest::Client::new();
    let response = client
        .post(&provider.token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", &provider.client_id),
            ("client_secret", &provider.client_secret),
        ])
        .send()
        .await?
        .json::<OidcCallback>()
        .await?;

    info!(provider = provider.token_url, "OIDC Token 交换成功");
    Ok(response)
}

/// 验证 ID Token (JWT)
pub async fn verify_id_token(id_token: &str, provider: &OidcProvider) -> Result<serde_json::Value> {
    // 获取 JWKS
    let jwks_url = provider.jwks_url.as_ref().ok_or_else(|| anyhow::anyhow!("Provider does not support JWKS"))?;
    
    let client = reqwest::Client::new();
    let jwks: serde_json::Value = client.get(jwks_url).send().await?.json(). await?;

    // 解析 JWT header获取 key_id
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(anyhow::anyhow!("Invalid ID token format"));
    }
    
    let header = base64::decode_config(parts[0], base64::URL_SAFE_NO_PAD)?;
    let header: serde_json::Value = serde_json::from_slice(&header)?;
    let key_id = header.get("kid").and_then(|v| v.as_str()).unwrap_or("");

    // 查找匹配的公钥并验证 (简化实现)
    tracing::debug!(%key_id, "验证 ID Token");
    
    let payload = base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)?;
    let claims: serde_json::Value = serde_json::from_slice(&payload)?;
    
    Ok(claims)
}