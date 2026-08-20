//! Error utilities for Admin UI
//!
//! Provides user-friendly error messages for API failures,
//! preventing 500-style internal crashes from showing technical
//! details to users.

#![allow(dead_code)]

/// Converts an API error string to a user-friendly message.
/// This prevents technical stack traces from leaking to end users.
pub fn user_friendly_error(err: &str) -> String {
    if err.contains("401") || err.contains("unauthorized") || err.contains("Sitzung") {
        "Ihre Sitzung ist abgelaufen. Bitte melden Sie sich erneut an.".to_string()
    } else if err.contains("403") || err.contains("forbidden") {
        "Zugriff verweigert. Kontaktieren Sie Ihren Administrator.".to_string()
    } else if err.contains("500") || err.contains("status") {
        "Ein Server-Fehler ist aufgetreten. Bitte versuchen Sie es später erneut.".to_string()
    } else if err.contains("timeout") || err.contains("network") {
        "Verbindungsfehler. Bitte prüfen Sie Ihre Internetverbindung.".to_string()
    } else if err.contains("not found") || err.contains("404") {
        "Die angeforderten Daten wurden nicht gefunden.".to_string()
    } else {
        format!("Fehler beim Laden der Daten: {}", err)
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
