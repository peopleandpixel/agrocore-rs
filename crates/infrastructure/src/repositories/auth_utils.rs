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
        .checked_add_signed(chrono::Duration::minutes(30))
        .ok_or_else(|| SharedError::Internal("JWT expiry timestamp overflow".into()))?
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

#[cfg(test)]
mod tests {
    use super::*;
    use agrocore_domain::entities::tenant::TenantId;
    use agrocore_domain::entities::user::{User, UserRole};
    use chrono::Utc;
    use jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER;
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestClaims {
        sub: String,
        tenant_id: String,
        roles: Vec<String>,
        exp: usize,
    }

    fn sample_user() -> User {
        let now = Utc::now();
        User {
            id: Uuid::new_v4(),
            tenant_id: TenantId::new_v4(),
            firstname: String::from("Alice"),
            lastname: String::from("Smith"),
            email: String::from("alice@example.com"),
            password_hash: String::from("hash"),
            roles: vec![UserRole::Admin, UserRole::Manager],
            is_active: true,
            internal_cost_per_hour: None,
            external_cost_per_hour: None,
            color: None,
            language: Some(String::from("de")),
            assigned_site_ids: None,
            last_login: None,
            refresh_token: None,
            refresh_token_expires_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn password_hash_roundtrip_and_rejects_wrong_password() {
        let hash = hash_password("correct horse battery staple").expect("hash");
        assert!(verify_password("correct horse battery staple", &hash).is_ok());
        assert!(verify_password("wrong password", &hash).is_err());
    }

    #[test]
    fn generate_jwt_roundtrips_with_shared_secret() {
        let _ = DEFAULT_PROVIDER.install_default();
        let user = sample_user();
        let token = generate_jwt(&user).expect("jwt");
        let decoded = decode::<TestClaims>(
            &token,
            &DecodingKey::from_secret(agrocore_shared::config::jwt_secret().as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .expect("decode");

        assert_eq!(decoded.claims.sub, user.id.to_string());
        assert_eq!(decoded.claims.tenant_id, user.tenant_id.to_string());
        assert_eq!(
            decoded.claims.roles,
            vec![String::from("Admin"), String::from("Manager")]
        );
        assert!(decoded.claims.exp > 0);
    }
}
