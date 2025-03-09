//! Wrapper around anyhow::Error for axum.
//! Taken from https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

pub struct HTTPError(pub StatusCode, pub String);

impl std::error::Error for HTTPError {}

impl std::fmt::Debug for HTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP Error {}: {}", self.0, self.1)
    }
}

impl std::fmt::Display for HTTPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP Error {}: {}", self.0, self.1)
    }
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self.0.downcast_ref::<HTTPError>() {
            Some(HTTPError(status, message)) => (status.clone(), message.clone()).into_response(),
            None => {
                error!("Returning Internal Server Error: {:?}", self.0);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal Server Error: {:?}", self.0),
                )
                    .into_response()
            }
        }
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
