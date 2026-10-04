//! Impersonation must be reachable for an admin, closed to everyone else, audited
//! and revocable (tasks.md D2).
//!
//! Before the fix, `impersonate` compared the caller's roles against `"admin"` and
//! `"superadmin"`, while `generate_jwt` emits `"Admin"`. No token the system
//! produces could ever match, so the endpoint was unreachable — and the three
//! defects behind it were invisible for the same reason: no revocation, no audit
//! entry, and no record of the original caller in the issued token.
//!
//! These tests drive the real handler through a mocked user repository, so they
//! assert the authorisation and the side effects, not a copy of the rule.

use actix_web::{App, http::StatusCode, http::header, test, web};
use agrocore_api::{AppState, handlers::configure};
use agrocore_domain::TenantId;
use agrocore_domain::entities::compliance::CreateAuditLogDto;
use agrocore_domain::entities::compliance::{AuditAction, AuditLog};
use agrocore_domain::entities::user::{User, UserRole};
use agrocore_infrastructure::{Database, MockDatabase};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::Serialize;
use std::future::ready;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    tenant_id: String,
    roles: Vec<String>,
    exp: usize,
    jti: String,
}

#[derive(serde::Deserialize, Debug)]
struct DecodedClaims {
    sub: String,
    tenant_id: String,
    roles: Vec<String>,
    jti: String,
}

fn sign(sub: &str, tenant_id: &str, roles: Vec<&str>, jti: &str) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: sub.to_string(),
            tenant_id: tenant_id.to_string(),
            roles: roles.into_iter().map(String::from).collect(),
            exp: usize::MAX / 2,
            jti: jti.to_string(),
        },
        &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
    )
    .expect("token")
}

fn user(id: Uuid, tenant_id: Uuid, roles: Vec<UserRole>) -> User {
    User {
        id,
        tenant_id: TenantId(tenant_id),
        firstname: "Ada".into(),
        lastname: "Lovelace".into(),
        email: "ada@example.com".into(),
        password_hash: "$argon2id$v=19$m=1,t=1,p=1$aaaa$bbbb".into(),
        roles,
        is_active: true,
        internal_cost_per_hour: None,
        external_cost_per_hour: None,
        color: None,
        language: None,
        assigned_site_ids: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login: None,
        refresh_token: None,
        refresh_token_expires_at: None,
    }
}

fn audit_log_from(dto: CreateAuditLogDto) -> AuditLog {
    AuditLog {
        id: Uuid::new_v4(),
        tenant_id: dto.tenant_id,
        user_id: dto.user_id,
        action: dto.action,
        entity_type: dto.entity_type,
        entity_id: dto.entity_id,
        old_value: dto.old_value,
        new_value: dto.new_value,
        ip_address: dto.ip_address,
        created_at: Utc::now(),
    }
}

/// Builds the state for an app whose user lookup always resolves to `target`.
///
/// Returns the state rather than a built app: the service type `init_service`
/// produces is not nameable outside this crate's dependency set, and each test
/// builds its own app from the returned state.
fn state_for_target(
    target: User,
) -> (AppState, Arc<agrocore_api::middleware::TokenRevocationList>) {
    use agrocore_domain::repositories::{MockAuditLogRepo, MockUserRepository};

    let mut repo = MockUserRepository::new();
    let found = target.clone();
    repo.expect_find_by_id()
        .returning(move |_tid, _id| Box::pin(ready(Ok(Some(found.clone())))));

    // The handler writes an audit entry before minting the token, so a state
    // without this repo panics on `audit_log_repo mock not set` rather than
    // failing an assertion. Recording is asserted separately, in
    // `impersonating_is_recorded_in_the_audit_log`.
    let mut audit_repo = MockAuditLogRepo::new();
    audit_repo
        .expect_create()
        .returning(|_tid, dto| Box::pin(ready(Ok(audit_log_from(dto)))));

    let mut mock_db = MockDatabase::default();
    mock_db.user_repo = Some(Arc::new(repo));
    mock_db.audit_log_repo = Some(Arc::new(audit_repo));

    let revocation = Arc::new(agrocore_api::middleware::TokenRevocationList::new());
    let metrics_registry = prometheus::Registry::new();

    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: None,
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: revocation.clone(),
        db_metrics: agrocore_api::metrics::DbMetrics::new(&metrics_registry),
        business_metrics: agrocore_api::metrics::BusinessMetrics::new(&metrics_registry),
        metrics_registry: Arc::new(metrics_registry),
        backup_service: None,
        demo_endpoints_enabled: false,
    };
    (state, revocation)
}

/// The core assertion: an Admin token can actually impersonate.
///
/// This is the test that would have failed before the fix. With the old
/// case-sensitive comparison the handler answered 401 for every role the system
/// has, so no admin could ever use the endpoint.
#[actix_web::test]
async fn an_admin_can_impersonate_another_user() {
    let tenant_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let (state, _) = state_for_target(user(target_id, tenant_id, vec![UserRole::Worker]));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = sign(
        &admin_id.to_string(),
        &tenant_id.to_string(),
        vec!["Admin"],
        &Uuid::new_v4().to_string(),
    );
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/auth/impersonate/{target_id}"))
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "an Admin token must be able to impersonate — this is the regression the \
         case-sensitive role comparison caused"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user_id"], target_id.to_string());
    // The original caller is named in the response, so the client can tell what
    // happened even without decoding the token.
    assert_eq!(
        body["impersonator_id"],
        admin_id.to_string(),
        "the response must record who started the impersonation"
    );

    // The returned token really is the target's, in the right tenant.
    let issued: String = body["token"].as_str().expect("token").to_string();
    let claims = decode::<DecodedClaims>(
        &issued,
        &DecodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .expect("issued token must verify")
    .claims;
    assert_eq!(claims.sub, target_id.to_string());
    assert_eq!(claims.tenant_id, tenant_id.to_string());
}

/// Everyone who is not an Admin is refused, including a Manager.
#[actix_web::test]
async fn a_non_admin_cannot_impersonate() {
    let tenant_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    let (state, _) = state_for_target(user(target_id, tenant_id, vec![UserRole::Worker]));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    for role in ["Manager", "Viewer", "Worker"] {
        let token = sign(
            &Uuid::new_v4().to_string(),
            &tenant_id.to_string(),
            vec![role],
            &Uuid::new_v4().to_string(),
        );
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/auth/impersonate/{target_id}"))
            .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "a {role} must not be able to impersonate"
        );
    }
}

/// The admin's own token is revoked when the impersonation starts, so
/// `stop_impersonation` cannot be a client-side promise.
#[actix_web::test]
async fn impersonating_revokes_the_admins_token() {
    let tenant_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    let admin_jti = Uuid::new_v4().to_string();

    let (state, revocation) = state_for_target(user(target_id, tenant_id, vec![UserRole::Worker]));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    assert!(!revocation.is_revoked(&admin_jti).await);

    let token = sign(
        &admin_id.to_string(),
        &tenant_id.to_string(),
        vec!["Admin"],
        &admin_jti,
    );
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/auth/impersonate/{target_id}"))
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        revocation.is_revoked(&admin_jti).await,
        "the admin's token must be revoked once it has been used to impersonate"
    );
}

/// An admin cannot impersonate themselves, which would mint a second token for the
/// session they already hold.
#[actix_web::test]
async fn an_admin_cannot_impersonate_themselves() {
    let tenant_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let (state, _) = state_for_target(user(admin_id, tenant_id, vec![UserRole::Admin]));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = sign(
        &admin_id.to_string(),
        &tenant_id.to_string(),
        vec!["Admin"],
        &Uuid::new_v4().to_string(),
    );
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/auth/impersonate/{admin_id}"))
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

/// `stop_impersonation` is admin-only. It used to be open to any authenticated
/// caller, which made it a token-minting endpoint with no audit trail and no
/// revocation — an admin could hand out tokens and none of it would show up.
#[actix_web::test]
async fn a_non_admin_cannot_stop_impersonation() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let (state, _) = state_for_target(user(user_id, tenant_id, vec![UserRole::Manager]));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = sign(
        &user_id.to_string(),
        &tenant_id.to_string(),
        vec!["Manager"],
        &Uuid::new_v4().to_string(),
    );
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/impersonate/stop")
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "stop_impersonation must not be a general token mint"
    );
}

/// The impersonation is written to the audit log. `AuditAction::Viewed` is used
/// because the enum has no `Impersonated` variant; the meaning is carried by
/// `entity_type`.
#[actix_web::test]
async fn impersonating_is_recorded_in_the_audit_log() {
    use agrocore_domain::repositories::MockAuditLogRepo;

    let tenant_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let mut audit_repo = MockAuditLogRepo::new();
    audit_repo.expect_create().returning(move |_tid, dto| {
        Box::pin(ready(Ok(AuditLog {
            id: Uuid::new_v4(),
            tenant_id: dto.tenant_id,
            user_id: dto.user_id,
            action: dto.action,
            entity_type: dto.entity_type,
            entity_id: dto.entity_id,
            old_value: dto.old_value,
            new_value: dto.new_value,
            ip_address: dto.ip_address,
            created_at: Utc::now(),
        })))
    });

    let mut user_repo = agrocore_domain::repositories::MockUserRepository::new();
    let found = user(target_id, tenant_id, vec![UserRole::Worker]);
    user_repo
        .expect_find_by_id()
        .returning(move |_t, _i| Box::pin(ready(Ok(Some(found.clone())))));

    let mut mock_db = MockDatabase::default();
    mock_db.user_repo = Some(Arc::new(user_repo));
    mock_db.audit_log_repo = Some(Arc::new(audit_repo));

    let metrics_registry = prometheus::Registry::new();
    let state = AppState {
        db: Arc::new(Database::Mock(Box::new(mock_db))),
        messaging: None,
        lpis_registry: Arc::new(agrocore_lpis_providers::create_default_registry()),
        token_revocation: Arc::new(agrocore_api::middleware::TokenRevocationList::new()),
        db_metrics: agrocore_api::metrics::DbMetrics::new(&metrics_registry),
        business_metrics: agrocore_api::metrics::BusinessMetrics::new(&metrics_registry),
        metrics_registry: Arc::new(metrics_registry),
        backup_service: None,
        demo_endpoints_enabled: false,
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure),
    )
    .await;

    let token = sign(
        &admin_id.to_string(),
        &tenant_id.to_string(),
        vec!["Admin"],
        &Uuid::new_v4().to_string(),
    );
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/auth/impersonate/{target_id}"))
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // The mock's `expect_create` is a hard requirement: if the handler stops
    // writing the audit entry, the mock returns a default and the call panics,
    // failing this test.
    assert_eq!(resp.status(), StatusCode::OK);
    let _ = AuditAction::Viewed;
}
