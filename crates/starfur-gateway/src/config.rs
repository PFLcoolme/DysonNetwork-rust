use std::{env, net::SocketAddr, str::FromStr, time::Duration};

use thiserror::Error;
use url::Url;

use crate::routing::Upstreams;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_EXTERNAL_SCHEME: &str = "http";
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const DEFAULT_RATE_LIMIT_PER_MINUTE: u32 = 600;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub external_scheme: String,
    pub upstream_timeout: Duration,
    pub rate_limit_per_minute: u32,
    pub upstreams: Upstreams,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{name} must be a valid socket address: {value}")]
    SocketAddress { name: &'static str, value: String },
    #[error("{name} must be an unsigned integer: {value}")]
    Integer { name: &'static str, value: String },
    #[error("{name} must use http and include a host: {value}")]
    Upstream { name: &'static str, value: String },
    #[error("{name} must be either http or https: {value}")]
    ExternalScheme { name: &'static str, value: String },
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = parse_socket_addr("STARFUR_BIND_ADDR", DEFAULT_BIND_ADDR)?;
        let external_scheme =
            parse_external_scheme("STARFUR_EXTERNAL_SCHEME", DEFAULT_EXTERNAL_SCHEME)?;
        let timeout_seconds =
            parse_integer("STARFUR_UPSTREAM_TIMEOUT_SECONDS", DEFAULT_TIMEOUT_SECONDS)?;
        let rate_limit_per_minute = parse_integer(
            "STARFUR_RATE_LIMIT_PER_MINUTE",
            DEFAULT_RATE_LIMIT_PER_MINUTE,
        )?;

        Ok(Self {
            bind_addr,
            external_scheme,
            upstream_timeout: Duration::from_secs(timeout_seconds),
            rate_limit_per_minute,
            upstreams: Upstreams {
                padlock: parse_upstream("STARFUR_UPSTREAM_PADLOCK", "http://127.0.0.1:5101")?,
                passport: parse_upstream("STARFUR_UPSTREAM_PASSPORT", "http://127.0.0.1:5102")?,
                sphere: parse_upstream("STARFUR_UPSTREAM_SPHERE", "http://127.0.0.1:5103")?,
                messager: parse_upstream("STARFUR_UPSTREAM_MESSAGER", "http://127.0.0.1:5104")?,
                ring: parse_upstream("STARFUR_UPSTREAM_RING", "http://127.0.0.1:5105")?,
                wallet: parse_upstream("STARFUR_UPSTREAM_WALLET", "http://127.0.0.1:5106")?,
                develop: parse_upstream("STARFUR_UPSTREAM_DEVELOP", "http://127.0.0.1:5107")?,
                storage: parse_upstream("STARFUR_UPSTREAM_STORAGE", "http://127.0.0.1:5108")?,
                push: parse_upstream("STARFUR_UPSTREAM_PUSH", "http://127.0.0.1:5109")?,
                mail: parse_upstream("STARFUR_UPSTREAM_MAIL", "http://127.0.0.1:5110")?,
                workspace: parse_upstream("STARFUR_UPSTREAM_WORKSPACE", "http://127.0.0.1:5111")?,
                websocket: parse_optional_upstream("STARFUR_UPSTREAM_WEBSOCKET")?,
            },
        })
    }
}

fn parse_socket_addr(name: &'static str, default: &str) -> Result<SocketAddr, ConfigError> {
    let value = env::var(name).unwrap_or_else(|_| default.to_owned());
    SocketAddr::from_str(&value).map_err(|_| ConfigError::SocketAddress { name, value })
}

fn parse_integer<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: FromStr + ToString,
{
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    value
        .parse()
        .map_err(|_| ConfigError::Integer { name, value })
}

fn parse_upstream(name: &'static str, default: &str) -> Result<Url, ConfigError> {
    let value = env::var(name).unwrap_or_else(|_| default.to_owned());
    validate_upstream(name, value)
}

fn parse_optional_upstream(name: &'static str) -> Result<Option<Url>, ConfigError> {
    let Ok(value) = env::var(name) else {
        return Ok(None);
    };
    if value.trim().is_empty() {
        return Ok(None);
    }
    validate_upstream(name, value).map(Some)
}

fn validate_upstream(name: &'static str, value: String) -> Result<Url, ConfigError> {
    let parsed = Url::parse(&value).map_err(|_| ConfigError::Upstream {
        name,
        value: value.clone(),
    })?;
    if parsed.scheme() != "http" || parsed.host_str().is_none() {
        return Err(ConfigError::Upstream { name, value });
    }
    Ok(parsed)
}

fn parse_external_scheme(name: &'static str, default: &str) -> Result<String, ConfigError> {
    let value = env::var(name).unwrap_or_else(|_| default.to_owned());
    if !matches!(value.as_str(), "http" | "https") {
        return Err(ConfigError::ExternalScheme { name, value });
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_internal_http_upstream() {
        let upstream = validate_upstream("TEST", "http://service:8080".to_owned()).unwrap();

        assert_eq!(upstream.host_str(), Some("service"));
    }

    #[test]
    fn rejects_unsupported_https_upstream() {
        let error = validate_upstream("TEST", "https://service:8443".to_owned()).unwrap_err();

        assert!(matches!(error, ConfigError::Upstream { .. }));
    }

    #[test]
    fn rejects_unknown_external_scheme() {
        let error = parse_external_scheme("TEST", "ftp").unwrap_err();

        assert!(matches!(error, ConfigError::ExternalScheme { .. }));
    }
}
