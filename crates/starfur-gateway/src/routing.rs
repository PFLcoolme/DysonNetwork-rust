use axum::http::Uri;
use thiserror::Error;
use url::Url;

#[derive(Clone, Debug)]
pub struct Upstreams {
    pub padlock: Url,
    pub passport: Url,
    pub sphere: Url,
    pub messager: Url,
    pub ring: Url,
    pub wallet: Url,
    pub develop: Url,
    pub storage: Url,
    pub push: Url,
    pub mail: Url,
    pub workspace: Url,
    pub websocket: Option<Url>,
}

#[derive(Clone, Debug)]
pub struct RouteTable {
    upstreams: Upstreams,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedRoute {
    pub service: &'static str,
    pub uri: Uri,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum RouteError {
    #[error("no route matches this path")]
    NotFound,
    #[error("the websocket upstream is not configured")]
    WebSocketNotConfigured,
    #[error("the resolved upstream URI is invalid")]
    InvalidUri,
}

impl RouteTable {
    pub fn new(upstreams: Upstreams) -> Self {
        Self { upstreams }
    }

    pub fn resolve(&self, incoming: &Uri) -> Result<ResolvedRoute, RouteError> {
        let path = incoming.path();

        if path == "/ws" {
            let upstream = self
                .upstreams
                .websocket
                .as_ref()
                .ok_or(RouteError::WebSocketNotConfigured)?;
            return build_route("websocket", upstream, path, incoming.query());
        }

        if path == "/api/tus" || path.starts_with("/api/tus/") {
            return build_route("storage", &self.upstreams.storage, path, incoming.query());
        }

        if is_padlock_well_known(path) {
            return build_route("padlock", &self.upstreams.padlock, path, incoming.query());
        }

        if is_sphere_federation(path) {
            return build_route("sphere", &self.upstreams.sphere, path, incoming.query());
        }

        let service_route = [
            ("padlock", "/padlock", &self.upstreams.padlock),
            ("passport", "/passport", &self.upstreams.passport),
            ("passport", "/pass", &self.upstreams.passport),
            ("sphere", "/sphere", &self.upstreams.sphere),
            ("messager", "/messager", &self.upstreams.messager),
            ("ring", "/ring", &self.upstreams.ring),
            ("wallet", "/wallet", &self.upstreams.wallet),
            ("develop", "/develop", &self.upstreams.develop),
            ("storage", "/storage", &self.upstreams.storage),
            ("storage", "/drive", &self.upstreams.storage),
            ("push", "/push", &self.upstreams.push),
            ("mail", "/mail", &self.upstreams.mail),
            ("workspace", "/workspace", &self.upstreams.workspace),
            ("padlock", "/id", &self.upstreams.padlock),
        ]
        .into_iter()
        .find(|(_, prefix, _)| path == *prefix || path.starts_with(&format!("{prefix}/")));

        let Some((service, prefix, upstream)) = service_route else {
            return Err(RouteError::NotFound);
        };
        let suffix = path.strip_prefix(prefix).unwrap_or_default();
        let rewritten_path = if suffix.is_empty() {
            "/api".to_owned()
        } else {
            format!("/api{suffix}")
        };
        build_route(service, upstream, &rewritten_path, incoming.query())
    }

    pub fn public_routes(&self) -> &'static [&'static str] {
        &[
            "/padlock/*",
            "/passport/*",
            "/sphere/*",
            "/messager/*",
            "/ring/*",
            "/wallet/*",
            "/develop/*",
            "/storage/*",
            "/push/*",
            "/mail/*",
            "/workspace/*",
            "/ws",
        ]
    }
}

fn is_padlock_well_known(path: &str) -> bool {
    matches!(
        path,
        "/.well-known/openid-configuration"
            | "/.well-known/jwks"
            | "/.well-known/webauthn"
            | "/.well-known/permissions"
            | "/.well-known/error-codes"
            | "/.well-known/apple-app-site-association"
            | "/.well-known/assetlinks.json"
    )
}

fn is_sphere_federation(path: &str) -> bool {
    path == "/.well-known/webfinger"
        || path == "/.well-known/nodeinfo"
        || path.starts_with("/.well-known/nodeinfo/")
        || path == "/activitypub"
        || path.starts_with("/activitypub/")
}

fn build_route(
    service: &'static str,
    upstream: &Url,
    path: &str,
    query: Option<&str>,
) -> Result<ResolvedRoute, RouteError> {
    let mut target = format!("{}{}", upstream.as_str().trim_end_matches('/'), path);
    if let Some(query) = query {
        target.push('?');
        target.push_str(query);
    }
    let uri = target.parse().map_err(|_| RouteError::InvalidUri)?;
    Ok(ResolvedRoute { service, uri })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn routes() -> RouteTable {
        let url = |port| Url::parse(&format!("http://127.0.0.1:{port}")).unwrap();
        RouteTable::new(Upstreams {
            padlock: url(5101),
            passport: url(5102),
            sphere: url(5103),
            messager: url(5104),
            ring: url(5105),
            wallet: url(5106),
            develop: url(5107),
            storage: url(5108),
            push: url(5109),
            mail: url(5110),
            workspace: url(5111),
            websocket: Some(url(5120)),
        })
    }

    #[test]
    fn rewrites_service_prefix_and_preserves_query() {
        let route = routes()
            .resolve(&"/padlock/auth/login?redirect=%2Fhome".parse().unwrap())
            .unwrap();

        assert_eq!(route.service, "padlock");
        assert_eq!(
            route.uri,
            "http://127.0.0.1:5101/api/auth/login?redirect=%2Fhome"
                .parse::<Uri>()
                .unwrap()
        );
    }

    #[test]
    fn maps_legacy_service_names() {
        let pass = routes()
            .resolve(&"/pass/users/me".parse().unwrap())
            .unwrap();
        let drive = routes()
            .resolve(&"/drive/files/1".parse().unwrap())
            .unwrap();

        assert_eq!(pass.service, "passport");
        assert_eq!(pass.uri.path(), "/api/users/me");
        assert_eq!(drive.service, "storage");
        assert_eq!(drive.uri.path(), "/api/files/1");
    }

    #[test]
    fn sends_oidc_discovery_without_rewriting() {
        let route = routes()
            .resolve(&"/.well-known/openid-configuration".parse().unwrap())
            .unwrap();

        assert_eq!(route.service, "padlock");
        assert_eq!(route.uri.path(), "/.well-known/openid-configuration");
    }

    #[test]
    fn sends_activity_pub_without_rewriting() {
        let route = routes()
            .resolve(&"/activitypub/actors/alice".parse().unwrap())
            .unwrap();

        assert_eq!(route.service, "sphere");
        assert_eq!(route.uri.path(), "/activitypub/actors/alice");
    }

    #[test]
    fn sends_tus_uploads_without_rewriting() {
        let route = routes().resolve(&"/api/tus/abc".parse().unwrap()).unwrap();

        assert_eq!(route.service, "storage");
        assert_eq!(route.uri.path(), "/api/tus/abc");
    }

    #[test]
    fn rejects_prefix_lookalikes() {
        let error = routes().resolve(&"/ringtone".parse().unwrap()).unwrap_err();

        assert_eq!(error, RouteError::NotFound);
    }

    #[test]
    fn reports_missing_websocket_upstream() {
        let mut table = routes();
        table.upstreams.websocket = None;

        assert_eq!(
            table.resolve(&"/ws".parse().unwrap()).unwrap_err(),
            RouteError::WebSocketNotConfigured
        );
    }
}
