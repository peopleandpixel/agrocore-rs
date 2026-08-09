use actix_web::{Error, FromRequest, HttpRequest};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use std::future::{Ready, ready};

#[derive(Debug, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub roles: Vec<String>,
}

pub struct AuthExtractor(pub AuthenticatedUser);

impl FromRequest for AuthExtractor {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_header = req.headers().get("authorization");
        match auth_header {
            Some(header_value) => {
                let header_str = match header_value.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return ready(Err(actix_web::error::ErrorUnauthorized(
                            "Invalid auth header",
                        )));
                    }
                };
                let token = match header_str.strip_prefix("Bearer ") {
                    Some(t) => t,
                    None => {
                        return ready(Err(actix_web::error::ErrorUnauthorized("No Bearer prefix")));
                    }
                };
                let secret = agrocore_shared::config::jwt_secret();
                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(secret.as_bytes()),
                    &Validation::new(Algorithm::HS256),
                ) {
                    Ok(token_data) => match (
                        parse_uuid(&token_data.claims.sub),
                        parse_uuid(&token_data.claims.tenant_id),
                    ) {
                        (Ok(user_id), Ok(tenant_id)) => {
                            ready(Ok(AuthExtractor(AuthenticatedUser {
                                user_id,
                                tenant_id,
                                roles: token_data.claims.roles,
                            })))
                        }
                        _ => ready(Err(actix_web::error::ErrorUnauthorized("Invalid UUID"))),
                    },
                    Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
                }
            }
            None => ready(Err(actix_web::error::ErrorUnauthorized(
                "Missing auth header",
            ))),
        }
    }
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
            },
            &EncodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
        )
        .expect("token")
    }

    #[test]
    fn auth_extractor_rejects_missing_header() {
        let req = TestRequest::default().to_http_request();
        let mut payload = Payload::None;
        let result = AuthExtractor::from_request(&req, &mut payload).into_inner();
        assert!(result.is_err());
    }

    #[test]
    fn auth_extractor_rejects_bad_prefix() {
        let req = TestRequest::default()
            .insert_header((header::AUTHORIZATION, "Token abc"))
            .to_http_request();
        let mut payload = Payload::None;
        let result = AuthExtractor::from_request(&req, &mut payload).into_inner();
        assert!(result.is_err());
    }

    #[test]
    fn auth_extractor_rejects_invalid_uuid_claims() {
        let token = signed_token("not-a-uuid", "also-not-a-uuid", vec!["Admin"]);
        let req = TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
            .to_http_request();
        let mut payload = Payload::None;
        let result = AuthExtractor::from_request(&req, &mut payload).into_inner();
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
        let mut payload = Payload::None;
        let extractor = AuthExtractor::from_request(&req, &mut payload)
            .into_inner()
            .expect("extractor");

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
        });
        assert!(admin.require_admin().is_ok());
        assert!(admin.require_manager().is_ok());

        let manager = AuthExtractor(AuthenticatedUser {
            user_id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            roles: vec![String::from("Manager")],
        });
        assert!(manager.require_admin().is_err());
        assert!(manager.require_manager().is_ok());

        let viewer = AuthExtractor(AuthenticatedUser {
            user_id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            roles: vec![String::from("Viewer"), String::from("Unknown")],
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
