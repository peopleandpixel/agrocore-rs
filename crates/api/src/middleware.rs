use actix_web::{FromRequest, HttpRequest, Error};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::Deserialize;
use std::future::{ready, Ready};

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
                    Err(_) => return ready(Err(actix_web::error::ErrorUnauthorized("Invalid auth header"))),
                };
                let token = match header_str.strip_prefix("Bearer ") {
                    Some(t) => t,
                    None => return ready(Err(actix_web::error::ErrorUnauthorized("No Bearer prefix"))),
                };
                let _secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".into());
                let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".into());
                match decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256)) {
                    Ok(token_data) => match (parse_uuid(&token_data.claims.sub), parse_uuid(&token_data.claims.tenant_id)) {
                        (Ok(user_id), Ok(tenant_id)) => ready(Ok(AuthExtractor(AuthenticatedUser { 
                            user_id, 
                            tenant_id, 
                            roles: token_data.claims.roles 
                        }))),
                        _ => ready(Err(actix_web::error::ErrorUnauthorized("Invalid UUID"))),
                    },
                    Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
                }
            }
            None => ready(Err(actix_web::error::ErrorUnauthorized("Missing auth header"))),
        }
    }
}

fn parse_uuid(s: &str) -> Result<uuid::Uuid, Error> {
    uuid::Uuid::parse_str(s).map_err(|_| actix_web::error::ErrorUnauthorized("Invalid UUID"))
}

pub type AuthUser = AuthenticatedUser;
