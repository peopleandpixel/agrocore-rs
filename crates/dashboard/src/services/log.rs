//! Log entries for API calls and SQL queries.
//!
//! These are populated from JSONL files written by the agrocore-api crate
//! and read by the dashboard on every refresh cycle.
//!
//! File formats (JSONL — one JSON object per line):
//!   logs/api_calls.jsonl     — API call entries
//!   logs/sql_queries.jsonl   — SQL query entries

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single API call log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLogEntry {
    /// Timestamp when the call was received.
    pub timestamp: DateTime<Utc>,
    /// Client IP address.
    pub client: String,
    /// HTTP method (GET, POST, etc.).
    pub method: String,
    /// Request path (e.g. "/api/v1/sites").
    pub path: String,
    /// Response status code (200, 404, 500, etc.).
    pub status: u16,
    /// Response time in milliseconds.
    pub latency_ms: u64,
    /// User agent header (optional, truncated to 200 chars).
    pub user_agent: Option<String>,
}

/// A single SQL query log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlLogEntry {
    /// Timestamp when the query was executed.
    pub timestamp: DateTime<Utc>,
    /// Database query (SQL statement).
    pub query: String,
    /// Query parameters (serialized as a string representation).
    pub params: Vec<String>,
    /// Query duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the query succeeded.
    pub success: bool,
    /// Error message if the query failed.
    pub error: Option<String>,
    /// Database connection label (e.g. "primary", "replica").
    pub db_label: String,
}

/// Ring buffer for log entries — keeps only the last N entries.
#[derive(Debug)]
pub struct LogBuffer<T> {
    pub entries: Vec<T>,
    pub capacity: usize,
}

impl<T> LogBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, entry: T) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
