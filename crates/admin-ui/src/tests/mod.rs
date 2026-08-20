//! Test modules for Admin UI
//!
//! - `api_error_handling` — Tests verifying that all API fetch functions
//!   return Result<T, String> and handle errors gracefully.
//! - `mock_test_framework` — Mock API client and ApiError type for testing
//!   UI components against simulated API failures (500s, 401s, network errors).

#![cfg(test)]

mod api_error_handling;
mod mock_test_framework;
