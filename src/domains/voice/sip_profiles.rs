use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// SIP Profiles / endpoints — single registerable SIP UAs (desk phones,
/// softphones, SIP apps) that register to Signal House with username and
/// password. Distinct from a SIP trunk (peer-to-peer link to a PBX/carrier).
///
/// Routes are mounted under `/voice/sip-profiles` on the Voice service.
/// Accessed via `client.voice.sip_profiles`.
pub struct SipProfilesDomain {
    http: Arc<HttpClient>,
}

impl SipProfilesDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// List all SIP profiles for the current account.
    ///
    /// **GET** `/voice/sip-profiles`
    pub async fn list(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/voice/sip-profiles", Method::GET, None, options)
            .await
    }

    /// Get a single SIP profile by ID. Response does NOT include the password —
    /// use `get_password()` to retrieve it.
    ///
    /// **GET** `/voice/sip-profiles/:id`
    pub async fn get(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/sip-profiles/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Fetch the SIP password for this profile. Returns `{ "password": "..." }`.
    ///
    /// **GET** `/voice/sip-profiles/:id/password`
    pub async fn get_password(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/sip-profiles/{}/password", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// List valid SIP transports (UDP/TCP/TLS) with their address and port.
    ///
    /// **GET** `/voice/sip-profiles/transports`
    pub async fn get_transports(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/voice/sip-profiles/transports", Method::GET, None, options)
            .await
    }

    /// Create a SIP profile. The server generates the SIP username and
    /// password; the password is returned at the TOP LEVEL of the response
    /// (not nested in `sipProfile`) and is the only chance to surface it
    /// without an explicit `get_password()` call.
    ///
    /// `profile_data` keys: `name` (required); optional `recordingAllowed`,
    /// `transcriptionEnabled`, `sentimentFlag`.
    ///
    /// **POST** `/voice/sip-profiles`
    pub async fn create(
        &self,
        profile_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/sip-profiles",
                Method::POST,
                Some(profile_data),
                options,
            )
            .await
    }

    /// Update a SIP profile. To rotate the password, pass a new `password`
    /// value — there is no dedicated regenerate endpoint for profiles.
    ///
    /// **PATCH** `/voice/sip-profiles/:id`
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
                &format!("/voice/sip-profiles/{}", safe),
                Method::PATCH,
                Some(update_data),
                options,
            )
            .await
    }

    /// Delete a SIP profile. Unassigns any linked numbers as a side effect.
    ///
    /// **DELETE** `/voice/sip-profiles/:id`
    pub async fn delete(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/sip-profiles/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Assign a phone number to this SIP profile (routes inbound calls on that
    /// number to the endpoint). `phone_number_id` may be the number's E.164
    /// (preferred; the backend detects it) or a legacy Voice service
    /// phone-number UUID.
    ///
    /// **POST** `/voice/sip-profiles/:id/assign-number`
    pub async fn assign_number(
        &self,
        id: &str,
        phone_number_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id)), ("phoneNumberId", Some(phone_number_id))])?;
        let safe = urlencoding::encode(id);
        let body = json!({ "phoneNumberId": phone_number_id });
        self.http
            .request_json(
                &format!("/voice/sip-profiles/{}/assign-number", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Unassign a phone number from this SIP profile. `phone_number_id` may be
    /// the number's E.164 (preferred; the backend detects it) or a legacy
    /// Voice service phone-number UUID.
    ///
    /// **POST** `/voice/sip-profiles/:id/unassign-number`
    pub async fn unassign_number(
        &self,
        id: &str,
        phone_number_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id)), ("phoneNumberId", Some(phone_number_id))])?;
        let safe = urlencoding::encode(id);
        let body = json!({ "phoneNumberId": phone_number_id });
        self.http
            .request_json(
                &format!("/voice/sip-profiles/{}/unassign-number", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }
}
