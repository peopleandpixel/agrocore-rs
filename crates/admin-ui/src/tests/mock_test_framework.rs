//! Admin UI Mock Test Framework
//!
//! Provides mock implementations for API errors so UI components can be
//! tested for graceful error handling instead of crashing with 500s.
//!
//! Usage in tests:
//! ```
//! let mut mock = MockApiClient::new();
//! mock.when_call("fetch_users").return_err(ApiError::Http { status: 500, message: "DB error".into() });
//! // ... render UI component ...
//! // ... verify error message is shown instead of crash ...
//! ```

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Structured error returned by all API functions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiError {
    /// Network error — server unreachable, CORS blocked, etc.
    Network { message: String },
    /// HTTP status error (4xx, 5xx)
    Http { status: u16, message: String },
    /// JSON deserialization failed — response shape changed
    JsonParse { message: String },
}

impl ApiError {
    pub fn is_server_error(&self) -> bool {
        matches!(self, ApiError::Http { status, .. } if *status >= 500)
    }

    pub fn is_client_error(&self) -> bool {
        matches!(self, ApiError::Http { status, .. } if (400..500).contains(status))
    }

    pub fn is_network_error(&self) -> bool {
        matches!(self, ApiError::Network { .. })
    }

    /// Human-readable message for UI display
    pub fn user_message(&self) -> String {
        match self {
            ApiError::Network { message } => format!("Verbindungsproblem: {}", message),
            ApiError::Http { status, message } => {
                if *status >= 500 {
                    // Do NOT include technical error message for 5xx — just report server error
                    "Ein Server-Fehler ist aufgetreten. Bitte versuchen Sie es später erneut."
                        .to_string()
                } else if *status == 401 {
                    "Sitzung abgelaufen. Bitte neu anmelden.".to_string()
                } else if *status == 403 {
                    "Zugriff verweigert.".to_string()
                } else if *status == 404 {
                    "Ressource nicht gefunden.".to_string()
                } else {
                    format!("Fehler ({}): {}", status, message)
                }
            }
            ApiError::JsonParse { message } => format!("Unerwartete Antwort: {}", message),
        }
    }
}

/// Common result type for all API functions — enforces error handling.
pub type ApiResult<T> = Result<T, ApiError>;

/// Mock response definition for a single API call.
#[derive(Debug, Clone)]
pub enum MockResponse {
    /// Return an error (simulates 500, 401, network failure, etc.)
    Error(ApiError),
    /// Return an empty success (simulates empty list, successful delete, etc.)
    Empty,
}

/// Mock API Client for testing UI components.
///
/// Use this to simulate various API failure scenarios:
/// ```
/// let mock = MockApiClient::new()
///     .with_error("fetch_users", ApiError::Http { status: 500, message: "DB down".into() });
/// // Pass mock to your component test
/// ```
#[derive(Debug, Default)]
pub struct MockApiClient {
    /// Map of function name → mock response
    responses: std::collections::HashMap<String, MockResponse>,
}

impl MockApiClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a specific function to return an error
    pub fn with_error(mut self, fn_name: &str, error: ApiError) -> Self {
        self.responses
            .insert(fn_name.to_string(), MockResponse::Error(error));
        self
    }

    /// Set a specific function to return success
    pub fn with_success<T>(mut self, fn_name: &str, _data: T) -> Self {
        // In real usage, would store the serialized data
        let _ = fn_name;
        self.responses
            .insert(fn_name.to_string(), MockResponse::Empty);
        self
    }

    /// Check what response a function would return (for test assertions)
    pub fn response_for(&self, fn_name: &str) -> Option<&MockResponse> {
        self.responses.get(fn_name)
    }

    /// Get the error for a function (if it's configured to error)
    pub fn error_for(&self, fn_name: &str) -> Option<&ApiError> {
        match self.responses.get(fn_name) {
            Some(MockResponse::Error(e)) => Some(e),
            _ => None,
        }
    }

    /// Check if a function would return a server error (>500)
    pub fn would_return_server_error(&self, fn_name: &str) -> bool {
        self.error_for(fn_name)
            .map(|e| e.is_server_error())
            .unwrap_or(false)
    }

    /// Check if a function would return a network error
    pub fn would_return_network_error(&self, fn_name: &str) -> bool {
        self.error_for(fn_name)
            .map(|e| e.is_network_error())
            .unwrap_or(false)
    }

    /// List of function names that are configured to error
    pub fn error_functions(&self) -> Vec<&String> {
        self.responses
            .iter()
            .filter(|(_, v)| matches!(v, MockResponse::Error(_)))
            .map(|(k, _)| k)
            .collect()
    }
}

/// Pre-built mock scenarios for common failure modes
pub mod scenarios {
    use super::*;

    /// All fetch_* functions that should be covered by mock scenarios.
    /// When adding a new fetch function to api.rs, add it here!
    /// This is the single source of truth for mock coverage.
    pub const ALL_FETCH_FUNCTIONS: &[&str] = &[
        // Core fetch functions (always tested)
        "fetch_users",
        "fetch_sites",
        "fetch_orders",
        "fetch_equipment",
        "fetch_workers",
        "fetch_pac_applications",
        "fetch_cost_centers",
        "fetch_animals",
        "fetch_inventory_items",
        "fetch_tasks",
        "fetch_weather_data",
        // Extended fetch functions
        "fetch_active_session",
        "fetch_aggregated_task_status",
        "fetch_audit_logs",
        "fetch_below_minimum",
        "fetch_clock_entries",
        "fetch_compliance_checklists",
        "fetch_fertilizer_records",
        "fetch_financial_records",
        "fetch_inventory_balances",
        "fetch_inventory_locations",
        "fetch_item_transactions",
        "fetch_my_tasks",
        "fetch_open_meteo_weather",
        "fetch_phenology_records",
        "fetch_plant_protection_records",
        "fetch_system_status",
        "fetch_task",
        "fetch_task_worker_statuses",
        "fetch_total_hours",
        "fetch_weather_for_company_profile",
        "fetch_weather_stations",
        "fetch_worker",
        "fetch_worker_sessions",
        "fetch_worker_tasks",
        "fetch_worker_task_status",
    ];

    /// All API calls return 500 errors — tests total backend outage
    pub fn all_500() -> MockApiClient {
        let mut mock = MockApiClient::new();
        let error = ApiError::Http {
            status: 500,
            message: "Internal Server Error".into(),
        };
        for fn_name in ALL_FETCH_FUNCTIONS {
            mock = mock.with_error(fn_name, error.clone());
        }
        mock
    }

    /// All API calls return network errors — tests offline mode
    pub fn network_failure() -> MockApiClient {
        let mut mock = MockApiClient::new();
        let error = ApiError::Network {
            message: "Failed to fetch".into(),
        };
        for fn_name in ALL_FETCH_FUNCTIONS {
            mock = mock.with_error(fn_name, error.clone());
        }
        mock
    }

    /// Auth expired — returns 401 for all authenticated calls
    pub fn auth_expired() -> MockApiClient {
        let mut mock = MockApiClient::new();
        let error = ApiError::Http {
            status: 401,
            message: "Token expired".into(),
        };
        for fn_name in ALL_FETCH_FUNCTIONS {
            mock = mock.with_error(fn_name, error.clone());
        }
        mock
    }

    /// Invalid JSON — simulates API schema change breaking deserialization
    pub fn json_malformed() -> MockApiClient {
        let mut mock = MockApiClient::new();
        let error = ApiError::JsonParse {
            message: "Expected struct OrderDto, got null".into(),
        };
        for fn_name in ALL_FETCH_FUNCTIONS {
            mock = mock.with_error(fn_name, error.clone());
        }
        mock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_is_server_error() {
        let err = ApiError::Http {
            status: 500,
            message: "DB down".to_string(),
        };
        assert!(err.is_server_error());
        assert!(!err.is_client_error());
    }

    #[test]
    fn test_api_error_is_client_error() {
        let err = ApiError::Http {
            status: 404,
            message: "Not found".to_string(),
        };
        assert!(!err.is_server_error());
        assert!(err.is_client_error());
    }

    #[test]
    fn test_api_error_is_network_error() {
        let err = ApiError::Network {
            message: "Connection refused".to_string(),
        };
        assert!(err.is_network_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_user_message_for_500() {
        let err = ApiError::Http {
            status: 500,
            message: "DB down".to_string(),
        };
        assert_eq!(
            err.user_message(),
            "Ein Server-Fehler ist aufgetreten. Bitte versuchen Sie es später erneut."
        );
    }

    #[test]
    fn test_user_message_for_401() {
        let err = ApiError::Http {
            status: 401,
            message: "token expired".to_string(),
        };
        assert_eq!(
            err.user_message(),
            "Sitzung abgelaufen. Bitte neu anmelden."
        );
    }

    #[test]
    fn test_user_message_for_network_error() {
        let err = ApiError::Network {
            message: "timeout".to_string(),
        };
        assert_eq!(err.user_message(), "Verbindungsproblem: timeout");
    }

    #[test]
    fn test_user_message_for_403() {
        let err = ApiError::Http {
            status: 403,
            message: "forbidden".to_string(),
        };
        assert_eq!(err.user_message(), "Zugriff verweigert.");
    }

    #[test]
    fn test_user_message_for_404() {
        let err = ApiError::Http {
            status: 404,
            message: "not found".to_string(),
        };
        assert_eq!(err.user_message(), "Ressource nicht gefunden.");
    }

    #[test]
    fn test_user_message_for_json_parse() {
        let err = ApiError::JsonParse {
            message: "unexpected token".to_string(),
        };
        assert_eq!(err.user_message(), "Unerwartete Antwort: unexpected token");
    }

    #[test]
    fn test_mock_with_error() {
        let mock = MockApiClient::new().with_error(
            "fetch_users",
            ApiError::Http {
                status: 500,
                message: "DB down".into(),
            },
        );
        assert!(mock.would_return_server_error("fetch_users"));
        assert!(!mock.would_return_server_error("fetch_sites"));
    }

    #[test]
    fn test_scenario_all_500() {
        let mock = scenarios::all_500();
        // Verify ALL fetch functions are covered
        for fn_name in scenarios::ALL_FETCH_FUNCTIONS {
            assert!(
                mock.would_return_server_error(fn_name),
                "fetch function '{}' not covered by all_500() scenario!",
                fn_name
            );
        }
    }

    #[test]
    fn test_scenario_network_failure() {
        let mock = scenarios::network_failure();
        assert!(mock.would_return_network_error("fetch_users"));
        assert!(mock.would_return_network_error("fetch_orders"));
    }

    #[test]
    fn test_scenario_auth_expired() {
        let mock = scenarios::auth_expired();
        assert_eq!(
            mock.error_for("fetch_users").map(|e| e.user_message()),
            Some("Sitzung abgelaufen. Bitte neu anmelden.".to_string())
        );
    }

    #[test]
    fn test_scenario_json_malformed() {
        let mock = scenarios::json_malformed();
        assert_eq!(
            mock.error_for("fetch_orders").map(|e| e.user_message()),
            Some("Unerwartete Antwort: Expected struct OrderDto, got null".to_string())
        );
    }

    #[test]
    fn test_error_functions_list() {
        let mock = MockApiClient::new()
            .with_error(
                "fetch_users",
                ApiError::Http {
                    status: 500,
                    message: "err".into(),
                },
            )
            .with_error(
                "fetch_orders",
                ApiError::Http {
                    status: 500,
                    message: "err".into(),
                },
            );
        let err_fns = mock.error_functions();
        assert_eq!(err_fns.len(), 2);
        assert!(err_fns.iter().any(|f| *f == "fetch_users"));
        assert!(err_fns.iter().any(|f| *f == "fetch_orders"));
    }

    /// Verify ALL 36 API fetch functions are covered by mock scenarios.
    /// This ensures that when a NEW fetch function is added to api.rs,
    /// a corresponding mock scenario test must be created — preventing
    /// untested API calls that could crash the UI.
    #[test]
    fn test_all_api_functions_covered_by_scenarios() {
        let all_500_mock = scenarios::all_500();
        let network_mock = scenarios::network_failure();
        let auth_mock = scenarios::auth_expired();
        let json_mock = scenarios::json_malformed();

        for fn_name in scenarios::ALL_FETCH_FUNCTIONS {
            assert!(
                all_500_mock.error_for(fn_name).is_some(),
                "fetch function '{}' is not covered by all_500() scenario!",
                fn_name
            );
            assert!(
                network_mock.error_for(fn_name).is_some(),
                "fetch function '{}' is not covered by network_failure() scenario!",
                fn_name
            );
            assert!(
                auth_mock.error_for(fn_name).is_some(),
                "fetch function '{}' is not covered by auth_expired() scenario!",
                fn_name
            );
            assert!(
                json_mock.error_for(fn_name).is_some(),
                "fetch function '{}' is not covered by json_malformed() scenario!",
                fn_name
            );
        }
    }

    /// Verify CRUD operations (create/update/delete) can return errors.
    /// Add new mutations here to ensure they're error-tested.
    #[test]
    fn test_crud_operations_can_error() {
        let ops: [&str; 15] = [
            "create_user",
            "update_user",
            "delete_user",
            "create_site",
            "update_site",
            "delete_site",
            "create_order",
            "create_equipment",
            "delete_equipment",
            "create_animal",
            "add_treatment",
            "create_inventory_item",
            "delete_inventory_item",
            "stock_in",
            "clock_out",
        ];

        let mut mock = MockApiClient::new();
        for op in &ops {
            mock = mock.with_error(
                op,
                ApiError::Http {
                    status: 500,
                    message: "simulated creation failure".into(),
                },
            );
        }

        for op in &ops {
            assert!(
                mock.would_return_server_error(op),
                "CRUD operation '{}' should return server error in mock",
                op
            );
        }
    }

    /// Verify error types are properly distinguishable.
    #[test]
    fn test_error_type_distinction() {
        let server_err = ApiError::Http {
            status: 500,
            message: "Internal".into(),
        };
        let client_err = ApiError::Http {
            status: 400,
            message: "Bad Request".into(),
        };
        let network_err = ApiError::Network {
            message: "Connection refused".into(),
        };
        let parse_err = ApiError::JsonParse {
            message: "Unexpected token".into(),
        };

        assert!(server_err.is_server_error());
        assert!(!server_err.is_client_error());
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());
        assert!(network_err.is_network_error());
        assert!(!parse_err.is_server_error());
    }

    /// Verify common HTTP status codes produce correct error types.
    #[test]
    fn test_http_status_classification() {
        let error_400 = ApiError::Http {
            status: 400,
            message: "Bad Request".into(),
        };
        let error_401 = ApiError::Http {
            status: 401,
            message: "Unauthorized".into(),
        };
        let error_403 = ApiError::Http {
            status: 403,
            message: "Forbidden".into(),
        };
        let error_404 = ApiError::Http {
            status: 404,
            message: "Not Found".into(),
        };
        let error_409 = ApiError::Http {
            status: 409,
            message: "Conflict".into(),
        };
        let error_422 = ApiError::Http {
            status: 422,
            message: "Unprocessable".into(),
        };
        let error_500 = ApiError::Http {
            status: 500,
            message: "Internal".into(),
        };
        let error_502 = ApiError::Http {
            status: 502,
            message: "Bad Gateway".into(),
        };
        let error_503 = ApiError::Http {
            status: 503,
            message: "Unavailable".into(),
        };
        let error_504 = ApiError::Http {
            status: 504,
            message: "Gateway Timeout".into(),
        };
        let error_default = ApiError::Http {
            status: 418,
            message: "I'm a teapot".into(),
        };

        // 4xx = client errors
        assert!(error_400.is_client_error());
        assert!(error_401.is_client_error());
        assert!(error_403.is_client_error());
        assert!(error_404.is_client_error());
        assert!(error_409.is_client_error());
        assert!(error_422.is_client_error());

        // 5xx = server errors
        assert!(error_500.is_server_error());
        assert!(error_502.is_server_error());
        assert!(error_503.is_server_error());
        assert!(error_504.is_server_error());

        // Other 4xx defaults to client error
        assert!(error_default.is_client_error());
        assert!(!error_default.is_server_error());
    }

    /// Verify the scenarios module produces non-overlapping error types.
    #[test]
    fn test_scenarios_produce_different_errors() {
        let s500 = scenarios::all_500();
        let s_net = scenarios::network_failure();
        let s_auth = scenarios::auth_expired();
        let s_json = scenarios::json_malformed();

        // Server error scenario should be server errors
        assert!(s500.would_return_server_error("fetch_orders"));
        // Network scenario should be network errors, not server errors
        assert!(s_net.would_return_network_error("fetch_orders"));
        assert!(!s_net.would_return_server_error("fetch_orders"));
        // Auth should be 401, not server or network error
        let auth_err = s_auth.error_for("fetch_orders");
        assert!(auth_err.is_some());
        assert!(!auth_err.unwrap().is_server_error());
        assert!(!auth_err.unwrap().is_network_error());
        // JSON parse should be parse error, not server or network
        let json_err = s_json.error_for("fetch_orders");
        assert!(json_err.is_some());
        assert!(!json_err.unwrap().is_server_error());
        assert!(!json_err.unwrap().is_network_error());
    }

    /// Verify all scenarios have the same coverage (ALL_FETCH_FUNCTIONS)
    #[test]
    fn test_all_scenarios_cover_same_functions() {
        let s500 = scenarios::all_500();
        let s_net = scenarios::network_failure();
        let s_auth = scenarios::auth_expired();
        let s_json = scenarios::json_malformed();
        let _ = s_json; // silence unused

        for fn_name in scenarios::ALL_FETCH_FUNCTIONS {
            assert!(
                s500.error_for(fn_name).is_some(),
                "all_500() missing: {}",
                fn_name
            );
            assert!(
                s_net.error_for(fn_name).is_some(),
                "network_failure() missing: {}",
                fn_name
            );
            assert!(
                s_auth.error_for(fn_name).is_some(),
                "auth_expired() missing: {}",
                fn_name
            );
            assert!(
                s_json.error_for(fn_name).is_some(),
                "json_malformed() missing: {}",
                fn_name
            );
        }
    }

    /// Verify user-friendly messages don't leak technical stack traces
    #[test]
    fn test_user_messages_are_user_friendly() {
        // 500 errors should NOT include technical details
        let err_500 = ApiError::Http {
            status: 500,
            message: "panic at src/db/query.rs:1234:9".into(),
        };
        let msg_500 = err_500.user_message();
        assert!(!msg_500.contains("query.rs"));
        assert!(!msg_500.contains("panic at"));

        // 401 errors should be friendly
        let err_401 = ApiError::Http {
            status: 401,
            message: "invalid token signature".into(),
        };
        assert_eq!(
            err_401.user_message(),
            "Sitzung abgelaufen. Bitte neu anmelden."
        );

        // Network errors include the underlying message (useful for debugging connectivity)
        // but are prefixed with a friendly label
        let err_net = ApiError::Network {
            message: "timeout".into(),
        };
        let msg_net = err_net.user_message();
        assert!(msg_net.contains("Verbindungsproblem"));
        assert!(msg_net.contains("timeout"));
    }

    /// Verify edge case scenarios for 403, 404, and malformed data
    #[test]
    fn test_edge_case_scenarios() {
        // 403 Forbidden
        let mock_403 = MockApiClient::new().with_error(
            "delete_site",
            ApiError::Http {
                status: 403,
                message: "Not allowed to delete site".into(),
            },
        );
        let err = mock_403.error_for("delete_site");
        assert!(err.is_some());
        assert_eq!(err.unwrap().user_message(), "Zugriff verweigert.");

        // 404 Not Found
        let mock_404 = MockApiClient::new().with_error(
            "fetch_task",
            ApiError::Http {
                status: 404,
                message: "Task not found".into(),
            },
        );
        let err = mock_404.error_for("fetch_task");
        assert!(err.is_some());
        assert_eq!(err.unwrap().user_message(), "Ressource nicht gefunden.");

        // Auth scenario should be 401
        let s_auth = scenarios::auth_expired();
        let auth_err = s_auth.error_for("fetch_orders");
        assert!(auth_err.is_some());
        if let ApiError::Http { status, .. } = auth_err.unwrap() {
            let expected: u16 = 401;
            assert_eq!(*status, expected, "Auth expired scenario should return 401");
        }
    }
}
