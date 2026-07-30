//! 通用工具模块
//!
//! 功能:
//! - 字符串处理
//! - 日期时间工具
//! - ID 生成
//! - 错误类型定义

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 通用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] anyhow::Error),

    #[error("认证失败: {0}")]
    Auth(String),

    #[error("无效请求: {0}")]
    InvalidRequest(String),

    #[error("资源未找到: {0}")]
    NotFound(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 分页请求
#[derive(Debug, Deserialize)]
pub struct PaginationRequest {
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    #[serde(default)]
    pub page: u32,
}

fn default_page_size() -> u32 {
    20
}

/// 分页响应
#[derive(Debug, Serialize)]
pub struct PaginationResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

impl<T> PaginationResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, page_size: u32) -> Self {
        Self {
            items,
            total,
            page,
            page_size,
        }
    }
}

/// 生成随机 UUID v4
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 验证邮箱格式
pub fn validate_email(email: &str) -> bool {
    regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .unwrap()
        .is_match(email)
}

/// 生成随机字符串
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect()
}

/// 将字符串转为 snake_case
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let characters: Vec<char> = s.chars().collect();
    for (index, current) in characters.iter().copied().enumerate() {
        let previous = index.checked_sub(1).and_then(|value| characters.get(value));
        let next = characters.get(index + 1);
        let starts_word = current.is_uppercase()
            && (previous.is_some_and(|value| value.is_lowercase() || value.is_numeric())
                || previous.is_some_and(|value| value.is_uppercase())
                    && next.is_some_and(|value| value.is_lowercase()));
        if starts_word && !result.ends_with('_') {
            result.push('_');
        }
        result.extend(current.to_lowercase());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("HTTPSConnection"), "https_connection");
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid"));
    }
}
