use std::{error::Error, fmt};

use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, Clone)]
pub struct HandlerError {
    pub status: StatusCode,
    pub message: String,
}

impl HandlerError {
    pub fn new(status: StatusCode, message: String) -> Self {
        Self { status, message }
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.status, self.message)
    }
}

impl Error for HandlerError {}

impl IntoResponse for HandlerError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.message).into_response()
    }
}
