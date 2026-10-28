use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// SIP Trunks — peer-to-peer SIP connections to a customer PBX or carrier.
/// Routes are mounted under `/voice/sip-trunks` on the Voice service.
/// Accessed via `client.voice.sip_trunks`.
pub struct SipTrunksDomain {
    http: Arc<HttpClient>,
}

impl SipTrunksDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// List all SIP trunks for the current account.
    ///
    /// **GET** `/voice/sip-trunks`
    pub async fn list(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/voice/sip-trunks", Method::GET, None, options)
            .await
    }

    /// Get a single SIP trunk by ID.
    ///
    /// **GET** `/voice/sip-trunks/:id`
    pub async fn get(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/sip-trunks/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a new SIP trunk.
    ///
    /// `trunk_data` keys: `name`, `region`, `connectionType` (`"IP_AUTH"` or
    /// `"REGISTRATION"`); optional `allowedIps`, `transport`,
    /// `maxSpendPerMinute`, `destinationHosts`, `sourceHosts`. Registration
    /// trunks return a one-time-visible password on the response.
    ///
    /// **POST** `/voice/sip-trunks`
    pub async fn create(
        &self,
        trunk_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/voice/sip-trunks", Method::POST, Some(trunk_data), options)
            .await
    }

    /// Update an existing SIP trunk.
    ///
    /// **PATCH** `/voice/sip-trunks/:id`
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
                &format!("/voice/sip-trunks/{}", safe),
                Method::PATCH,
                Some(update_data),
                options,
            )
            .await
    }

    /// Delete a SIP trunk.
    ///
    /// **DELETE** `/voice/sip-trunks/:id`
    pub async fn delete(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/sip-trunks/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Toggle a trunk's active/inactive status.
    ///
    /// **POST** `/voice/sip-trunks/:id/toggle-active`
    pub async fn toggle_active(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/sip-trunks/{}/toggle-active", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Regenerate the SIP password for a REGISTRATION-type trunk. The old
    /// password stops working immediately. Returns `{ "password": "..." }`.
    ///
    /// **POST** `/voice/sip-trunks/:id/regenerate-password`
    pub async fn regenerate_password(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/sip-trunks/{}/regenerate-password", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Assign phone numbers to a SIP trunk. Assigned numbers route their
    /// inbound calls to this trunk.
    ///
    /// **POST** `/voice/sip-trunks/:id/assign-numbers`
    pub async fn assign_numbers(
        &self,
        id: &str,
        phone_numbers: &[String],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = json!({ "phoneNumbers": phone_numbers });
        self.http
            .request_json(
                &format!("/voice/sip-trunks/{}/assign-numbers", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Unassign phone numbers from a SIP trunk.
    ///
    /// **POST** `/voice/sip-trunks/:id/unassign-numbers`
    pub async fn unassign_numbers(
        &self,
        id: &str,
        phone_numbers: &[String],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = json!({ "phoneNumbers": phone_numbers });
        self.http
            .request_json(
                &format!("/voice/sip-trunks/{}/unassign-numbers", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// List available SIP Points of Presence.
    ///
    /// **GET** `/voice/sip-trunks/pops`
    pub async fn get_pops(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/voice/sip-trunks/pops", Method::GET, None, options)
            .await
    }
}
