use agrocore_domain::entities::user::{User, UserRole};
use agrocore_shared::config::jwt_secret;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::Serialize;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

pub fn generate_jwt(user: &User) -> anyhow::Result<String> {
    let expiration = Utc::now() + Duration::minutes(30);
    
    let roles = user.roles.iter().map(|r| match r {
        UserRole::Admin => "Admin".to_string(),
        UserRole::Manager => "Manager".to_string(),
        UserRole::Worker => "Worker".to_string(),
        UserRole::Viewer => "Viewer".to_string(),
        UserRole::Custom(id) => format!("Custom:{}", id),
    }).collect();

    let claims = Claims {
        sub: user.id.to_string(),
        tenant_id: user.tenant_id.to_string(),
        roles,
        exp: expiration.timestamp() as usize,
    };

    let secret = jwt_secret();
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes())
    )?;

    Ok(token)
}
