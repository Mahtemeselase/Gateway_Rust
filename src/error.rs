//#![allow(unused)]
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

//Unified error type for the entire payment gateway.
#[derive(Error, Debug)]
pub enum GatewayError {
    // -- 1. Business Logic Errors (Merchant's Fault - 4xx)
    #[error("Invalid API credentials")]
    Unauthorized,

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Invalid payload: {0}")]
    InvalidPayload(String),

    // --2. Infranstructure Errors (System Fault - 5xx)
    // use #[from] to auto convert when '?' operator is used
    #[error("Database ledger error")]
    Database(#[from] sqlx::Error),

    #[error("Network error communicating with upstream bank")]
    Upstream(#[from] reqwest::Error),

    #[error("Cryptographic operation failed")]
    Crypto(#[from] rsa::errors::Error),
}

// public HTTP responses
impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        tracing::error!("Gateway Fault: {:?}", self);

        let (status, public_message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::TransactionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::InvalidPayload(_) => (StatusCode::BAD_REQUEST, self.to_string()),

            Self::Database(_) | Self::Upstream(_) | Self::Crypto(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal gateway failure".to_string(),
            ),
        };

        let payload = json!({
            "status": "error",
            "message": public_message
        });

        (status, Json(payload)).into_response()
    }
}
