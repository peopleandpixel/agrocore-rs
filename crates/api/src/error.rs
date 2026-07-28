//! HTTP-Fehlerbrücke für den API-Crate.
//!
//! `SharedError` lebt im framework-freien `agrocore-shared`-Crate und darf
//! `actix-web` nicht kennen. Damit Handler trotzdem `?` nutzen und korrekte
//! HTTP-Statuscodes liefern können, wird `SharedError` hier in `ApiError`
//! verpackt, das `actix_web::ResponseError` implementiert.

use crate::dto::ErrorResponse;
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use agrocore_shared::SharedError;

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

impl ApiError {
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
            SharedError::Database(_) | SharedError::Internal(_) | SharedError::NotImplemented(_) => "internal",
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
            SharedError::Database(_) | SharedError::Internal(_) | SharedError::NotImplemented(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        // Server-Fehler werden protokolliert; Client-Fehler (4xx) sind erwartbar.
        if status.is_server_error() {
            tracing::error!("Request failed: {}", self.0);
        }
        HttpResponse::build(status).json(ErrorResponse {
            error: self.slug().into(),
            message: self.0.to_string(),
        })
    }
}
