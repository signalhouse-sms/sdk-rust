use serde_json::Value;

/// Standardised response wrapper returned by JSON SDK methods.
///
/// Mirrors the shape used by the JavaScript SDK so that consumers can rely on
/// a consistent envelope regardless of language.
#[derive(Debug)]
pub struct SignalHouseResponse {
    /// `true` when the API returned a 2xx status code.
    pub success: bool,
    /// Parsed JSON body on success.
    pub data: Option<Value>,
    /// Error message on failure.
    pub error: Option<String>,
    /// HTTP status code returned by the server.
    pub status: u16,
}

/// File download response. Successful bodies are never decoded as text or JSON.
#[derive(Debug)]
pub struct SignalHouseFileResponse {
    /// Whether the API returned a 2xx status.
    pub success: bool,
    /// Exact file bytes on success, including an empty file.
    pub data: Option<Vec<u8>>,
    /// The normal API error message on failure.
    pub error: Option<String>,
    /// HTTP status code returned by the server.
    pub status: u16,
}
