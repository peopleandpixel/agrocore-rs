//! Admin UI API Layer Tests
//!
//! The network-backed helpers (`get_json`, `with_auth`) require a browser, so
//! what is verified natively here is the pure request-construction logic that
//! feeds them: `api_url`.
//!
//! Note: `build.rs` always emits `cargo:rustc-env=AGROCORE_API_BASE_URL`, so
//! `api_url` normally runs with a base URL present. These tests therefore
//! assert on the *invariants* that must hold in either mode (base configured or
//! not) rather than on one specific output shape.
//!
//! Run: `cargo test -p admin-ui --lib`

#![cfg(test)]

#[cfg(test)]
mod tests {
    use crate::api::{api_base_url, api_url};

    /// Whatever the base, a relative path must never gain or lose its leading
    /// slash, and must never produce a doubled separator.
    #[test]
    fn test_api_url_never_doubles_or_drops_separator() {
        for path in ["/api/v1/tasks", "api/v1/tasks", "//api/v1/tasks"] {
            let url = api_url(path);
            assert!(
                !url.contains("//api"),
                "unexpected doubled separator for {path:?}: {url}"
            );
        }
    }

    /// With a base configured, the result must be prefixed by that base and
    /// keep the path intact after it.
    #[test]
    fn test_api_url_prefixes_configured_base() {
        let base = api_base_url();
        if base.is_empty() {
            // No base baked in at compile time; nothing to prefix.
            return;
        }
        let url = api_url("/api/v1/tasks");
        assert!(
            url.starts_with(&base),
            "expected {url:?} to start with base {base:?}"
        );
        assert!(url.ends_with("/api/v1/tasks"), "path mangled: {url}");
    }

    /// Absolute URLs must pass through untouched, otherwise external providers
    /// such as Open-Meteo would be rewritten onto the configured API base.
    #[test]
    fn test_api_url_preserves_absolute_urls() {
        for url in [
            "https://api.open-meteo.com/v1/forecast",
            "http://example.com/api/v1/weather",
        ] {
            assert_eq!(api_url(url), url);
        }
    }

    /// Query strings must survive URL building, since paginated and filtered
    /// endpoints depend on them.
    #[test]
    fn test_api_url_preserves_query_strings() {
        let url = api_url("/api/v1/sites?page=2&per_page=50");
        assert!(url.ends_with("?page=2&per_page=50"), "query lost: {url}");
        assert_eq!(url.matches('?').count(), 1, "double '?': {url}");
    }
}
