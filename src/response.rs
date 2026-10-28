use serde_json::Value;

/// Standardised response wrapper returned by every SDK method.
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
