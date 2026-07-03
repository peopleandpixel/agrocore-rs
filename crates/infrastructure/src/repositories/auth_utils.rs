use agrocore_domain::entities::user::User;
use agrocore_shared::{Result, SharedError};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub fn hash_password(pw: &str) -> Result<String> {
    use argon2::PasswordHasher;
    argon2::Argon2::default()
        .hash_password(pw.as_bytes())
        .map(|h| h.to_string())
        .map_err(|e| SharedError::Internal(e.to_string()))
}

pub fn verify_password(pw: &str, hash: &str) -> Result<()> {
    use argon2::PasswordHash;
    use argon2::PasswordVerifier;
    let parsed = PasswordHash::new(hash).map_err(|e| SharedError::Internal(e.to_string()))?;
    argon2::Argon2::default()
        .verify_password(pw.as_bytes(), &parsed)
        .map_err(|_| SharedError::Unauthorized("Invalid credentials".into()))
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    tenant_id: String,
    roles: Vec<String>,
    exp: usize,
}

pub fn generate_jwt(u: &User) -> Result<String> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    let secret = agrocore_shared::config::jwt_secret();
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;
    let roles: Vec<String> = u.roles.iter().map(|r| format!("{:?}", r)).collect();
    let claims = Claims {
        sub: u.id.to_string(),
        tenant_id: u.tenant_id.to_string(),
        roles,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| SharedError::Internal(e.to_string()))
}
