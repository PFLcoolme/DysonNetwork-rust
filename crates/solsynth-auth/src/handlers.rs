//! HTTP 请求处理器

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::services::AuthService;

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: Option<String>,
}

/// 响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub status: u16,
}

impl ApiResponse<()> {
    pub fn ok() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>, status: u16) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                status,
            }),
        }
    }
}

impl IntoResponse for ApiResponse<()> {
    fn into_response(self) -> axum::response::Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::from_u16(self.error.as_ref().unwrap().status)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        };
        (status, serde_json::to_string(&self).unwrap()).into_response()
    }
}