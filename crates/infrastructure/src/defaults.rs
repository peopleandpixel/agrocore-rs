/// Statische Default-Werte für häufig verwendete Strings
/// Vermeidet wiederholte String-Allocationen zur Optimierungszeit

#[inline]
pub const fn default_mongodb_uri() -> &'static str {
    "mongodb://localhost:27017"
}

#[inline]
pub const fn default_nats_url() -> &'static str {
    "nats://localhost:4222"
}

#[inline]
pub const fn default_bind_addr() -> &'static str {
    "0.0.0.0"
}
