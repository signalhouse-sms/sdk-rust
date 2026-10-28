use std::sync::Arc;

use reqwest::Method;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Optional filters for [`AnalyticsDomain::get`].
///
/// All fields are optional. None-valued fields are omitted from the query
/// string. Field names are snake_case in Rust but serialize to the camelCase
/// query keys the Voice service `/voice/stats/voice-analytics` API expects.
///
/// The per-direction summary tiles honor every filter **except** `direction`
/// (they always cover both directions); the byDate / byNumber / byCarrier
/// breakdowns honor all filters.
#[derive(Debug, Default)]
pub struct VoiceAnalyticsParams<'a> {
    /// ISO-8601 lower bound on call start time.
    pub date_from: Option<&'a str>,
    /// ISO-8601 upper bound on call start time.
    pub date_to: Option<&'a str>,
    /// Bucket size for the byDate breakdown. One of `day`, `week`, or `month`
    /// (default `day`).
    pub group_by: Option<&'a str>,
    /// Restrict the breakdowns to one direction. One of `INBOUND` or
    /// `OUTBOUND`. Summary tiles always cover both directions.
    pub direction: Option<&'a str>,
    /// Comma-separated `CallLog.callSource` list (`SIP_TRUNK`,
    /// `PROGRAMMABLE_VOICE`, `AI_VOICE`, `WEBRTC`, `VOICEMAIL`, `CONFERENCE`,
    /// `PSTN`).
    pub call_source: Option<&'a str>,
    /// Filter by owning subgroup.
    pub subgroup_id: Option<&'a str>,
    /// Substring match against either leg (from OR to).
    pub number: Option<&'a str>,
    /// Filter by carrier.
    pub carrier: Option<&'a str>,
}

/// Voice Analytics — aggregated call metrics for the portal Voice Analytics
/// view (Voice service `/voice/stats/voice-analytics`). Returns per-direction
/// summary tiles (both directions) plus byDate / byNumber / byCarrier status
/// breakdowns for charts. Distinct from `voice.call_logs`, which is the raw
/// call history. Accessed via `client.voice.analytics`.
pub struct AnalyticsDomain {
    http: Arc<HttpClient>,
}

impl AnalyticsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get aggregated voice analytics for the current account.
    ///
    /// Returns `{ success, range: { dateFrom, dateTo, days }, outbound: Summary,
    /// inbound: Summary, byDate: [Bucket], byNumber: [Bucket], byCarrier: [Bucket], byChannel: [Bucket] }`,
    /// where:
    ///
    /// * **Summary** = `{ totalCalls, successfulCalls, successRate,
    ///   avgCallsPerDay, avgCallDurationSeconds, totalCallMinutes, answerRate,
    ///   abandonmentRate, failedCalls, failureRate, pctUnder6s,
    ///   avgRingDurationSeconds, sentimentScore, sentimentLabel }` — rates are
    ///   percentages (0–100); `avgCallDurationSeconds`, `avgRingDurationSeconds`,
    ///   `sentimentScore`, and `sentimentLabel` may be null.
    /// * **Bucket** = `{ key, total, completed, failed, noAnswer, busy, canceled,
    ///   inProgress, totalDurationSeconds, avgSentimentScore }` — `key` is the
    ///   date / number / carrier the bucket aggregates; `avgSentimentScore` may
    ///   be null.
    ///
    /// **GET** `/voice/stats/voice-analytics`
    pub async fn get(
        &self,
        params: VoiceAnalyticsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("dateFrom", params.date_from.map(String::from)),
            ("dateTo", params.date_to.map(String::from)),
            ("groupBy", params.group_by.map(String::from)),
            ("direction", params.direction.map(String::from)),
            ("callSource", params.call_source.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("number", params.number.map(String::from)),
            ("carrier", params.carrier.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/voice/stats/voice-analytics{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }
}
