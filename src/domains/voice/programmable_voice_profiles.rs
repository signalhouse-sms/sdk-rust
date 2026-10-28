use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Programmable Voice Profiles — group a set of the account's numbers (across
/// one or more subgroups) under a single inbound call-handling decision routed
/// through Signal House's own voice servers — the same idea as a SIP trunk, but
/// without a customer SBC. A number can belong to at most one profile and is
/// mutually exclusive with SIP trunks.
///
/// Routes are mounted under `/voice/api/v1/programmable-voice-profiles` on the
/// Voice service. Accessed via `client.voice.programmable_voice_profiles`.
pub struct ProgrammableVoiceProfilesDomain {
    http: Arc<HttpClient>,
}

impl ProgrammableVoiceProfilesDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// List all Programmable Voice Profiles for the current account.
    ///
    /// **GET** `/voice/api/v1/programmable-voice-profiles`
    pub async fn list(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/api/v1/programmable-voice-profiles",
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a single Programmable Voice Profile by ID.
    ///
    /// **GET** `/voice/api/v1/programmable-voice-profiles/:id`
    pub async fn get(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/programmable-voice-profiles/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a Programmable Voice Profile.
    ///
    /// `profile_data` keys: `name` (required); `subgroupIds` (required, string
    /// array with at least one entry); optional `region`, `routeAction`
    /// (`"FORWARD"`, `"WEBRTC"`, `"SIP_TRUNK"`, `"SIP_PROFILE"`, or
    /// `"CALL_CONTROL"`; defaults to `"FORWARD"`) — deliver to a PSTN number
    /// (FORWARD), ring the profile's subgroup registered softphones (WEBRTC),
    /// send to a SIP trunk (SIP_TRUNK), ring a registered SIP endpoint
    /// (SIP_PROFILE), or hand the call to your programmable call-control webhook
    /// that returns SHML (CALL_CONTROL); `forwardToE164` (required when
    /// `routeAction` is `"FORWARD"`), `routeSipTrunkId` (required when
    /// `routeAction` is `"SIP_TRUNK"`), `routeSipProfileId` (required when
    /// `routeAction` is `"SIP_PROFILE"`), `webhookUrl` (inbound voice webhook /
    /// SHML program URL; required when `routeAction` is `"CALL_CONTROL"`),
    /// `forwardAfterSeconds`, `recordingEnabled`, `enabled` (defaults to `true`).
    ///
    /// **POST** `/voice/api/v1/programmable-voice-profiles`
    pub async fn create(
        &self,
        profile_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/api/v1/programmable-voice-profiles",
                Method::POST,
                Some(profile_data),
                options,
            )
            .await
    }

    /// Update an existing Programmable Voice Profile (partial).
    ///
    /// `update_data` accepts the same keys as `create` — `name`, `subgroupIds`,
    /// `region`, `routeAction` (`"FORWARD"` | `"WEBRTC"` | `"SIP_TRUNK"` |
    /// `"SIP_PROFILE"` | `"CALL_CONTROL"`), `forwardToE164`, `routeSipTrunkId`
    /// (required when `routeAction` is `"SIP_TRUNK"`), `routeSipProfileId`
    /// (required when `routeAction` is `"SIP_PROFILE"`), `webhookUrl` (required
    /// when `routeAction` is `"CALL_CONTROL"`), `forwardAfterSeconds`,
    /// `recordingEnabled`, `enabled` — all optional.
    ///
    /// **PATCH** `/voice/api/v1/programmable-voice-profiles/:id`
    pub async fn update(
        &self,
        id: &str,
        update_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/programmable-voice-profiles/{}", safe),
                Method::PATCH,
                Some(update_data),
                options,
            )
            .await
    }

    /// Toggle a profile active/inactive.
    ///
    /// **POST** `/voice/api/v1/programmable-voice-profiles/:id/toggle-active`
    pub async fn toggle_active(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!(
                    "/voice/api/v1/programmable-voice-profiles/{}/toggle-active",
                    safe
                ),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Delete a Programmable Voice Profile. Its number assignments cascade away,
    /// so those numbers fall back to subgroup/global routing. Returns
    /// `{ "message": "..." }`.
    ///
    /// **DELETE** `/voice/api/v1/programmable-voice-profiles/:id`
    pub async fn delete(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/programmable-voice-profiles/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Assign a phone number (by E.164) to this profile. Rejected if the number
    /// is already configured on a SIP trunk/endpoint or another profile — it
    /// must be de-configured there first (mutual exclusivity). A leading `"+"`
    /// on `e164` is optional.
    ///
    /// **POST** `/voice/api/v1/programmable-voice-profiles/:id/assign-number`
    pub async fn assign_number(
        &self,
        id: &str,
        e164: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id)), ("e164", Some(e164))])?;
        let safe = urlencoding::encode(id);
        let body = json!({ "e164": e164 });
        self.http
            .request_json(
                &format!(
                    "/voice/api/v1/programmable-voice-profiles/{}/assign-number",
                    safe
                ),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Unassign a phone number (by E.164) from this profile. A leading `"+"` on
    /// `e164` is optional.
    ///
    /// **POST** `/voice/api/v1/programmable-voice-profiles/:id/unassign-number`
    pub async fn unassign_number(
        &self,
        id: &str,
        e164: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id)), ("e164", Some(e164))])?;
        let safe = urlencoding::encode(id);
        let body = json!({ "e164": e164 });
        self.http
            .request_json(
                &format!(
                    "/voice/api/v1/programmable-voice-profiles/{}/unassign-number",
                    safe
                ),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }
}
