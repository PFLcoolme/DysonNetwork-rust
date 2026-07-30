//! OIDC 第三方登录提供商

/// OIDC 提供商类型
#[derive(Debug, Clone, PartialEq)]
pub enum OidcProvider {
    Google,
    Apple,
    GitHub,
    Microsoft,
    Discord,
    Afdian,
    Steam,
}

impl std::fmt::Display for OidcProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OidcProvider::Google => write!(f, "google"),
            OidcProvider::Apple => write!(f, "apple"),
            OidcProvider::GitHub => write!(f, "github"),
            OidcProvider::Microsoft => write!(f, "microsoft"),
            OidcProvider::Discord => write!(f, "discord"),
            OidcProvider::Afdian => write!(f, "afdian"),
            OidcProvider::Steam => write!(f, "steam"),
        }
    }
}

impl From<&str> for OidcProvider {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "google" => Self::Google,
            "apple" => Self::Apple,
            "github" => Self::GitHub,
            "microsoft" => Self::Microsoft,
            "discord" => Self::Discord,
            "afdian" => Self::Afdian,
            "steam" => Self::Steam,
            _ => panic!("Unknown OIDC provider: {}", s),
        }
    }
}

/// OIDC 回调响应
#[derive(Debug, serde::Serialize)]
pub struct OidcCallback {
    pub provider: String,
    pub user_info: OidcUserInfo,
}

/// OIDC 用户信息
#[derive(Debug, serde::Serialize)]
pub struct OidcUserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}
