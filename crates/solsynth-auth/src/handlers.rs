//! HTTP 请求处理器

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::models::service::{AuthState, AuthService, JwtConfig, TokenResponse};

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: String,
}

/// 注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub account: String,
    pub password: String,
}

/// 通用响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

/// 登录处理器
pub async fn login(
    State(state): State<AuthState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.auth_service.verify_password(&req.account, &req.password).await {
        Ok(account_id) => {
            let device_id = uuid::Uuid::now_v7().to_string();
            match state.auth_service.create_session(account_id, device_id, None, "Web").await {
                Ok(tokens) => {
                    let jar = jar.add(
                        axum_extra::cookie::Cookie::build(&state.cookie_name, tokens.token.clone())
                            .path("/")
                            .http_only(true)
                            .secure(cfg!(feature = "secure-cookie"))
                            .same_site(axum_extra::cookie::SameSite::Strict)
                            .max_age(humantime::Duration::from(std::time::Duration::from_secs(3600)))
                            .into(),
                    );
                    info!(account = req.account, "用户登录成功");
                    (jar, Json(ApiResponse::ok(tokens)))
                }
                Err(e) => {
                    tracing::warn!(error = ?e, "创建会话失败");
                    error_response("session_error", "Failed to create session")
                }
            }
        }
        Err(_) => {
            tracing::warn!(account = req.account, "登录失败");
            error_response("invalid_credentials", "Invalid username or password")
        }
    }
}

/// 注册处理器
pub async fn register(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    match state.auth_service.register(&req.account, &req.password).await {
        Ok(_) => ok_response("Registration successful"),
        Err(e) => {
            tracing::warn!(account = req.account, error = ?e, "注册失败");
            error_response("register_error", "Registration failed")
        }
    }
}

/// 登出处理器
pub async fn logout(
    State(state): State<AuthState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let jar = jar.remove(axum_extra::cookie::Cookie::named(&state.cookie_name));
    ok_response("Logged out successfully")
}

/// 健康检查
pub async fn health() -> impl IntoResponse {
    ok_response("ok")
}

/// OIDC 配置端点 (Padlock Well-Known)
pub async fn openid_configuration() -> impl IntoResponse {
    serde_json::json!({
        "issuer": "https://api.starfur.test/padlock",
        "authorization_endpoint": "https://api.starfur.test/padlock/oauth/authorize",
        "token_endpoint": "https://api.starfur.test/padlock/oauth/token",
        "jwks_uri": "https://api.starfur.test/padlock/.well-known/jwks",
        "webauthn_endpoint": "https://api.starfur.test/padlock/.well-known/webauthn",
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
    })
}

/// JWKS 端点
pub async fn jwks() -> impl IntoResponse {
    // TODO: 实现 JWKS 端点 (RSA 公钥)
    serde_json::json!({ "keys": [] })
}

/// WebAuthn 配置端点
pub async fn webauthn_config() -> impl IntoResponse {
    // TODO: 实现 WebAuthn 配置
    serde_json::json!({
        "rp_id": "localhost",
        "rp_name": "Starfur",
        "supported_passkeys": true,
    })
}

/// 认证
因子列表pub async fn auth_factors(State(state): State<AuthState>) -> impl IntoResponse {
    // TODO: 实现查询用户认证因子列表
    serde_json::json!({ "factors": [] })
}

/// 启用 2FA
pub async fn enable_2fa(
    State(state): State<AuthState>,
    Json(_req): Json<Enable2FARequest>,
) -> impl IntoResponse {
    // TODO: 实现启用 2FA
    ok_response("2FA enabled")
}

/// 禁用 2FA
pub async fn disable_2fa(
    State(state): State<AuthState>,
    Json(_req): Json<Disable2FARequest>,
) -> impl IntoResponse {
    // TODO: 实现禁用 2FA
    ok_response("2FA disabled")
}

/// 路由配置
pub fn routes(auth_state: AuthState) -> Router {
    Router::new()
        // 认证端点
        .route("/padlock/auth/login", post(login))
        .route("/padlock/auth/register", post(register))
        .route("/padlock/auth/logout", post(logout))
        // 健康检查
        .route("/health", get(health))
        // OIDC 发现端点
        .route("/.well-known/openid-configuration", get(openid_configuration))
        .route("/.well-known/jwks", get(jwks))
        .route("/.well-known/webauthn", get(webauthn_config))
        // 认证因子
        .route("/padlock/auth/factors", get(auth_factors))
        .route("/padlock/auth/factor/enable", post(enable_2fa))
        .route("/padlock/auth/factor/disable", post(disable_2fa))
        .with_state(auth_state)
}

#[derive(Debug, Deserialize)]
pub struct Enable2FARequest {
    pub factor_type: String,
    pub verification_code: String,
}

#[derive(Debug, Deserialize)]
pub struct Disable2FARequest {
    pub factor_type: String,
    pub verification_code: String,
}

fn ok_response(message: &str) -> impl IntoResponse {
    (StatusCode::OK, Json(ApiResponse::ok(message)))
}

fn error_response(code: &str, message: &str) -> impl IntoResponse {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
            }),
        }),
    )
}