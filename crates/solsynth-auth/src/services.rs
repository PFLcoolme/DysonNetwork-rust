//! 认证服务层

use crate::models::{Account, AuthChallenge, AuthSession, TokenResponse};
use async_trait::async_trait;
use uuid::Uuid;

/// 认证服务接口
#[async_trait]
pub trait AuthService: Send + Sync {
    /// 创建认证挑战
    async fn create_challenge(
        &self,
        account_id: Uuid,
        device_id: String,
        device_name: Option<String>,
        platform: String,
    ) -> Result<AuthChallenge, anyhow::Error>;

    /// 完成认证挑战
    async fn complete_challenge(
        &self,
        challenge_id: Uuid,
        password: Option<String>,
        passkey_assertion: Option<PasskeyAssertion>,
    ) -> Result<AuthSession, anyhow::Error>;

    /// 创建会话并签发 Token
    async fn create_session(
        &self,
        account_id: Uuid,
        device_id: String,
        platform: String,
    ) -> Result<TokenResponse, anyhow::Error>;

    /// 刷新 Token
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenResponse, anyhow::Error>;

    /// 撤销会话
    async fn revoke_session(&self, session_id: Uuid) -> Result<(), anyhow::Error>;

    /// 验证 Token
    async fn verify_token(&self, token: &str) -> Result<Account, anyhow::Error>;
}

/// Passkey 断言
pub struct PasskeyAssertion {
    pub credential_id: String,
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
}

/// OIDC 配置
pub struct OidcConfig {
    pub client_id: String,
    pub client_secret: String,
    pub issuer: String,
    pub scopes: Vec<String>,
}
