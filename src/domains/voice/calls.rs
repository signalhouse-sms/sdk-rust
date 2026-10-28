use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Optional filters for [`CallsDomain::list`].
///
/// All fields are optional. None-valued fields are omitted from the query
/// string. Field names use snake_case both in Rust and over the wire, matching
/// the Voice service `/voice/v1/calls` API.
#[derive(Debug, Default)]
pub struct ListCallsParams<'a> {
    /// Page number (1-indexed). Defaults server-side to 1.
    pub page: Option<u32>,
    /// Page size (1-100). Defaults server-side to 20.
    pub limit: Option<u32>,
    /// Filter by call status. One of `QUEUED`, `RINGING`, `IN_PROGRESS`,
    /// `COMPLETED`, `FAILED`, `BUSY`, `NO_ANSWER`, `CANCELED`.
    pub status: Option<&'a str>,
    /// Filter by call direction. One of `INBOUND` or `OUTBOUND`.
    pub direction: Option<&'a str>,
    /// Filter by caller number.
    pub from: Option<&'a str>,
    /// Filter by destination number.
    pub to: Option<&'a str>,
    /// ISO-8601 lower bound on `start_time`.
    pub date_from: Option<&'a str>,
    /// ISO-8601 upper bound on `start_time`.
    pub date_to: Option<&'a str>,
}

/// Voice calls — REST API for placing outbound calls and inspecting call logs.
/// Wraps the Voice service's `/voice/v1/calls` surface. Accessed via
/// `client.voice.calls`.
///
/// Origination modes:
/// * **Single-leg** (no trunk/profile/identity): Twilio-style direct outbound —
///   dial `to` from caller-ID `from`. Customer's `from` must be an
///   account-owned number.
/// * **SIP trunk** (`sip_trunk_id`): Route via a configured trunk; the trunk's
///   auth/ACL governs caller-ID rules.
/// * **Two-leg-with-bridge** (`sip_profile_id` OR `to_identity`): Ring the
///   SDK-registered endpoint first, then dial `to` and bridge. `to_identity`
///   resolves to the most recently-minted unexpired ephemeral identity for
///   that name.
pub struct CallsDomain {
    http: Arc<HttpClient>,
}

impl CallsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Create an outbound call.
    ///
    /// `call_data` keys: `to` (required, E.164 or SIP URI), `from` (required,
    /// caller ID — must be account-owned when not using a trunk); optional
    /// `sip_trunk_id`, `sip_profile_id`, `to_identity`, `answer_url`,
    /// `answer_method` (`"GET"` or `"POST"`, default POST), `status_callback`,
    /// `status_callback_method`, `recording_enabled`, `transcription_enabled`,
    /// `ivr_flow_id`, `metadata`.
    ///
    /// `sip_trunk_id`, `sip_profile_id`, and `to_identity` are mutually
    /// exclusive routing modes. Returns
    /// `{ call_id, status, direction, from, to, created_at }`.
    ///
    /// **POST** `/voice/v1/calls`
    pub async fn create(
        &self,
        call_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/voice/v1/calls", Method::POST, Some(call_data), options)
            .await
    }

    /// List call logs for the current account, paginated and filterable.
    /// Returns `{ calls, total, page, limit }`.
    ///
    /// **GET** `/voice/v1/calls`
    pub async fn list(
        &self,
        params: ListCallsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("page", params.page.map(|p| p.to_string())),
            ("limit", params.limit.map(|l| l.to_string())),
            ("status", params.status.map(String::from)),
            ("direction", params.direction.map(String::from)),
            ("from", params.from.map(String::from)),
            ("to", params.to.map(String::from)),
            ("date_from", params.date_from.map(String::from)),
            ("date_to", params.date_to.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/voice/v1/calls{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a single call log by ID. `id` is the call log UUID (the value
    /// returned as `call_id` from [`Self::create`]).
    ///
    /// **GET** `/voice/v1/calls/:id`
    pub async fn get(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/v1/calls/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Hang up an in-progress call. `reason` is one of `"NORMAL"`, `"BUSY"`,
    /// `"NO_ANSWER"`, `"REJECTED"` and is recorded on the call log; defaults
    /// to `NORMAL` when omitted. Returns `{ success, message }`.
    ///
    /// **POST** `/voice/v1/calls/:id/hangup`
    pub async fn hangup(
        &self,
        id: &str,
        reason: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = match reason {
            Some(r) => json!({ "reason": r }),
            None => json!({}),
        };
        self.http
            .request_json(
                &format!("/voice/v1/calls/{}/hangup", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }
}
