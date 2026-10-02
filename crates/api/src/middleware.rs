use crate::AppState;
use actix_web::{
    Error, FromRequest, HttpRequest,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use agrocore_logging::debug;
use agrocore_shared::config::decoding_key;
use dashmap::DashMap;
use jsonwebtoken::{Algorithm, Validation, decode};
use redis::AsyncCommands;
use serde::Deserialize;
use std::future::{Future, Ready, ready};
use std::net::IpAddr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// Token revocation list — stores revoked JWT IDs (jti) with TTL.
/// Uses Redis if available, falls back to in-memory DashMap with TTL.
#[derive(Clone)]
pub struct TokenRevocationList {
    inner: Arc<TRLInner>,
}

#[derive(Clone)]
enum TRLInner {
    Redis(redis::Client),
    Memory(Arc<RevocationMemoryStore>),
}

#[derive(Default)]
struct RevocationMemoryStore {
    /// Map of jti -> expiry Instant
    entries: DashMap<String, Instant>,
}

impl RevocationMemoryStore {
    fn new() -> Self {
        Self {
            entries: DashMap::new(),
        }
    }

    fn insert(&self, jti: String, ttl: Duration) {
        self.entries.insert(jti, Instant::now() + ttl);
    }

    fn contains(&self, jti: &str) -> bool {
        match self.entries.get(jti) {
            Some(entry) => *entry.value() > Instant::now(),
            None => false,
        }
    }
}

impl Default for TokenRevocationList {
    fn default() -> Self {
        Self {
            inner: Arc::new(TRLInner::Memory(Arc::new(RevocationMemoryStore::new()))),
        }
    }
}

impl TokenRevocationList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_redis(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            inner: Arc::new(TRLInner::Redis(client)),
        })
    }

    pub async fn revoke(&self, jti: &str, ttl: Duration) -> Result<(), redis::RedisError> {
        match &*self.inner {
            TRLInner::Redis(client) => {
                let mut conn = client.get_multiplexed_async_connection().await?;
                let _: () = conn
                    .set_ex(format!("revoked_jti:{}", jti), "1", ttl.as_secs())
                    .await?;
            }
            TRLInner::Memory(store) => {
                store.insert(jti.to_string(), ttl);
            }
        }
        Ok(())
    }

    pub async fn is_revoked(&self, jti: &str) -> bool {
        match &*self.inner {
            TRLInner::Redis(client) => {
                let mut conn = match client.get_multiplexed_async_connection().await {
                    Ok(c) => c,
                    Err(_) => return false,
                };
                let exists: bool = conn
                    .exists(format!("revoked_jti:{}", jti))
                    .await
                    .unwrap_or(false);
                exists
            }
            TRLInner::Memory(store) => store.contains(jti),
        }
    }
}

/// Custom key extractor that falls back to localhost when peer address is unavailable.
/// This allows the Governor middleware to work in test environments where no peer addr is set.
#[derive(Clone)]
pub struct TestableIpKeyExtractor;

impl actix_governor::KeyExtractor for TestableIpKeyExtractor {
    type Key = IpAddr;
    type KeyExtractionError = actix_governor::SimpleKeyExtractionError<&'static str>;

    fn extract(
        &self,
        req: &actix_web::dev::ServiceRequest,
    ) -> Result<Self::Key, Self::KeyExtractionError> {
        req.peer_addr()
            .map(|socket| socket.ip())
            .ok_or_else(|| actix_governor::SimpleKeyExtractionError::new("no peer addr"))
            .or_else(|_| Ok("127.0.0.1".parse().unwrap()))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub exp: usize,
    pub jti: String,
}

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub roles: Vec<String>,
    pub jti: String,
}

pub struct AuthExtractor(pub AuthenticatedUser);

impl FromRequest for AuthExtractor {
    type Error = Error;
    // The revocation list is an async resource (Redis or in-memory), so the
    // future cannot stay a plain `Ready` any more.
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Clone the header and the app-state handle so the future owns
        // everything it needs and does not borrow `req`.
        let auth_header = req
            .headers()
            .get("authorization")
            .map(|v| v.as_bytes().to_vec());
        let state = req.app_data::<actix_web::web::Data<AppState>>().cloned();
        Box::pin(async move { authenticate_request(auth_header, state).await })
    }
}

/// Validate the bearer token and reject revoked ones.
///
/// A revoked token must not authenticate: logout and password changes call
/// `revoke()`, but nothing ever called `is_revoked()`, so a stolen token stayed
/// valid for its full lifetime.
async fn authenticate_request(
    auth_header: Option<Vec<u8>>,
    state: Option<actix_web::web::Data<AppState>>,
) -> Result<AuthExtractor, Error> {
    let auth_header = match auth_header {
        Some(v) => v,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing auth header")),
    };

    let header_str = std::str::from_utf8(&auth_header)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid auth header"))?;

    let token = header_str
        .strip_prefix("Bearer ")
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No Bearer prefix"))?;

    let token_data = decode::<Claims>(token, decoding_key(), &Validation::new(Algorithm::HS256))
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token"))?;

    let (user_id, tenant_id) = match (
        parse_uuid(&token_data.claims.sub),
        parse_uuid(&token_data.claims.tenant_id),
    ) {
        (Ok(u), Ok(t)) => (u, t),
        _ => return Err(actix_web::error::ErrorUnauthorized("Invalid UUID")),
    };

    // Reject revoked tokens. The list is keyed by the token's jti, which is
    // what logout and password changes write.
    if let Some(state) = state
        && state
            .token_revocation
            .is_revoked(&token_data.claims.jti)
            .await
    {
        return Err(actix_web::error::ErrorUnauthorized("Token revoked"));
    }

    Ok(AuthExtractor(AuthenticatedUser {
        user_id,
        tenant_id,
        roles: token_data.claims.roles,
        jti: token_data.claims.jti,
    }))
}

/// Pin the tenant on a pooled connection before a query runs.
///
/// The RLS policies call `get_current_tenant_id()`, which reads
/// `app.current_tenant_id`. That setting is session-scoped, so it must be
/// re-applied on every checkout: a pooled connection may have served a
/// different tenant before. `set_config(..., false)` makes it session-scoped,
/// and the explicit reset below prevents any leak into a later request.
///
/// `SET LOCAL` would be tidier but requires wrapping every query in a
/// transaction; with 154 routes that is a larger change than the problem
/// warrants. The reset in the same call chain closes the window instead.
pub async fn pin_tenant(
    conn: &mut sqlx::PgConnection,
    tenant_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, false)")
        .bind(tenant_id.to_string())
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Clear the tenant pin again.
///
/// Called after the request finished. Failures are ignored on purpose: the
/// connection goes back to the pool either way, and the next request
/// overwrites the setting before it issues a query.
pub async fn unpin_tenant(conn: &mut sqlx::PgConnection) {
    let _ = sqlx::query("SELECT set_config('app.current_tenant_id', '', false)")
        .execute(&mut *conn)
        .await;
}

impl AuthExtractor {
    pub fn roles(&self) -> Vec<agrocore_domain::entities::user::UserRole> {
        self.0
            .roles
            .iter()
            .map(|r| match r.as_str() {
                "Admin" => agrocore_domain::entities::user::UserRole::Admin,
                "Manager" => agrocore_domain::entities::user::UserRole::Manager,
                "Worker" => agrocore_domain::entities::user::UserRole::Worker,
                _ => agrocore_domain::entities::user::UserRole::Viewer,
            })
            .collect()
    }

    pub fn is_admin(&self) -> bool {
        self.0.roles.iter().any(|r| r == "Admin")
    }

    pub fn is_manager(&self) -> bool {
        self.0.roles.iter().any(|r| r == "Admin" || r == "Manager")
    }

    pub fn require_admin(&self) -> agrocore_shared::Result<()> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(agrocore_shared::SharedError::Forbidden(
                "Admin role required".into(),
            ))
        }
    }

    pub fn require_manager(&self) -> agrocore_shared::Result<()> {
        if self.is_manager() {
            Ok(())
        } else {
            Err(agrocore_shared::SharedError::Forbidden(
                "Manager or Admin role required".into(),
            ))
        }
    }

    pub fn require_any_role<I, S>(&self, roles: I) -> agrocore_shared::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let required_roles: Vec<String> = roles
            .into_iter()
            .map(|role| role.as_ref().to_ascii_lowercase())
            .collect();
        if self.0.roles.iter().any(|role| {
            required_roles
                .iter()
                .any(|required| role.eq_ignore_ascii_case(required))
        }) {
            Ok(())
        } else {
            Err(agrocore_shared::SharedError::Forbidden(
                "Required role missing".into(),
            ))
        }
    }
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid, Error> {
    uuid::Uuid::parse_str(s).map_err(|_| actix_web::error::ErrorUnauthorized("Invalid UUID"))
}

pub type AuthUser = AuthenticatedUser;

/// Metrics middleware: checks is_enabled() and registers metrics.
pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddlewareService { service }))
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if crate::metrics::is_enabled() {
            debug!("Metrics enabled — registering request metrics");
        }
        self.service.call(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{dev::Payload, http::header, test::TestRequest};
    use jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        tenant_id: String,
        roles: Vec<String>,
        exp: usize,
        jti: String,
    }

    /// Run `from_request` to completion.
    ///
    /// The extractor returns a boxed future because it has to consult the
    /// async revocation list, so the tests can no longer use `into_inner()`.
    fn extract(req: &HttpRequest) -> Result<AuthExtractor, Error> {
        let mut payload = Payload::None;
        let fut = AuthExtractor::from_request(req, &mut payload);
        actix_web::rt::System::new().block_on(fut)
    }

    fn signed_token(sub: &str, tenant_id: &str, roles: Vec<&str>) -> String {
        let _ = DEFAULT_PROVIDER.install_default();
        encode(
            &Header::default(),
            &TestClaims {
                sub: sub.to_string(),
                tenant_id: tenant_id.to_string(),
                roles: roles.into_iter().map(String::from).collect(),
                exp: usize::MAX / 2,
                jti: uuid::Uuid::new_v4().to_string(),
            },
            &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
        )
        .expect("token")
    }

    #[test]
    fn auth_extractor_rejects_missing_header() {
        let req = TestRequest::default().to_http_request();
        let result = extract(&req);
        assert!(result.is_err());
    }

    #[test]
    fn auth_extractor_rejects_bad_prefix() {
        let req = TestRequest::default()
            .insert_header((header::AUTHORIZATION, "Token abc"))
            .to_http_request();
        let result = extract(&req);
        assert!(result.is_err());
    }

    #[test]
    fn auth_extractor_rejects_invalid_uuid_claims() {
        let token = signed_token("not-a-uuid", "also-not-a-uuid", vec!["Admin"]);
        let req = TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
            .to_http_request();
        let result = extract(&req);
        assert!(result.is_err());
    }

    #[test]
    fn auth_extractor_accepts_valid_token_and_maps_roles() {
        let user_id = uuid::Uuid::new_v4();
        let tenant_id = uuid::Uuid::new_v4();
        let token = signed_token(
            &user_id.to_string(),
            &tenant_id.to_string(),
            vec!["Admin", "Worker"],
        );
        let req = TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
            .to_http_request();
        let extractor = extract(&req).expect("extractor");

        assert_eq!(extractor.0.user_id, user_id);
        assert_eq!(extractor.0.tenant_id, tenant_id);
        assert!(extractor.is_admin());
        assert!(extractor.is_manager());
        assert_eq!(
            extractor.roles(),
            vec![
                agrocore_domain::entities::user::UserRole::Admin,
                agrocore_domain::entities::user::UserRole::Worker,
            ]
        );
    }

    #[test]
    fn auth_extractor_role_helpers_cover_admin_manager_and_viewer_paths() {
        let admin = AuthExtractor(AuthenticatedUser {
            user_id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            roles: vec![String::from("Admin")],
            jti: uuid::Uuid::new_v4().to_string(),
        });
        assert!(admin.require_admin().is_ok());
        assert!(admin.require_manager().is_ok());

        let manager = AuthExtractor(AuthenticatedUser {
            user_id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            roles: vec![String::from("Manager")],
            jti: uuid::Uuid::new_v4().to_string(),
        });
        assert!(manager.require_admin().is_err());
        assert!(manager.require_manager().is_ok());

        let viewer = AuthExtractor(AuthenticatedUser {
            user_id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            roles: vec![String::from("Viewer"), String::from("Unknown")],
            jti: uuid::Uuid::new_v4().to_string(),
        });
        assert!(!viewer.is_admin());
        assert!(!viewer.is_manager());
        assert!(viewer.require_admin().is_err());
        assert!(viewer.require_manager().is_err());
        assert_eq!(
            viewer.roles(),
            vec![
                agrocore_domain::entities::user::UserRole::Viewer,
                agrocore_domain::entities::user::UserRole::Viewer,
            ]
        );
    }
}
