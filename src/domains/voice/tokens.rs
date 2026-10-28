use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// WebRTC voice tokens — mint ephemeral SIP identities the browser/mobile
/// voice SDK uses to register against the SignalHouse voice fabric. Wraps
/// the Voice service's `/voice/tokens` endpoint. Accessed via
/// `client.voice.tokens`.
///
/// Customer flow:
/// 1. Backend calls [`TokensDomain::create`] and returns the resulting `token`
///    to the customer's browser/app.
/// 2. Browser SDK registers using the embedded `sip_credentials`.
/// 3. Outbound calls from the SDK go through OpenSIPS -> Asterisk -> carrier;
///    the server-side `calls.create({ to_identity: "alice", ... })` flow can
///    also ring this identity for click-to-call patterns.
pub struct TokensDomain {
    http: Arc<HttpClient>,
}

impl TokensDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Mint an ephemeral voice token + SIP credentials.
    ///
    /// `token_data` keys (all optional): `identity` (customer-facing logical
    /// name; persisted as `requested_identity` so server-initiated
    /// `calls.create({ to_identity: ... })` can resolve it later; defaults to
    /// `"anonymous"`), `ttl` (seconds, clamped to the server's max; defaults
    /// to `VOICE_TOKEN_DEFAULT_TTL_SEC`), `grants` (capability grants —
    /// `{ "voice": { "incoming": { "allow": bool }, "outgoing":
    /// { "allowedNumbers": [...] } } }`).
    ///
    /// Returns `{ token, identity, expires_at, sip_credentials: { username,
    /// password, domain, wss_url } }`. The plaintext SIP password is sent
    /// ONCE inside `sip_credentials` and is never recoverable.
    ///
    /// Pass `None` for `token_data` to mint a token with all server defaults.
    ///
    /// **POST** `/voice/tokens`
    pub async fn create(
        &self,
        token_data: Option<&Value>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let empty = json!({});
        let body = token_data.unwrap_or(&empty);
        self.http
            .request_json("/voice/tokens", Method::POST, Some(body), options)
            .await
    }
}
