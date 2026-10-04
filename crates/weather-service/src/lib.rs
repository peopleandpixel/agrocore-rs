//! Weather service: provider implementations and the background fetch worker.
//!
//! This was a binary crate with no `lib.rs`, which meant the three providers in
//! `providers/` could not be called from anywhere else. `GET /api/v1/calculate/
//! weather/fetch` parses the requested provider and then ignores it, returning a
//! fixed 20.5 C / 65 % / 1013.25 hPa — so the provider code had no caller at all.
//!
//! The modules are exported here so the API can select and invoke a provider; the
//! `[[bin]]` target below keeps the standalone service runnable as before.

pub mod providers;
pub mod worker;
