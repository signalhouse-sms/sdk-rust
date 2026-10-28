use std::sync::Arc;

use reqwest::Method;
use serde_json::json;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Optional filters for [`CallLogsDomain::list`].
///
/// All fields are optional. None-valued fields are omitted from the query
/// string. Field names are snake_case in Rust but serialize to the camelCase
/// query keys the Voice service `/voice/api/v1/call-logs` API expects.
#[derive(Debug, Default)]
pub struct ListCallLogsParams<'a> {
    /// 1-indexed page (default 1).
    pub page: Option<u32>,
    /// Page size (default 20, max 1000).
    pub limit: Option<u32>,
    /// Filter by direction. One of `INBOUND`, `OUTBOUND`, or `INTERNAL`.
    pub direction: Option<&'a str>,
    /// Filter by status. One of `QUEUED`, `RINGING`, `IN_PROGRESS`,
    /// `COMPLETED`, `FAILED`, `BUSY`, `NO_ANSWER`, `CANCELED`.
    pub status: Option<&'a str>,
    /// Filter by how the call was handled. One of `SIP_TRUNK`,
    /// `PROGRAMMABLE_VOICE`, `AI_VOICE`, `WEBRTC`, `VOICEMAIL`, `CONFERENCE`,
    /// or `PSTN`.
    pub call_source: Option<&'a str>,
    /// Filter by owning subgroup.
    pub subgroup_id: Option<&'a str>,
    /// Filter by SIP trunk id.
    pub sip_trunk_id: Option<&'a str>,
    /// Filter by caller number (contains).
    pub from: Option<&'a str>,
    /// Filter by called number (contains).
    pub to: Option<&'a str>,
    /// Free-text search over callId / from / to.
    pub q: Option<&'a str>,
    /// Filter by sentiment. One of `positive`, `negative`, or `neutral`.
    pub sentiment: Option<&'a str>,
    /// When true, return only voicemails.
    pub voicemail: Option<bool>,
    /// ISO-8601 lower bound on start time.
    pub date_from: Option<&'a str>,
    /// ISO-8601 upper bound on start time.
    pub date_to: Option<&'a str>,
    /// Sort field (default `startTime`).
    pub sort: Option<&'a str>,
    /// Sort order. One of `asc` or `desc` (default `desc`).
    pub order: Option<&'a str>,
}

/// Call Logs — the account-scoped voice call history (Voice service
/// `/voice/api/v1/call-logs`). Distinct from `voice.calls`, which is the
/// Twilio-compatible programmatic call-control surface; this is the richer log
/// read surface with cost, recording, and voice-config attribution used by the
/// portal Call Logs view. Accessed via `client.voice.call_logs`.
pub struct CallLogsDomain {
    http: Arc<HttpClient>,
}

impl CallLogsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// List call logs for the current account, paginated and filterable.
    /// Returns `{ success, data: [...call logs], meta: { total, page, limit, totalPages } }`.
    ///
    /// **GET** `/voice/api/v1/call-logs`
    pub async fn list(
        &self,
        params: ListCallLogsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("page", params.page.map(|p| p.to_string())),
            ("limit", params.limit.map(|l| l.to_string())),
            ("direction", params.direction.map(String::from)),
            ("status", params.status.map(String::from)),
            ("callSource", params.call_source.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("sipTrunkId", params.sip_trunk_id.map(String::from)),
            ("from", params.from.map(String::from)),
            ("to", params.to.map(String::from)),
            ("q", params.q.map(String::from)),
            ("sentiment", params.sentiment.map(String::from)),
            ("voicemail", params.voicemail.map(|v| v.to_string())),
            ("dateFrom", params.date_from.map(String::from)),
            ("dateTo", params.date_to.map(String::from)),
            ("sort", params.sort.map(String::from)),
            ("order", params.order.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/voice/api/v1/call-logs{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a single call log by its UUID or callId. Returns
    /// `{ success, data: { ...call log } }`.
    ///
    /// **GET** `/voice/api/v1/call-logs/:id`
    pub async fn get(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/call-logs/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a short-TTL presigned playback URL for a call's recording.
    /// Account-scoped — only the owning account can presign. 404 if the call
    /// has no recording. Returns `{ success, data: { callId, url, expiresIn } }`.
    ///
    /// **GET** `/voice/api/v1/call-logs/:id/recording`
    pub async fn get_recording(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/call-logs/{}/recording", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Mark a voicemail (a call log with a recording) read or unread.
    /// Account-scoped. Pass `read = true` to mark read, `false` to mark unread.
    /// Returns `{ success, data: { id, voicemailReadAt } }`.
    ///
    /// **PATCH** `/voice/api/v1/call-logs/:id/voicemail-read`
    pub async fn mark_voicemail_read(
        &self,
        id: &str,
        read: bool,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = json!({ "read": read });
        self.http
            .request_json(
                &format!("/voice/api/v1/call-logs/{}/voicemail-read", safe),
                Method::PATCH,
                Some(&body),
                options,
            )
            .await
    }
}
