use std::collections::HashMap;

/// Per-request overrides that can be passed to any SDK method.
///
/// Most commonly used to supply a user-specific bearer token that differs from
/// the API key the client was initialised with.
#[derive(Debug, Default, Clone)]
pub struct RequestOptions {
    /// Override the default bearer token for this single request.
    pub token: Option<String>,
    /// Additional HTTP headers to include in this single request.
    pub headers: Option<HashMap<String, String>>,
    /// Sent as the Idempotency-Key header; a retry with the same key and request replays the first
    /// response instead of repeating it.
    pub idempotency_key: Option<String>,
}
