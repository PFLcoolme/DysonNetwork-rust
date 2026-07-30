use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use hyper::upgrade::OnUpgrade;
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::{TokioExecutor, TokioIo},
};
use serde::Serialize;
use tokio::{io::copy_bidirectional, time::timeout};
use tracing::{info, warn};

use crate::{
    limiter::{LimitDecision, RateLimiter},
    routing::{RouteError, RouteTable},
};

type HttpClient = Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct AppState {
    client: HttpClient,
    routes: RouteTable,
    limiter: Arc<RateLimiter>,
    upstream_timeout: Duration,
    external_scheme: HeaderValue,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

impl AppState {
    pub fn new(
        routes: RouteTable,
        limiter: Arc<RateLimiter>,
        upstream_timeout: Duration,
        external_scheme: HeaderValue,
    ) -> Self {
        let mut connector = HttpConnector::new();
        connector.enforce_http(false);
        let client = Client::builder(TokioExecutor::new()).build(connector);
        Self {
            client,
            routes,
            limiter,
            upstream_timeout,
            external_scheme,
        }
    }

    pub fn public_routes(&self) -> &'static [&'static str] {
        self.routes.public_routes()
    }
}

pub async fn proxy(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    mut request: Request<Body>,
) -> Response<Body> {
    let decision = state.limiter.check(peer.ip());
    if !decision.allowed {
        return rate_limited(decision);
    }

    let original_uri = request.uri().clone();
    let route = match state.routes.resolve(&original_uri) {
        Ok(route) => route,
        Err(RouteError::NotFound) => {
            return with_rate_headers(
                json_error(
                    StatusCode::NOT_FOUND,
                    "route_not_found",
                    "No gateway route matches this path",
                ),
                decision,
            );
        }
        Err(RouteError::WebSocketNotConfigured) => {
            return with_rate_headers(
                json_error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "websocket_not_configured",
                    "The WebSocket upstream is not configured",
                ),
                decision,
            );
        }
        Err(RouteError::InvalidUri) => {
            return with_rate_headers(
                json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "invalid_upstream_uri",
                    "The gateway produced an invalid upstream URI",
                ),
                decision,
            );
        }
    };

    let original_host = request.headers().get(header::HOST).cloned();
    let is_upgrade = is_upgrade_request(request.headers());
    let downstream_upgrade = is_upgrade.then(|| hyper::upgrade::on(&mut request));
    *request.uri_mut() = route.uri;
    prepare_forwarded_headers(&mut request, peer, original_host, &state.external_scheme);

    let method = request.method().clone();
    let started_at = std::time::Instant::now();
    let upstream_response = timeout(state.upstream_timeout, state.client.request(request)).await;
    let mut response = match upstream_response {
        Ok(Ok(response)) => response,
        Ok(Err(error)) => {
            warn!(service = route.service, %error, "upstream request failed");
            return with_rate_headers(
                json_error(
                    StatusCode::BAD_GATEWAY,
                    "upstream_unavailable",
                    "The upstream service could not be reached",
                ),
                decision,
            );
        }
        Err(_) => {
            warn!(service = route.service, "upstream request timed out");
            return with_rate_headers(
                json_error(
                    StatusCode::GATEWAY_TIMEOUT,
                    "upstream_timeout",
                    "The upstream service did not respond in time",
                ),
                decision,
            );
        }
    };

    if response.status() == StatusCode::SWITCHING_PROTOCOLS {
        if let Some(downstream_upgrade) = downstream_upgrade {
            let upstream_upgrade = hyper::upgrade::on(&mut response);
            tokio::spawn(relay_upgrades(downstream_upgrade, upstream_upgrade));
        }
    }

    info!(
        service = route.service,
        %method,
        path = original_uri.path(),
        status = response.status().as_u16(),
        elapsed_ms = started_at.elapsed().as_millis(),
        "proxied request"
    );

    let (parts, body) = response.into_parts();
    with_rate_headers(Response::from_parts(parts, Body::new(body)), decision)
}

async fn relay_upgrades(downstream: OnUpgrade, upstream: OnUpgrade) {
    let (Ok(downstream), Ok(upstream)) = tokio::join!(downstream, upstream) else {
        warn!("failed to establish both sides of an upgraded connection");
        return;
    };
    let mut downstream = TokioIo::new(downstream);
    let mut upstream = TokioIo::new(upstream);
    if let Err(error) = copy_bidirectional(&mut downstream, &mut upstream).await {
        warn!(%error, "upgraded connection relay failed");
    }
}

fn prepare_forwarded_headers(
    request: &mut Request<Body>,
    peer: SocketAddr,
    original_host: Option<HeaderValue>,
    external_scheme: &HeaderValue,
) {
    if let Some(authority) = request.uri().authority() {
        if let Ok(value) = HeaderValue::from_str(authority.as_str()) {
            request.headers_mut().insert(header::HOST, value);
        }
    }
    if let Ok(value) = HeaderValue::from_str(&peer.ip().to_string()) {
        request
            .headers_mut()
            .insert(HeaderName::from_static("x-forwarded-for"), value);
    }
    request.headers_mut().insert(
        HeaderName::from_static("x-forwarded-proto"),
        external_scheme.clone(),
    );
    if let Some(host) = original_host {
        request
            .headers_mut()
            .insert(HeaderName::from_static("x-forwarded-host"), host);
    }
}

fn is_upgrade_request(headers: &HeaderMap) -> bool {
    headers.contains_key(header::UPGRADE)
        && headers
            .get(header::CONNECTION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value
                    .split(',')
                    .any(|token| token.trim().eq_ignore_ascii_case("upgrade"))
            })
}

fn rate_limited(decision: LimitDecision) -> Response<Body> {
    let mut response = json_error(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limit_exceeded",
        "Too many requests from this client",
    );
    if let Ok(value) = HeaderValue::from_str(&decision.retry_after_seconds.to_string()) {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }
    with_rate_headers(response, decision)
}

fn with_rate_headers(mut response: Response<Body>, decision: LimitDecision) -> Response<Body> {
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(&decision.limit.to_string()) {
        headers.insert(HeaderName::from_static("ratelimit-limit"), value);
    }
    if let Ok(value) = HeaderValue::from_str(&decision.remaining.to_string()) {
        headers.insert(HeaderName::from_static("ratelimit-remaining"), value);
    }
    response
}

fn json_error(status: StatusCode, code: &'static str, message: &'static str) -> Response<Body> {
    (status, Json(ErrorBody { code, message })).into_response()
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, Router};
    use serde_json::{json, Value};
    use tokio::net::TcpListener;
    use url::Url;

    use super::*;
    use crate::routing::Upstreams;

    #[tokio::test]
    async fn proxies_streaming_request_with_rewritten_path() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream_address = listener.local_addr().unwrap();
        let upstream = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(echo_request))
                .await
                .unwrap();
        });
        let base_url = Url::parse(&format!("http://{upstream_address}")).unwrap();
        let state = test_state(base_url);
        let request = Request::builder()
            .uri("/padlock/auth/login?next=%2Fhome")
            .header(header::HOST, "api.starfur.test")
            .header("x-forwarded-for", "203.0.113.50")
            .header("x-forwarded-proto", "https")
            .body(Body::from("payload"))
            .unwrap();

        let response = proxy(
            State(state),
            ConnectInfo(SocketAddr::from(([127, 0, 0, 2], 45678))),
            request,
        )
        .await;
        let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        let echoed: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(echoed["path"], "/api/auth/login?next=%2Fhome");
        assert_eq!(echoed["body"], "payload");
        assert_eq!(echoed["forwarded_for"], "127.0.0.2");
        assert_eq!(echoed["forwarded_proto"], "https");
        assert_eq!(echoed["forwarded_host"], "api.starfur.test");
        upstream.abort();
    }

    #[tokio::test]
    async fn returns_json_for_unknown_route() {
        let state = test_state(Url::parse("http://127.0.0.1:9").unwrap());
        let request = Request::builder()
            .uri("/unknown")
            .body(Body::empty())
            .unwrap();

        let response = proxy(
            State(state),
            ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 45678))),
            request,
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers().get("ratelimit-remaining").unwrap(), "99");
        let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        let error: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error["code"], "route_not_found");
    }

    async fn echo_request(request: Request<Body>) -> Json<Value> {
        let path = request
            .uri()
            .path_and_query()
            .map_or_else(String::new, ToString::to_string);
        let headers = request.headers().clone();
        let body = to_bytes(request.into_body(), 64 * 1024).await.unwrap();
        Json(json!({
            "path": path,
            "body": String::from_utf8(body.to_vec()).unwrap(),
            "forwarded_for": header_value(&headers, "x-forwarded-for"),
            "forwarded_proto": header_value(&headers, "x-forwarded-proto"),
            "forwarded_host": header_value(&headers, "x-forwarded-host"),
        }))
    }

    fn header_value<'a>(headers: &'a HeaderMap, name: &'static str) -> Option<&'a str> {
        headers.get(name).and_then(|value| value.to_str().ok())
    }

    fn test_state(base_url: Url) -> AppState {
        let upstreams = Upstreams {
            padlock: base_url.clone(),
            passport: base_url.clone(),
            sphere: base_url.clone(),
            messager: base_url.clone(),
            ring: base_url.clone(),
            wallet: base_url.clone(),
            develop: base_url.clone(),
            storage: base_url.clone(),
            push: base_url.clone(),
            mail: base_url.clone(),
            workspace: base_url.clone(),
            websocket: Some(base_url),
        };
        AppState::new(
            RouteTable::new(upstreams),
            Arc::new(RateLimiter::per_minute(100)),
            Duration::from_secs(2),
            HeaderValue::from_static("https"),
        )
    }
}
