//! HTTP-Fehlerbrücke für den API-Crate.
//!
//! `SharedError` lebt im framework-freien `agrocore-shared`-Crate und darf
//! `actix-web` nicht kennen. Damit Handler trotzdem `?` nutzen und korrekte
//! HTTP-Statuscodes liefern können, wird `SharedError` hier in `ApiError`
//! verpackt, das `actix_web::ResponseError` implementiert.

use crate::dto::ErrorResponse;
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use agrocore_shared::SharedError;

use agrocore_logging::error;
#[cfg(feature = "sqlx")]
use sqlx;

/// Dünner Wrapper um [`SharedError`], der sich als HTTP-Antwort rendern lässt.
#[derive(Debug)]
pub struct ApiError(pub SharedError);

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl From<SharedError> for ApiError {
    fn from(err: SharedError) -> Self {
        ApiError(err)
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError(SharedError::from(err))
    }
}

impl ApiError {
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self(SharedError::Forbidden(message.into()))
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self(SharedError::NotFound(message.into()))
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self(SharedError::Validation(message.into()))
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self(SharedError::Conflict(message.into()))
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self(SharedError::Validation(message.into()))
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self(SharedError::Internal(message.into()))
    }

    /// Kurzer, stabiler Fehler-Slug für das `error`-Feld der JSON-Antwort.
    fn slug(&self) -> &'static str {
        match self.0 {
            SharedError::NotFound(_) => "not_found",
            SharedError::Validation(_) => "validation",
            SharedError::Unauthorized(_) => "unauthorized",
            SharedError::Forbidden(_) => "forbidden",
            SharedError::Conflict(_) => "conflict",
            SharedError::AlreadyExists(_) => "already_exists",
            SharedError::ReferenceError(_) => "reference_error",
            SharedError::Database(_)
            | SharedError::Internal(_)
            | SharedError::NotImplemented(_) => "internal",
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self.0 {
            SharedError::NotFound(_) => StatusCode::NOT_FOUND,
            SharedError::Validation(_) => StatusCode::BAD_REQUEST,
            SharedError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            SharedError::Forbidden(_) => StatusCode::FORBIDDEN,
            SharedError::Conflict(_) => StatusCode::CONFLICT,
            SharedError::AlreadyExists(_) => StatusCode::CONFLICT,
            SharedError::ReferenceError(_) => StatusCode::BAD_REQUEST,
            SharedError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            SharedError::Database(_) | SharedError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let server_error = status.is_server_error();

        // Server errors are logged in full. Client errors (4xx) are expected, so
        // they are not logged here.
        if server_error {
            error!("Request failed: {}", self.0);
        }

        // A 5xx body must not carry the underlying message. `sqlx::Error`'s
        // `Display` names the table, the column, the constraint and — for a failed
        // decode — the Rust type it expected, which is a free map of the schema for
        // anyone who can reach the port. It also changes whenever the schema does,
        // so a client cannot parse it reliably anyway.
        //
        // 4xx messages are authored: they describe what the caller did wrong and are
        // safe and useful to return.
        let message = if server_error {
            match &self.0 {
                SharedError::NotImplemented(_) => {
                    "This endpoint is not implemented in this build.".to_string()
                }
                _ => "An internal error occurred. The details are in the server log.".to_string(),
            }
        } else {
            self.0.to_string()
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: self.slug().into(),
            message,
        })
    }
}
