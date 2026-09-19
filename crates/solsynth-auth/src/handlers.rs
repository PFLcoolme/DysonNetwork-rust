//! HTTP 请求处理器

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::models::service::{AuthState, TokenResponse};

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

/// 启用 2FA 请求
#[derive(Debug, Deserialize)]
pub struct Enable2FARequest {
    pub factor_type: String,
    pub verification_code: String,
}

/// 禁用 2FA 请求
#[derive(Debug, Deserialize)]
pub struct Disable2FARequest {
    pub factor_type: String,
    pub verification_code: String,
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

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }
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
) -> (CookieJar, Json<ApiResponse<TokenResponse>>) {
    match state.auth_service.verify_password(&req.account, &req.password).await {
        Ok(account_id) => {
            let device_id = uuid::Uuid::now_v7().to_string();
            match state.auth_service.create_session(account_id, device_id, None, "Web").await {
                Ok(tokens) => {
                    let mut cookie = Cookie::new(state.cookie_name.clone(), tokens.token.clone());
                    cookie.set_path("/");
                    cookie.set_http_only(true);
                    cookie.set_same_site(SameSite::Strict);
                    cookie.set_max_age(cookie::time::Duration::hours(1));
                    let new_jar = jar.add(cookie);
                    info!(account = req.account, "用户登录成功");
                    (new_jar, Json(ApiResponse::ok(tokens)))
                }
                Err(e) => {
                    tracing::warn!(error = ?e, "创建会话失败");
                    (
                        jar,
                        Json(ApiResponse::<TokenResponse> {
                            success: false,
                            data: None,
                            error: Some(ApiError {
                                code: "session_error".to_string(),
                                message: "Failed to create session".to_string(),
                            }),
                        }),
                    )
                }
            }
        }
        Err(_) => {
            tracing::warn!(account = req.account, "登录失败");
            (
                jar,
                Json(ApiResponse::<TokenResponse> {
                    success: false,
                    data: None,
                    error: Some(ApiError {
                        code: "invalid_credentials".to_string(),
                        message: "Invalid username or password".to_string(),
                    }),
                }),
            )
        }
    }
}

/// 注册处理器
pub async fn register(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    match state.auth_service.register(&req.account, &req.password).await {
        Ok(id) => {
            let body = ApiResponse::ok(format!("Registration successful, account_id: {}", id));
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(e) => {
            tracing::warn!(account = req.account, error = ?e, "注册失败");
            let body = ApiResponse::<()> {
                success: false,
                data: None,
                error: Some(ApiError {
                    code: "register_error".to_string(),
                    message: "Registration failed".to_string(),
                }),
            };
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        }
    }
}

/// 登出处理器
pub async fn logout(
    State(state): State<AuthState>,
    jar: CookieJar,
) -> impl IntoResponse {
    let _jar = jar.remove(Cookie::new(state.cookie_name.clone(), ""));
    let body = ApiResponse::ok("Logged out successfully");
    (StatusCode::OK, Json(body)).into_response()
}

/// 健康检查
pub async fn health() -> impl IntoResponse {
    let body = ApiResponse::ok("ok");
    (StatusCode::OK, Json(body)).into_response()
}

/// OIDC 配置端点
pub async fn openid_configuration() -> impl IntoResponse {
    let body = serde_json::json!({
        "issuer": "https://api.starfur.test/padlock",
        "authorization_endpoint": "https://api.starfur.test/padlock/oauth/authorize",
        "token_endpoint": "https://api.starfur.test/padlock/oauth/token",
        "jwks_uri": "https://api.starfur.test/padlock/.well-known/jwks",
        "webauthn_endpoint": "https://api.starfur.test/padlock/.well-known/webauthn",
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
    });
    Json(body).into_response()
}

/// JWKS 端点
pub async fn jwks() -> impl IntoResponse {
    let body = serde_json::json!({ "keys": [] });
    Json(body).into_response()
}

/// WebAuthn 配置端点
pub async fn webauthn_config() -> impl IntoResponse {
    let body = serde_json::json!({
        "rp_id": "localhost",
        "rp_name": "Starfur",
        "supported_passkeys": true,
    });
    Json(body).into_response()
}

/// 认证因子列表
pub async fn auth_factors(State(_state): State<AuthState>) -> impl IntoResponse {
    let body = serde_json::json!({ "factors": [] });
    Json(body).into_response()
}

/// 启用 2FA
pub async fn enable_2fa(
    State(_state): State<AuthState>,
    _req: Json<Enable2FARequest>,
) -> impl IntoResponse {
    let body = ApiResponse::ok("2FA enabled");
    (StatusCode::OK, Json(body)).into_response()
}

/// 禁用 2FA
pub async fn disable_2fa(
    State(_state): State<AuthState>,
    _req: Json<Disable2FARequest>,
) -> impl IntoResponse {
    let body = ApiResponse::ok("2FA disabled");
    (StatusCode::OK, Json(body)).into_response()
}

/// 路由配置
pub fn routes(auth_state: AuthState) -> Router {
    Router::new()
        .route("/padlock/auth/login", post(login))
        .route("/padlock/auth/register", post(register))
        .route("/padlock/auth/logout", post(logout))
        .route("/health", get(health))
        .route("/.well-known/openid-configuration", get(openid_configuration))
        .route("/.well-known/jwks", get(jwks))
        .route("/.well-known/webauthn", get(webauthn_config))
        .route("/padlock/auth/factors", get(auth_factors))
        .route("/padlock/auth/factor/enable", post(enable_2fa))
        .route("/padlock/auth/factor/disable", post(disable_2fa))
        .with_state(auth_state)
}