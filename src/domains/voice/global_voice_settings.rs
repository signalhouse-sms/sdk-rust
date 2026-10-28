use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Global Voice Settings — account-wide voice defaults (accepted regions, max
/// spend per minute, E911). SIP trunks and Programmable Voice Profiles override
/// these. Wraps the Voice service's `/voice/api/v1/global-voice-settings` surface.
/// Accessed via `client.voice.global_voice_settings`.
pub struct GlobalVoiceSettingsDomain {
    http: Arc<HttpClient>,
}

impl GlobalVoiceSettingsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get the current account's global voice settings. Returns `data: null` if
    /// the account has never configured them (defaults apply). Returns
    /// `{ success, data: { accountId, acceptedRegions, maxSpendPerMinute, e911Enabled, ... } | null }`.
    ///
    /// **GET** `/voice/api/v1/global-voice-settings`
    pub async fn get(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/api/v1/global-voice-settings",
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Upsert the account's global voice settings. Partial — only the fields you
    /// provide are written; omitted fields are preserved.
    ///
    /// `settings` keys (camelCase): `acceptedRegions` (string array of country
    /// calling codes; empty = all allowed), `maxSpendPerMinute` (number|null —
    /// baseline max spend per minute in dollars; null = no limit), `e911Enabled`
    /// (boolean — whether numbers may dial 911). Returns
    /// `{ success, data: { ...settings } }`.
    ///
    /// **PUT** `/voice/api/v1/global-voice-settings`
    pub async fn update(
        &self,
        settings: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/api/v1/global-voice-settings",
                Method::PUT,
                Some(settings),
                options,
            )
            .await
    }
}
