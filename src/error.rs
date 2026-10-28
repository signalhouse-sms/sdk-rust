use thiserror::Error;

/// Errors that can occur when using the SignalHouse SDK.
#[derive(Error, Debug)]
pub enum SignalHouseError {
    /// A required parameter was missing or empty.
    #[error("Missing required parameter: {0}")]
    Validation(String),

    /// The API returned a non-2xx status code.
    #[error("API error (status {status}): {message}")]
    Api {
        /// HTTP status code returned by the API.
        status: u16,
        /// Human-readable error message from the API response.
        message: String,
    },

    /// An HTTP transport error occurred (network failure, timeout, etc.).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Failed to serialize a request body to JSON.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// The requested operation is not currently supported.
    #[error("{0}")]
    NotSupported(String),
}
