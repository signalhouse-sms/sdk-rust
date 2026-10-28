use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Number Routes — one inbound route set directly on a single phone number. A
/// number's own route takes priority over its Programmable Voice Profile and
/// subgroup voice settings.
///
/// Routes are mounted under `/voice/api/v1/numbers/:e164/route` on the
/// Voice service. Accessed via `client.voice.number_routes`.
pub struct NumberRoutesDomain {
    http: Arc<HttpClient>,
}

impl NumberRoutesDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a number's own route and the routing currently in effect for it. A
    /// leading `"+"` on `e164` is optional. Returns
    /// `{ "e164", "route": {...} | null, "effective": {...} }`.
    ///
    /// **GET** `/voice/api/v1/numbers/:e164/route`
    pub async fn get(
        &self,
        e164: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("e164", Some(e164))])?;
        let safe = urlencoding::encode(e164);
        self.http
            .request_json(
                &format!("/voice/api/v1/numbers/{}/route", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Set (create or replace) a number's route. The whole route is replaced:
    /// fields the new `routeAction` doesn't use are cleared. A leading `"+"` on
    /// `e164` is optional.
    ///
    /// `route_data` keys: `routeAction` (required: `"FORWARD"`, `"WEBRTC"`,
    /// `"SIP_TRUNK"`, `"SIP_PROFILE"`, `"CALL_CONTROL"`, `"AI"`, or
    /// `"VOICEMAIL"`); optional `forwardToE164` (required for FORWARD, or the
    /// WEBRTC no-answer destination when `noAnswerAction` is `"FORWARD"`),
    /// `noAnswerAction` (WEBRTC only: `"VOICEMAIL"` | `"FORWARD"` |
    /// `"KEEP_RINGING"`, default `"VOICEMAIL"`), `noAnswerSeconds` (WEBRTC only:
    /// integer 5-120, default 20), `routeSipTrunkId` (required for SIP_TRUNK),
    /// `routeSipProfileId` (required for SIP_PROFILE), `webhookUrl` (SHML program
    /// URL; required for CALL_CONTROL), `routeAgentProfileId` (AI agent profile
    /// id, `AGP…`; required for AI), `recordingEnabled`. Unset fields must be
    /// omitted or `null`. Returns `{ "e164", "route": {...}, "effective": {...} }`.
    ///
    /// **PUT** `/voice/api/v1/numbers/:e164/route`
    pub async fn set(
        &self,
        e164: &str,
        route_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("e164", Some(e164))])?;
        let safe = urlencoding::encode(e164);
        self.http
            .request_json(
                &format!("/voice/api/v1/numbers/{}/route", safe),
                Method::PUT,
                Some(route_data),
                options,
            )
            .await
    }

    /// Remove a number's own route. The number falls back to its Programmable
    /// Voice Profile, subgroup voice settings, or the default. A leading `"+"`
    /// on `e164` is optional. Returns
    /// `{ "e164", "route": null, "effective": {...} }`.
    ///
    /// **DELETE** `/voice/api/v1/numbers/:e164/route`
    pub async fn delete(
        &self,
        e164: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("e164", Some(e164))])?;
        let safe = urlencoding::encode(e164);
        self.http
            .request_json(
                &format!("/voice/api/v1/numbers/{}/route", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }
}
