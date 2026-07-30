//! 认证服务层

use anyhow::Context;
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};
use jsonwebtoken as jwt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, warn};

use super::entity::{accounts, account_secrets, account_auth_factor, auth_session, auth_challenge};

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
            access_expiry: TimeDelta::seconds(3600),      // 1 hour
            refresh_expiry: TimeDelta::seconds(604800),   // 7 days
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
const PASSWORD_HASH_COST: u32 = 3; //  argon2 的迭代次数

///认证服务实现
#[derive(Clone)]
pub struct AuthService {
    pub db: sea_orm::DatabaseConnection,
    pub jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(db: sea_orm::DatabaseConnection, jwt_config: JwtConfig) -> Self {
        Self { db, jwt_config }
    }

    /// 注册新用户
    pub async fn register(&self, name: &str, password: &str) -> Result<Uuid, AuthError> {
        // 检查用户名是否已存在
        use sea_orm::EntityTrait;
        let existing = accounts::Entity::find()
            .filter(accounts::Column::Name.eq(name))
            .one(&self.db)
            .await?;
        
        if existing.is_some() {
            return Err(AuthError::AlreadyExists("username".to_string()));
        }

        // 密码哈希
        let hashed_password = argon2::hash_encoded(
            password.as_bytes(),
            &rand::thread_rng().gen::<[u8; 16]>(),
            argon2::Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::0x600,
                argon2::Params::new(
                    argon2::MemorySize::new(65536), // 64MB
                    4, // 并行度
                    PASSWORD_HASH_COST,
                    None,
                ).unwrap(),
                None,
            ).unwrap(),
        ).unwrap();

        let account_id = Uuid::now_v7();
        let now = Utc::now();

        // 创建账户
        let account = accounts::ActiveModel {
            id: sea_orm::ActiveValue::Set(account_id),
            name: sea_orm::ActiveValue::Set(name.to_string()),
            nick: sea_orm::ActiveValue::Set(format!("User-{}", &name[..4.min(name.len())])),
            language: sea_orm::ActiveValue::Set("zh-Hans".to_string()),
            region: sea_orm::ActiveValue::Set("CN".to_string()),
            is_superuser: sea_orm::ActiveValue::Set(false),
            activated_at: sea_orm::ActiveValue::Set(Some(now)),
            created_at: sea_orm::ActiveValue::Set(now),
            updated_at: sea_orm::ActiveValue::Set(now),
        };
        account.insert(&self.db).await?;

        // 创建密码密钥
        let secret = account_secrets::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::now_v7()),
            account_id: sea_orm::ActiveValue::Set(account_id),
            secret_type: sea_orm::ActiveValue::Set("password".to_string()),
            secret_value: sea_orm::ActiveValue::Set(hashed_password),
            enabled_at: sea_orm::ActiveValue::Set(Some(now)),
            created_at: sea_orm::ActiveValue::Set(now),
        };
        secret.insert(&self.db).await?;

        info!(
%account_id, %name, "用户注册成功");        Ok(account_id)
    }

    /// 验证密码并返回账户 ID
    pub async fn verify_password(&self, name: &str, password: &str) -> Result<Uuid, AuthError> {
       
        
 use sea_orm::EntityTrait;        // 查找账户
        let account = accounts::Entity::find()
            .filter(accounts::Column::Name.eq(name))
            .one(&self.db)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        if !account.activated_at.is_some() {
            return Err(AuthError::AccountNotActivated);
        }

        // 查找密码密钥
        let secret = account_secrets::Entity::find()
            .filter(account_secrets::Column::AccountId.eq(account.id))
            .filter(account_secrets::Column::SecretType.eq("password"))
            .one(&self.db)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // 验证密码
        if !argon2::verify_encoded(&secret.secret_value, password.as_bytes()) {
            warn!(%account.id, "密码验证失败");
            return Err(AuthError::InvalidCredentials);
        }

        // 检查是否需要重新哈希（参数更新时）
        if argon2::needs_rehash(&secret.secret_value, argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::0x600,
            argon2::Params::new(
                argon2::MemorySize::new(65536),
                4,
                PASSWORD_HASH_COST,
                None,
            ).unwrap(),
            None,
        ).unwrap()) {
            let hashed = argon2::hash_encoded(
                password.as_bytes(),
                &rand::thread_rng().gen::<[u8; 16]>(),
                argon2::Argon2::new(
                    argon2::Algorithm::Argon2id,
                    argon2::Version::0x600,
                    argon2::Params::new(
                        argon2::MemorySize::new(65536),
                        4,
                        PASSWORD_HASH_COST,
                        None,
                    ).unwrap(),
                    None,
                ).unwrap(),
            ).unwrap();
            
            let mut update = account_secrets::ActiveModel::from(secret);
            update.secret_value = sea_orm::ActiveValue::Set(hashed);
            update.update(&self.db).await.ok();
        }

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
        use sea_orm::EntityTrait;

        let now = Utc::now();
        let access_expires = now + self.jwt_config.access_expiry;
        let refresh_expires = now + self.jwt_config.refresh_expiry;

        // 签发 Access Token
        let access_token = self.issue_token(account_id, true, &access_expires)?;
        // 签发 Refresh Token
        let refresh_token = self.issue_token(account_id, false, &refresh_expires)?;

        // 存储会话
        let session = auth_session::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::now_v7()),
            account_id: sea_orm::ActiveValue::Set(account_id),
            device_id: sea_orm::ActiveValue::Set(device_id),
            device_name: sea_orm::ActiveValue::Set(device_name),
            platform: sea_orm::ActiveValue::Set(platform.to_string()),
            access_token: sea_orm::ActiveValue::Set(access_token.clone()),
            access_expires_at: sea_orm::ActiveValue::Set(access_expires),
            refresh_token: sea_orm::ActiveValue::Set(refresh_token.clone()),
            refresh_expires_at: sea_orm::ActiveValue::Set(refresh_expires),
            created_at: sea_orm::ActiveValue::Set(now),
            expired_at: sea_orm::ActiveValue::Set(refresh_expires),
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
        let issuer = "solsynth-auth";
        let mut claims = TokenClaims {
            sub: account_id.to_string(),
            iss: issuer.to_string(),
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
        let token_data = jwt::decode::<TokenClaims>(
            token,
            &jwt::DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &jwt::Validation::default(),
        ).map_err(AuthError::TokenError)?;

        Ok(token_data.claims)
    }

    /// 刷新 Token
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenResponse, AuthError> {
        let claims = self.verify_token(refresh_token)?;

        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidTokenType);
        }

        let account_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

        // 检查账户是否存在
        use sea_orm::EntityTrait;
        let account = accounts::Entity::find()
            .filter(accounts::Column::Id.eq(account_id))
            .one(&self.db)
            .await?
            .ok_or(AuthError::AccountNotFound)?;

        if !account.activated_at.is_some() {
            return Err(AuthError::AccountNotActivated);
        }

        self.create_session(account_id, claims.jti, None, "refresh").await
    }

    /// 撤销会话
    pub async fn revoke_session(&self, session_id: Uuid) -> Result<(), AuthError> {
        use        sea_orm::EntityTrait;
 let session = auth_session::Entity::find()
            .filter(auth_session::Column::Id.eq(session_id))
            .one(&self.db)
            .await?
            .ok_or(AuthError::SessionNotFound)?;

        sea_orm::ActiveModelTrait::delete(session.into_active_model()).await?;
        Ok(())
    }

    /// 创建认证挑战 (用于多因素认证)
    pub async fn create_challenge(
        &self,
        account_id: Uuid,
        device_id: String,
        platform: &str,
    ) -> Result<auth_challenge::Model, AuthError> {
        use sea_orm::EntityTrait;

        let now = Utc::now();
        let expired_at = now + TimeDelta::minutes(5);

        let account = accounts::Entity::find()
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
            expired_at: sea_orm::ActiveValue::Set(Some(expired_at)),
            approved_at: sea_orm::ActiveValue::Set(None),
            declined_at: sea_orm::ActiveValue::Set(None),
            created_at: sea_orm::ActiveValue::Set(now),
        };

        let result = challenge.insert(&self.db).await?;
        info!(%result.id, "认证挑战创建成功");


        Ok(result)
    }    /// 添加认证因子 (2FA)
    pub async fn add_auth_factor(
        &self,
        account_id: Uuid,
        factor_type: &str,
        factor_data: &str,
    ) -> Result<Uuid, AuthError> {
        use sea_orm::EntityTrait;

        let now = Utc::now();
        let factor_id = Uuid::now_v7();

        let factor = account_auth_factor::ActiveModel {
            id: sea_orm::ActiveValue::Set(factor_id),
            account_id: sea_orm::ActiveValue::Set(account_id),
            factor_type: sea_orm::ActiveValue::Set(factor_type.to_string()),
            factor_data: sea_orm::ActiveValue::Set(factor_data.to_string()),
            enabled_at: sea_orm::ActiveValue::Set(Some(now)),
            trustworthy: sea_orm::ActiveValue::Set(1),
            created_at: sea_orm::ActiveValue::Set(now),
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
        use sea_orm::EntityTrait;

        let factor = account_auth_factor::Entity::find()
            .filter(account_auth_factor::Column::AccountId.eq(account_id))
            .filter(account_auth_factor::Column::FactorType.eq(factor_type))
            .one(&self.db)
            .await?;

        Ok(factor.is_some_and(|f| f.factor_data == data))
    }
}

 /// JWT Token声明
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
    TokenError(#[from] jwt::errors::JwtError),
    
    #[error("无效的凭据")]
    InvalidCredentials,
    
    #[error("账户未激活")]
    AccountNotActivated,
    
    #[error("账户不存在: {0}")]
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