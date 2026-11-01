//! Error utilities for Admin UI
//!
//! Provides user-friendly error messages for API failures,
//! preventing 500-style internal crashes from showing technical
//! details to users.

/// Converts an API error string to a user-friendly message.
/// This prevents technical stack traces from leaking to end users.
pub fn user_friendly_error(err: &str) -> String {
    if err.contains("401")
        || err.contains("unauthorized")
        || err.contains("Sitzung")
        || err.contains("abgelaufen")
    {
        "session_expired".to_string()
    } else if err.contains("403") || err.contains("forbidden") || err.contains("verweigert") {
        "access_denied".to_string()
    } else if err.contains("500") || err.contains("Server") {
        "server_error_generic".to_string()
    } else if err.contains("timeout") || err.contains("network") || err.contains("Verbindungs") {
        "connection_error".to_string()
    } else if err.contains("not found") || err.contains("404") {
        "not_found".to_string()
    } else {
        format!("error_unknown: {}", err)
    }
}

/// Safely handles an API Result, returning a user-friendly error message
/// instead of propagating the raw error string.
///
/// Usage in components:
/// ```ignore
/// let result = api::fetch_sites().await;
/// if let Err(err) = result {
///     let msg = error_boundary::user_friendly_error(&err);
///     // Display msg to user instead of raw error
/// }
/// ```
pub fn handle_api_result<T>(result: Result<T, String>) -> Result<T, String> {
    result.map_err(|e| user_friendly_error(&e))
}
