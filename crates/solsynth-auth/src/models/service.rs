//! 认证服务层

use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use jsonwebtoken as jwt;
use password_hash::{PasswordHasher, PasswordHash, SaltString, PasswordVerifier};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, warn};

use super::entity::{accounts, account_secrets, account_auth_factor, auth_session, auth_challenge};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

/// JWT 配置
#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub secret: String,
    pub access_expiry: TimeDelta,
    pub refresh_expiry: TimeDelta,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("AUTH_JWT_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string()),
            access_expiry: TimeDelta::seconds(3600),
            refresh_expiry: TimeDelta::seconds(604800),
        }
    }
}

/// Token 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub token_type: String,
}

/// 密码哈希参数
const PASSWORD_HASH_COST: u32 = 3;
const MEMORY_COST_KB: u32 = 64 * 1024;
const PARALLELISM: u32 = 4;

/// 认证服务实现
#[derive(Clone)]
pub struct AuthService {
    pub db: sea_orm::DatabaseConnection,
    pub jwt_config: JwtConfig,
}

/// 认证状态（用于 axum State）
#[derive(Clone)]
pub struct AuthState {
    pub auth_service: AuthService,
    pub cookie_name: String,
}

impl AuthService {
    pub fn new(db: sea_orm::DatabaseConnection, jwt_config: JwtConfig) -> Self {
        Self { db, jwt_config }
    }

    fn argon2_instance() -> argon2::Argon2<'static> {
        let params = argon2::Params::new(
            MEMORY_COST_KB,
            PARALLELISM,
            PASSWORD_HASH_COST,
            None,
        ).unwrap();
        argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x10,
            params,
        )
    }

    /// 注册新用户
    pub async fn register(&self, name: &str, password: &str) -> Result<Uuid, AuthError> {
        let existing = accounts::Entity::find()
            .filter(accounts::Column::Name.eq(name))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AuthError::AlreadyExists("username".to_string()));
        }

        let salt = SaltString::generate(&mut rand::thread_rng());
        let argon2 = Self::argon2_instance();
        let hashed_password = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::Database(sea_orm::error::DbErr::Custom(
                format!("Failed to hash password: {}", e)
            )))?
            .to_string();

        let account_id = Uuid::now_v7();
        let now = Utc::now();

        let account = accounts::ActiveModel {
            id: sea_orm::ActiveValue::Set(account_id),
            name: sea_orm::ActiveValue::Set(name.to_string()),
            nick: sea_orm::ActiveValue::Set(format!("User-{}", &name[..4.min(name.len())])),
            language: sea_orm::ActiveValue::Set("zh-Hans".to_string()),
            region: sea_orm::ActiveValue::Set("CN".to_string()),
            is_superuser: sea_orm::ActiveValue::Set(false),
            activated_at: sea_orm::ActiveValue::Set(Some(now.naive_utc())),
            created_at: sea_orm::ActiveValue::Set(now.naive_utc()),
            updated_at: sea_orm::ActiveValue::Set(now.naive_utc()),
        };
        account.insert(&self.db).await?;

        let secret = account_secrets::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::now_v7()),
            account_id: sea_orm::ActiveValue::Set(account_id),
            secret_type: sea_orm::ActiveValue::Set("password".to_string()),
            secret_value: sea_orm::ActiveValue::Set(hashed_password),
            enabled_at: sea_orm::ActiveValue::Set(Some(now.naive_utc())),
            created_at: sea_orm::ActiveValue::Set(now.naive_utc()),
        };
        secret.insert(&self.db).await?;

        info!(%account_id, %name, "用户注册成功");
        Ok(account_id)
    }

    /// 验证密码并返回账户 ID
    pub async fn verify_password(&self, name: &str, password: &str) -> Result<Uuid, AuthError> {
        let account = accounts::Entity::find()
            .filter(accounts::Column::Name.eq(name))
            .one(&self.db)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        if account.activated_at.is_none() {
            return Err(AuthError::AccountNotActivated);
        }

        let secret = account_secrets::Entity::find()
            .filter(account_secrets::Column::AccountId.eq(account.id))
            .filter(account_secrets::Column::SecretType.eq("password"))
            .one(&self.db)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let stored_hash = PasswordHash::new(&secret.secret_value)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let argon2 = Self::argon2_instance();
        argon2
            .verify_password(password.as_bytes(), &stored_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        Ok(account.id)
    }

    /// 创建认证会话并签发 Token
    pub async fn create_session(
        &self,
        account_id: Uuid,
        device_id: String,
        device_name: Option<String>,
        platform: &str,
    ) -> Result<TokenResponse, AuthError> {
        let now = Utc::now();
        let access_expires = now + self.jwt_config.access_expiry;
        let refresh_expires = now + self.jwt_config.refresh_expiry;

        let access_token = self.issue_token(account_id, true, &access_expires)?;
        let refresh_token = self.issue_token(account_id, false, &refresh_expires)?;

        let session = auth_session::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::now_v7()),
            account_id: sea_orm::ActiveValue::Set(account_id),
            device_id: sea_orm::ActiveValue::Set(device_id),
            device_name: sea_orm::ActiveValue::Set(device_name),
            platform: sea_orm::ActiveValue::Set(platform.to_string()),
            access_token: sea_orm::ActiveValue::Set(access_token.clone()),
            access_expires_at: sea_orm::ActiveValue::Set(access_expires.naive_utc()),
            refresh_token: sea_orm::ActiveValue::Set(refresh_token.clone()),
            refresh_expires_at: sea_orm::ActiveValue::Set(refresh_expires.naive_utc()),
            created_at: sea_orm::ActiveValue::Set(now.naive_utc()),
            expired_at: sea_orm::ActiveValue::Set(refresh_expires.naive_utc()),
        };
        session.insert(&self.db).await?;

        info!(%account_id, "会话创建成功");

        Ok(TokenResponse {
            token: access_token,
            refresh_token,
            expires_in: self.jwt_config.access_expiry.num_seconds(),
            refresh_expires_in: self.jwt_config.refresh_expiry.num_seconds(),
            token_type: "Bearer".to_string(),
        })
    }

    /// 签发 JWT Token
    fn issue_token(&self, account_id: Uuid, is_access: bool, expires_at: &DateTime<Utc>) -> Result<String, AuthError> {
        let claims = TokenClaims {
            sub: account_id.to_string(),
            iss: "solsynth-auth".to_string(),
            exp: expires_at.timestamp(),
            nbf: Utc::now().timestamp(),
            iat: Utc::now().timestamp(),
            jti: Uuid::now_v7().to_string(),
            token_type: if is_access { "access" } else { "refresh" }.to_string(),
        };

        jwt::encode(
            &jwt::Header::default(),
            &claims,
            &jwt::EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        ).map_err(AuthError::TokenError)
    }

    /// 验证并解码 JWT Token
    pub fn verify_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        jwt::decode::<TokenClaims>(
            token,
            &jwt::DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &jwt::Validation::default(),
        ).map_err(AuthError::TokenError).map(|d| d.claims)
    }

    /// 刷新 Token
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenResponse, AuthError> {
        let claims = self.verify_token(refresh_token)?;

        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidTokenType);
        }

        let account_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

        let account = accounts::Entity::find()
            .filter(accounts::Column::Id.eq(account_id))
            .one(&self.db)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        if account.activated_at.is_none() {
            return Err(AuthError::AccountNotActivated);
        }

        self.create_session(account_id, claims.jti, None, "refresh").await
    }

    /// 撤销会话
    pub async fn revoke_session(&self, session_id: Uuid) -> Result<(), AuthError> {
        let session = auth_session::Entity::find()
            .filter(auth_session::Column::Id.eq(session_id))
            .one(&self.db)
            .await?
            .ok_or(AuthError::SessionNotFound)?;

        let session_active: auth_session::ActiveModel = session.into();
        session_active.delete(&self.db).await?;
        Ok(())
    }

    /// 创建认证挑战
    pub async fn create_challenge(
        &self,
        account_id: Uuid,
        device_id: String,
        platform: &str,
    ) -> Result<auth_challenge::Model, AuthError> {
        let now = Utc::now();
        let expired_at = now + TimeDelta::minutes(5);

        let _account = accounts::Entity::find()
            .filter(accounts::Column::Id.eq(account_id))
            .one(&self.db)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        let challenge = auth_challenge::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::now_v7()),
            account_id: sea_orm::ActiveValue::Set(account_id),
            device_id: sea_orm::ActiveValue::Set(device_id),
            device_name: sea_orm::ActiveValue::Set(None),
            platform: sea_orm::ActiveValue::Set(platform.to_string()),
            ip_address: sea_orm::ActiveValue::Set(None),
            user_agent: sea_orm::ActiveValue::Set(None),
            step_total: sea_orm::ActiveValue::Set(2),
            step_remain: sea_orm::ActiveValue::Set(2),
            failed_attempts: sea_orm::ActiveValue::Set(0),
            expired_at: sea_orm::ActiveValue::Set(Some(expired_at.naive_utc())),
            approved_at: sea_orm::ActiveValue::Set(None),
            declined_at: sea_orm::ActiveValue::Set(None),
            created_at: sea_orm::ActiveValue::Set(now.naive_utc()),
        };

        let result = challenge.insert(&self.db).await?;
        info!(%result.id, "认证挑战创建成功");
        Ok(result)
    }

    /// 添加认证因子 (2FA)
    pub async fn add_auth_factor(
        &self,
        account_id: Uuid,
        factor_type: &str,
        factor_data: &str,
    ) -> Result<Uuid, AuthError> {
        let now = Utc::now();
        let factor_id = Uuid::now_v7();

        let factor = account_auth_factor::ActiveModel {
            id: sea_orm::ActiveValue::Set(factor_id),
            account_id: sea_orm::ActiveValue::Set(account_id),
            factor_type: sea_orm::ActiveValue::Set(factor_type.to_string()),
            factor_data: sea_orm::ActiveValue::Set(factor_data.to_string()),
            enabled_at: sea_orm::ActiveValue::Set(Some(now.naive_utc())),
            trustworthy: sea_orm::ActiveValue::Set(1),
            created_at: sea_orm::ActiveValue::Set(now.naive_utc()),
        };

        factor.insert(&self.db).await?;
        Ok(factor_id)
    }

    /// 验证认证因子
    pub async fn verify_auth_factor(
        &self,
        account_id: Uuid,
        factor_type: &str,
        data: &str,
    ) -> Result<bool, AuthError> {
        let factor = account_auth_factor::Entity::find()
            .filter(account_auth_factor::Column::AccountId.eq(account_id))
            .filter(account_auth_factor::Column::FactorType.eq(factor_type))
            .one(&self.db)
            .await?;

        Ok(factor.is_some_and(|f| f.factor_data == data))
    }
}

/// JWT Token 声明
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iss: String,
    pub exp: i64,
    pub nbf: i64,
    pub iat: i64,
    pub jti: String,
    #[serde(rename = "type")]
    pub token_type: String,
}

/// 认证错误
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("数据库错误: {0}")]
    Database(#[from] sea_orm::error::DbErr),

    #[error("Token 错误: {0}")]
    TokenError(#[from] jwt::errors::Error),

    #[error("无效的凭据")]
    InvalidCredentials,

    #[error("账户未激活")]
    AccountNotActivated,

    #[error("账户不存在")]
    AccountNotFound,

    #[error("{0} 已存在")]
    AlreadyExists(String),

    #[error("无效的 Token 类型")]
    InvalidTokenType,

    #[error("无效的 Token")]
    InvalidToken,

    #[error("会话不存在")]
    SessionNotFound,
}