use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Optional parameters for [`MessagesDomain::get_messages`].
#[derive(Debug, Default)]
pub struct GetMessagesParams<'a> {
    pub id: Option<&'a str>,
    pub campaign_id: Option<&'a str>,
    /// Filter to messages attributed to these registrations. Cannot be combined with
    /// `campaign_id`; each id must belong to the group. Single value, or a comma-separated list.
    pub registration_id: Option<&'a str>,
    pub brand_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    pub group_id: Option<&'a str>,
    pub phone_number: Option<&'a str>,
    /// A plain digits-only value is an exact match. A value containing `%` is a wildcard for a
    /// partial match (e.g. `"%1234"` matches numbers ending in 1234); 3-15 digits are required
    /// alongside any `%`.
    pub sender_phone_number: Option<&'a str>,
    /// A plain digits-only value is an exact match. A value containing `%` is a wildcard for a
    /// partial match (e.g. `"%1234"` matches numbers ending in 1234); 3-15 digits are required
    /// alongside any `%`.
    pub recipient_phone_number: Option<&'a str>,
    pub status: Option<&'a str>,
    pub direction: Option<&'a str>,
    /// Accepts a single value (e.g. "SMS") or a comma-separated list (e.g. "SMS,MMS").
    /// Allowed values: SMS, MMS, RCS, WHATSAPP, VIBER, P2P. None defaults to all non-P2P types.
    pub message_type: Option<&'a str>,
    /// Filter by channel — "tenDLC", "virtualLongCode", "tollFree", "shortCode", or "p2p" (single value, or a comma-separated
    /// list). Filters on each message's stored channel; older messages without one count as tenDLC.
    pub channel: Option<&'a str>,
    pub carrier: Option<&'a str>,
    /// Filter by sentiment bucket — "positive", "neutral", or "negative" (single value, or a comma-separated
    /// list). Applied as the sentimentScore range the label threshold produces, so unscored messages match no
    /// bucket and outbound messages (never scored) are excluded.
    pub sentiment_label: Option<&'a str>,
    /// Filter FAILED messages by Signal House error code (single value, or a comma-separated list). "OUT" is the
    /// campaign opt-out, displayed as RECIPIENT_OPTED_OUT. Carrier codes are not filterable here; they live inside
    /// successOrFailureReason.
    pub error_code: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    /// Field to sort by — "createdAt", "segmentCount", "status", or "sentimentScore".
    pub sort_field: Option<&'a str>,
    pub sort_order: Option<&'a str>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    /// Which contract binds `start_date`/`end_date`. `"day"`: each bound is widened to its whole
    /// UTC day. `"exact"`: a timestamp is bound as the instant it
    /// names and a bare date is its whole UTC day. `None`: `"day"`.
    pub date_bounds: Option<&'a str>,
    /// Filter by region — `"US"`, `"GB"`, or `"CA"` (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB; single value, or a comma-separated
    /// list).
    pub region: Option<&'a str>,
    /// JSON array of OR-ed branches `{region, channels, brandId?, campaignId?, phoneNumber?}` (channels: tenDLC,
    /// virtualLongCode, tollFree, shortCode, p2p); `"[]"` matches nothing.
    pub region_scopes: Option<&'a str>,
}

/// Optional parameters for [`MessagesDomain::get_analytics`].
///
/// `subgroup_id`, `brand_id`, `campaign_id`, `phone_number` and `carrier` each accept a single
/// value or a comma-separated list of values. `group_id` is single-valued.
#[derive(Debug, Default)]
pub struct GetAnalyticsParams<'a> {
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    pub brand_id: Option<&'a str>,
    pub campaign_id: Option<&'a str>,
    /// Filter to traffic attributed to these registrations (hourly analytics only; a 400 while
    /// hourly reads are off). A `phone_number` filter takes precedence. Single value, or a
    /// comma-separated list.
    pub registration_id: Option<&'a str>,
    pub phone_number: Option<&'a str>,
    pub carrier: Option<&'a str>,
    /// Filter by the recipient's resolved carrier family (Default, ATT, TMobile, Verizon,
    /// USCellular, GoogleVoice, ClearSky, Interop, RogueMobile, P2P, Standard, IceWireless, Bell, Rogers, Telus, Videotron, SaskTel, Eastlink, Fibernetics); single value, or a
    /// comma-separated list. Only accepted where the environment serves hourly analytics;
    /// elsewhere any value is rejected with a 400 -- omit the parameter unless it is wanted.
    pub carrier_family: Option<&'a str>,
    /// Scopes the SENTIMENT figures to "SMS", "MMS" or "P2P" (single value, or a comma-separated
    /// list). `None` means all scored types. Does NOT narrow the message counts, which are always
    /// returned split per type and have no filterable type dimension.
    pub message_type: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    /// Filter by channel — "tenDLC", "virtualLongCode", "tollFree", "shortCode", or "p2p" (single value, or a comma-separated
    /// list). Filters on the stored channel; a tenDLC selection also includes older messages with
    /// no channel, and p2p is matched by carrier.
    pub channel: Option<&'a str>,
    /// Which contract binds `start_date`/`end_date`. `"day"`: each bound is widened to its whole
    /// UTC day. `"exact"`: a timestamp is bound as the instant it
    /// names and a bare date is its whole UTC day. `None`: `"day"`.
    pub date_bounds: Option<&'a str>,
    /// Filter by region — `"US"`, `"GB"`, or `"CA"` (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB; single value, or a comma-separated
    /// list). Only accepted where the environment serves hourly analytics; elsewhere any value is rejected with a 400.
    pub region: Option<&'a str>,
    /// JSON array of OR-ed branches `{region, channels, brandId?, campaignId?, phoneNumber?}` (channels: tenDLC,
    /// virtualLongCode, tollFree, shortCode, p2p); `"[]"` matches nothing. Same hourly-analytics gate as `region`.
    pub region_scopes: Option<&'a str>,
}

/// Optional parameters for [`MessagesDomain::get_analytics_detail`].
///
/// `subgroup_id`, `brand_id`, `campaign_id`, `phone_number` and `carrier` each accept a single
/// value or a comma-separated list of values. `group_id` is single-valued.
#[derive(Debug, Default)]
pub struct GetAnalyticsDetailParams<'a> {
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    pub brand_id: Option<&'a str>,
    pub campaign_id: Option<&'a str>,
    /// Filter to traffic attributed to these registrations (hourly analytics only; a 400 while
    /// hourly reads are off). A `phone_number` filter takes precedence. Single value, or a
    /// comma-separated list.
    pub registration_id: Option<&'a str>,
    pub phone_number: Option<&'a str>,
    pub carrier: Option<&'a str>,
    /// Filter by the recipient's resolved carrier family (Default, ATT, TMobile, Verizon,
    /// USCellular, GoogleVoice, ClearSky, Interop, RogueMobile, P2P, Standard, IceWireless, Bell, Rogers, Telus, Videotron, SaskTel, Eastlink, Fibernetics); single value, or a
    /// comma-separated list. Only accepted where the environment serves hourly analytics;
    /// elsewhere any value is rejected with a 400 -- omit the parameter unless it is wanted.
    pub carrier_family: Option<&'a str>,
    /// Scopes the SENTIMENT figures to "SMS", "MMS" or "P2P" (single value, or a comma-separated
    /// list). `None` means all scored types. Does NOT narrow the message counts, which are always
    /// returned split per type and have no filterable type dimension.
    pub message_type: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    /// Filter by channel — "tenDLC", "virtualLongCode", "tollFree", "shortCode", or "p2p" (single value, or a comma-separated
    /// list). Filters on the stored channel; a tenDLC selection also includes older messages with
    /// no channel, and p2p is a real channel here.
    pub channel: Option<&'a str>,
    /// Time-bucket grain of the `byDate` rows: `"day"` or `"hour"`. `None` means day (each row's
    /// `_id` is a `"YYYY-MM-DD"` date). `"hour"` buckets by hour (each row's `_id` is a
    /// `"YYYY-MM-DD HH:MM:SS"` UTC hour; `start_date` is floored to its hour and `end_date` honoured
    /// as given) and is capped to a 7-day span. Only accepted where the environment serves hourly
    /// analytics; elsewhere any value, including `"day"`, is rejected with a 400 -- leave it `None`
    /// unless hour grain is wanted.
    pub granularity: Option<&'a str>,
    /// "carrier" or "carrierFamily". Adds a byBreakdown time × dimension series to the response
    /// ({dimension, rows}: each row is one byDate bucket (_id) for one carrier or carrier family
    /// (key), top 50 keys by volume, excluding P2P) and caps the span at 31 days. Only accepted where
    /// the environment serves hourly analytics; elsewhere any value is rejected with a 400 -- omit
    /// the parameter unless it is wanted.
    pub breakdown: Option<&'a str>,
    /// Which contract binds `start_date`/`end_date`. `"day"`: each bound is widened to its whole
    /// UTC day. `"exact"`: a timestamp is bound as the instant it
    /// names and a bare date is its whole UTC day. `None`: `"day"`.
    pub date_bounds: Option<&'a str>,
    /// Filter by region — `"US"`, `"GB"`, or `"CA"` (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB; single value, or a comma-separated
    /// list). Only accepted where the environment serves hourly analytics; elsewhere any value is rejected with a 400.
    pub region: Option<&'a str>,
    /// JSON array of OR-ed branches `{region, channels, brandId?, campaignId?, phoneNumber?}` (channels: tenDLC,
    /// virtualLongCode, tollFree, shortCode, p2p); `"[]"` matches nothing. Same hourly-analytics gate as `region`.
    pub region_scopes: Option<&'a str>,
}

/// Parameters for [`MessagesDomain::get_analytics_throughput`].
///
/// `group_id`, `start_date` and `end_date` are required. `brand_id`, `campaign_id` and
/// `carrier_family` each accept a single value or a comma-separated list; `campaign_id` wins over
/// `brand_id`.
#[derive(Debug, Default)]
pub struct GetAnalyticsThroughputParams<'a> {
    pub group_id: Option<&'a str>,
    pub brand_id: Option<&'a str>,
    pub campaign_id: Option<&'a str>,
    /// Filter to traffic attributed to these registrations (hourly analytics only; a 400 while
    /// hourly reads are off). Single value, or a comma-separated list.
    pub registration_id: Option<&'a str>,
    /// Carrier-family filter (ATT, TMobile, Verizon, USCellular, ...).
    pub carrier_family: Option<&'a str>,
    /// ISO-8601, bound per `date_bounds` (a bare date is always that whole UTC day), then floored
    /// to the start of its hour at hour granularity and of its minute at day granularity (the
    /// cube is keyed by minute).
    pub start_date: Option<&'a str>,
    /// ISO-8601, bound per `date_bounds` (a bare date is always that whole UTC day). At day
    /// granularity in `"exact"` mode the buckets align to the UTC offset this bound carries. The
    /// span is capped at 35 days.
    pub end_date: Option<&'a str>,
    /// Bucket grain of the `byDate` and `byBreakdown` rows: `"hour"` (default; span capped at 7
    /// days) or `"day"`.
    pub granularity: Option<&'a str>,
    /// Segments-per-minute ceiling. When set, every row also reports `minutesAtCeiling` (sending
    /// minutes at or above it); when `None`, `minutesAtCeiling` is null.
    pub ceiling: Option<u64>,
    /// Which contract binds `start_date`/`end_date`. `"day"`: each bound is widened to its whole
    /// UTC day. `"exact"`: a timestamp is bound as the instant it
    /// names and a bare date is its whole UTC day. `None`: `"day"`.
    pub date_bounds: Option<&'a str>,
}

/// Parameters for [`MessagesDomain::get_analytics_filter_options`].
#[derive(Debug)]
pub struct GetAnalyticsFilterOptionsParams<'a> {
    pub group_id: &'a str,
    /// Restrict the options to one region — `"US"`, `"GB"`, or `"CA"` (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB; single value, or a
    /// comma-separated list). Only accepted where the environment serves hourly analytics; elsewhere any value is
    /// rejected with a 400.
    pub region: Option<&'a str>,
}

/// Optional parameters for [`MessagesDomain::get_analytics_by_subgroup`].
/// `limit` is capped server-side at 50 per page.
#[derive(Debug, Default)]
pub struct GetAnalyticsBySubgroupParams<'a> {
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    pub brand_id: Option<&'a str>,
    pub campaign_id: Option<&'a str>,
    /// Filter to traffic attributed to these registrations (hourly analytics only; a 400 while
    /// hourly reads are off). A `phone_number` filter takes precedence. Single value, or a
    /// comma-separated list.
    pub registration_id: Option<&'a str>,
    pub phone_number: Option<&'a str>,
    pub carrier: Option<&'a str>,
    /// Filter by the recipient's resolved carrier family (Default, ATT, TMobile, Verizon,
    /// USCellular, GoogleVoice, ClearSky, Interop, RogueMobile, P2P, Standard, IceWireless, Bell, Rogers, Telus, Videotron, SaskTel, Eastlink, Fibernetics); single value, or a
    /// comma-separated list. Only accepted where the environment serves hourly analytics;
    /// elsewhere any value is rejected with a 400 -- omit the parameter unless it is wanted.
    pub carrier_family: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    /// "both" (default, all activity) or "tenDLC" / "virtualLongCode" / "tollFree" / "shortCode" / "p2p" (single value, or a
    /// comma-separated list). Scopes ORDER BY + row inclusion by the stored channel column so a
    /// single-channel caller doesn't get pages dominated by other channels.
    pub channel: Option<&'a str>,
    /// SMS / MMS / P2P (single value, or a comma-separated list). Scopes ONLY the sentiment figures;
    /// omit for all scored types. It does not narrow the message counts, which are always returned
    /// split per type and have no filterable type dimension.
    pub message_type: Option<&'a str>,
    /// Which contract binds `start_date`/`end_date`. `"day"`: each bound is widened to its whole
    /// UTC day. `"exact"`: a timestamp is bound as the instant it
    /// names and a bare date is its whole UTC day. `None`: `"day"`.
    pub date_bounds: Option<&'a str>,
    /// Filter by region — `"US"`, `"GB"`, or `"CA"` (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB; single value, or a comma-separated
    /// list). Only accepted where the environment serves hourly analytics; elsewhere any value is rejected with a 400.
    pub region: Option<&'a str>,
    /// JSON array of OR-ed branches `{region, channels, brandId?, campaignId?, phoneNumber?}` (channels: tenDLC,
    /// virtualLongCode, tollFree, shortCode, p2p); `"[]"` matches nothing. Same hourly-analytics gate as `region`.
    pub region_scopes: Option<&'a str>,
}

/// Optional parameters for [`MessagesDomain::get_analytics_by_error_code`].
/// `limit` is capped server-side at 50 per page.
#[derive(Debug, Default)]
pub struct GetAnalyticsByErrorCodeParams<'a> {
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    pub brand_id: Option<&'a str>,
    pub campaign_id: Option<&'a str>,
    /// Filter to traffic attributed to these registrations (hourly analytics only; a 400 while
    /// hourly reads are off). A `phone_number` filter takes precedence. Single value, or a
    /// comma-separated list.
    pub registration_id: Option<&'a str>,
    pub phone_number: Option<&'a str>,
    pub carrier: Option<&'a str>,
    /// Filter by the recipient's resolved carrier family (Default, ATT, TMobile, Verizon,
    /// USCellular, GoogleVoice, ClearSky, Interop, RogueMobile, P2P, Standard, IceWireless, Bell, Rogers, Telus, Videotron, SaskTel, Eastlink, Fibernetics); single value, or a
    /// comma-separated list. Only accepted where the environment serves hourly analytics;
    /// elsewhere any value is rejected with a 400 -- omit the parameter unless it is wanted.
    pub carrier_family: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    /// "both" (default, all activity) or "tenDLC" / "virtualLongCode" / "tollFree" / "shortCode" / "p2p" (single value, or a
    /// comma-separated list). Scopes ORDER BY + row inclusion by the stored channel column so a
    /// single-channel caller doesn't get pages dominated by other channels.
    pub channel: Option<&'a str>,
    /// Which contract binds `start_date`/`end_date`. `"day"`: each bound is widened to its whole
    /// UTC day. `"exact"`: a timestamp is bound as the instant it
    /// names and a bare date is its whole UTC day. `None`: `"day"`.
    pub date_bounds: Option<&'a str>,
    /// Filter by region — `"US"`, `"GB"`, or `"CA"` (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB; single value, or a comma-separated
    /// list). Only accepted where the environment serves hourly analytics; elsewhere any value is rejected with a 400.
    pub region: Option<&'a str>,
    /// JSON array of OR-ed branches `{region, channels, brandId?, campaignId?, phoneNumber?}` (channels: tenDLC,
    /// virtualLongCode, tollFree, shortCode, p2p); `"[]"` matches nothing. Same hourly-analytics gate as `region`.
    pub region_scopes: Option<&'a str>,
    /// Narrow the breakdown to a single failure reason: a numeric Infobip carrier code (as a
    /// string, e.g. `"607"`), the Signal House campaign-opt-out code `"OUT"`, or a no-code failure
    /// class (`other`, `moderation`, `att_limit`, `p2p_stop`, `campaign_opt_out`, `dnc_blocked`,
    /// `bind_hold_expired`). `None`: every reason.
    pub error_code: Option<&'a str>,
}

/// Optional parameters for [`MessagesDomain::get_dnc_records`].
#[derive(Debug, Default)]
pub struct GetDncRecordsParams<'a> {
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    pub brand_id: Option<&'a str>,
    pub campaign_id: Option<&'a str>,
    /// Filter to traffic attributed to these registrations (hourly analytics only; a 400 while
    /// hourly reads are off). A `phone_number` filter takes precedence. Single value, or a
    /// comma-separated list.
    pub registration_id: Option<&'a str>,
    pub phone_number: Option<&'a str>,
    pub carrier: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_field: Option<&'a str>,
    pub sort_order: Option<&'a str>,
    /// Filter by channel — "tenDLC", "virtualLongCode", "tollFree", or "shortCode" (single value, or a comma-separated list).
    /// Opt-outs are A2P-only, so "p2p" applies no filter. A tenDLC selection also includes older
    /// opt-outs with no channel.
    pub channel: Option<&'a str>,
    /// Which contract binds `start_date`/`end_date`. `"day"`: each bound is widened to its whole
    /// UTC day. `"exact"`: a timestamp is bound as the instant it
    /// names and a bare date is its whole UTC day. `None`: `"day"`.
    pub date_bounds: Option<&'a str>,
    /// Filter by region — `"US"`, `"GB"`, or `"CA"` (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB; single value, or a comma-separated
    /// list). Only accepted where the environment serves hourly analytics; elsewhere any value is rejected with a 400.
    pub region: Option<&'a str>,
    /// JSON array of OR-ed branches `{region, channels, brandId?, campaignId?, phoneNumber?}` (channels: tenDLC,
    /// virtualLongCode, tollFree, shortCode, p2p); `"[]"` matches nothing. Same hourly-analytics gate as `region`.
    pub region_scopes: Option<&'a str>,
}

/// Optional parameters for [`MessagesDomain::get_dnc_analytics`].
#[derive(Debug, Default)]
pub struct GetDncAnalyticsParams<'a> {
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    pub brand_id: Option<&'a str>,
    pub campaign_id: Option<&'a str>,
    /// Filter to traffic attributed to these registrations (hourly analytics only; a 400 while
    /// hourly reads are off). A `phone_number` filter takes precedence. Single value, or a
    /// comma-separated list.
    pub registration_id: Option<&'a str>,
    pub phone_number: Option<&'a str>,
    pub carrier: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    /// Filter by channel — "tenDLC", "virtualLongCode", "tollFree", or "shortCode" (single value, or a comma-separated list).
    /// Opt-outs are A2P-only, so "p2p" applies no filter. A tenDLC selection also includes older
    /// opt-outs with no channel.
    pub channel: Option<&'a str>,
    /// Which contract binds `start_date`/`end_date`. `"day"`: each bound is widened to its whole
    /// UTC day. `"exact"`: a timestamp is bound as the instant it
    /// names and a bare date is its whole UTC day. `None`: `"day"`.
    pub date_bounds: Option<&'a str>,
    /// Filter by region — `"US"`, `"GB"`, or `"CA"` (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB; single value, or a comma-separated
    /// list). Only accepted where the environment serves hourly analytics; elsewhere any value is rejected with a 400.
    pub region: Option<&'a str>,
    /// JSON array of OR-ed branches `{region, channels, brandId?, campaignId?, phoneNumber?}` (channels: tenDLC,
    /// virtualLongCode, tollFree, shortCode, p2p); `"[]"` matches nothing. Same hourly-analytics gate as `region`.
    pub region_scopes: Option<&'a str>,
}

/// Messages domain -- send SMS/MMS, get message logs and analytics.
///
/// Date-range parameters (`start_date`, `end_date`) on list and analytics
/// methods accept ISO-8601 date or timestamp strings, bound per `date_bounds`. A
/// bare date is always that whole UTC day. A timestamp is widened to its whole UTC
/// day in `"day"` mode and bound as the instant it
/// names, in whatever offset it was written (`"2026-09-14T00:00:00-04:00"` is
/// 04:00Z), in `"exact"` mode, where day buckets also align to the offset
/// `end_date` carries. Omitted, `date_bounds` is `"day"`.
/// Business-number filters and SMS/MMS senders accept digits-only 5-6 digit
/// Short Codes or 10+ digit long numbers. Recipient numbers remain 10+ digits.
pub struct MessagesDomain {
    http: Arc<HttpClient>,
}

impl MessagesDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a list of messages with optional filters.
    ///
    /// Inbound messages in the response also carry `sentimentScore` (-100..100, null when
    /// unscored), `sentimentLabel` ("positive", "neutral" or "negative", null when
    /// `sentimentScore` is null) and `sentimentScoredAt`. Outbound messages are never scored.
    ///
    /// **GET** `/message?...`
    pub async fn get_messages(
        &self,
        params: GetMessagesParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("id", params.id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("registrationId", params.registration_id.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("groupId", params.group_id.map(String::from)),
            ("phoneNumber", params.phone_number.map(String::from)),
            (
                "senderPhoneNumber",
                params.sender_phone_number.map(String::from),
            ),
            (
                "recipientPhoneNumber",
                params.recipient_phone_number.map(String::from),
            ),
            ("status", params.status.map(String::from)),
            ("direction", params.direction.map(String::from)),
            ("messageType", params.message_type.map(String::from)),
            ("channel", params.channel.map(String::from)),
            ("region", params.region.map(String::from)),
            ("regionScopes", params.region_scopes.map(String::from)),
            ("carrier", params.carrier.map(String::from)),
            ("sentimentLabel", params.sentiment_label.map(String::from)),
            ("errorCode", params.error_code.map(String::from)),
            ("startDate", params.start_date.map(String::from)),
            ("endDate", params.end_date.map(String::from)),
            ("sortField", params.sort_field.map(String::from)),
            ("sortOrder", params.sort_order.map(String::from)),
            ("page", params.page.map(|p| p.to_string())),
            ("limit", params.limit.map(|l| l.to_string())),
            ("dateBounds", params.date_bounds.map(String::from)),
        ]);
        self.http
            .request_json(&format!("/message{}", qs), Method::GET, None, options)
            .await
    }

    /// Get aggregated analytics for messages.
    ///
    /// The response also includes inbound-message sentiment for the requested scope:
    /// `sentimentScore` (volume-weighted average of every scored inbound message in range,
    /// -100..100, null when nothing was scored), `sentimentLabel`, `sentimentScoredCount`,
    /// `sentimentPositive`, `sentimentNeutral` and `sentimentNegative`. `sentimentScore` is an
    /// average, so it must never be summed across responses.
    ///
    /// **GET** `/message/analytics?...`
    pub async fn get_analytics(
        &self,
        params: GetAnalyticsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("registrationId", params.registration_id.map(String::from)),
            ("phoneNumber", params.phone_number.map(String::from)),
            ("carrier", params.carrier.map(String::from)),
            ("carrierFamily", params.carrier_family.map(String::from)),
            ("startDate", params.start_date.map(String::from)),
            ("endDate", params.end_date.map(String::from)),
            ("channel", params.channel.map(String::from)),
            ("region", params.region.map(String::from)),
            ("regionScopes", params.region_scopes.map(String::from)),
            ("messageType", params.message_type.map(String::from)),
            ("dateBounds", params.date_bounds.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/message/analytics{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get detailed analytics snapshot records for charting and aggregation.
    ///
    /// `cardTotals` and every `byDate`/`byPhoneNumber`/`byCarrier` row also carry that row's own
    /// inbound-message sentiment: `sentimentScore` (volume-weighted average, -100..100, null when
    /// nothing in that row's scope was scored), `sentimentLabel`, `sentimentScoredCount`,
    /// `sentimentPositive`, `sentimentNeutral` and `sentimentNegative`. The fields are always
    /// present; `sentimentScore` is an average and must never be summed across rows or responses.
    ///
    /// **GET** `/message/analytics/detail?...`
    pub async fn get_analytics_detail(
        &self,
        params: GetAnalyticsDetailParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("registrationId", params.registration_id.map(String::from)),
            ("phoneNumber", params.phone_number.map(String::from)),
            ("carrier", params.carrier.map(String::from)),
            ("carrierFamily", params.carrier_family.map(String::from)),
            ("startDate", params.start_date.map(String::from)),
            ("endDate", params.end_date.map(String::from)),
            ("channel", params.channel.map(String::from)),
            ("region", params.region.map(String::from)),
            ("regionScopes", params.region_scopes.map(String::from)),
            ("messageType", params.message_type.map(String::from)),
            ("granularity", params.granularity.map(String::from)),
            ("breakdown", params.breakdown.map(String::from)),
            ("dateBounds", params.date_bounds.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/message/analytics/detail{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get per-minute send throughput (peak segments per minute, sending minutes, minutes at a
    /// ceiling) per time bucket and per carrier family. Measured from SENT transitions at
    /// campaign × carrier-family grain — the grain carrier ceilings are enforced at — so a peak
    /// here is the figure a ceiling is compared against. Only accepted where the environment
    /// serves hourly analytics; elsewhere the request is rejected with a 400.
    ///
    /// Response: `totals`, `byDate`, `byCarrierFamily` and `byBreakdown` (`{dimension:
    /// "carrierFamily", rows}`). Every row carries `segments`, `messages`,
    /// `peakSegmentsPerMinute`, `peakMessagesPerMinute`, `peakMinute` (UTC
    /// `"YYYY-MM-DD HH:MM:SS"`, null when no traffic), `sendingMinutes`,
    /// `averageSegmentsPerSendingMinute` and `minutesAtCeiling`.
    ///
    /// **GET** `/message/analytics/throughput?...`
    pub async fn get_analytics_throughput(
        &self,
        params: GetAnalyticsThroughputParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", params.group_id),
            ("startDate", params.start_date),
            ("endDate", params.end_date),
        ])?;

        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("registrationId", params.registration_id.map(String::from)),
            ("carrierFamily", params.carrier_family.map(String::from)),
            ("startDate", params.start_date.map(String::from)),
            ("endDate", params.end_date.map(String::from)),
            ("granularity", params.granularity.map(String::from)),
            ("ceiling", params.ceiling.map(|c| c.to_string())),
            ("dateBounds", params.date_bounds.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/message/analytics/throughput{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get filter dropdown options (subgroups, brands, campaigns, phone numbers, carriers) for the
    /// Analytics page, scoped to a single group. Only items with
    /// at least one message are returned. Each phone number carries `type` (TOLL_FREE, SHORT_CODE,
    /// ALPHANUMERIC, VIRTUAL_LONG_NUMBER, ...) and `country` (ISO-2), null when unknown: TOLL_FREE,
    /// SHORT_CODE and ALPHANUMERIC are channels by type; any other US number is 10DLC and a non-US one
    /// is a Virtual Long Code. `carriers` is a plain array of carrier name strings.
    ///
    /// **GET** `/message/analytics/filter-options?...`
    pub async fn get_analytics_filter_options(
        &self,
        params: GetAnalyticsFilterOptionsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(params.group_id.to_string())),
            ("region", params.region.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/message/analytics/filter-options{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a paginated breakdown of message metrics grouped by subgroup. Each row carries the
    /// full set of sms/mms/p2p metric columns plus `smsOptOuts` and `mmsOptOuts` (sourced from
    /// the DNC analytics MV — P2P has no opt-outs) so callers can apply channel toggles client-side.
    ///
    /// Each row also carries that subgroup's inbound-message sentiment: `sentimentScore`
    /// (volume-weighted, -100..100, null when nothing was scored), `sentimentLabel`,
    /// `sentimentScoredCount` and the positive/neutral/negative counts, plus the same six per
    /// message type under `smsSentiment*`, `mmsSentiment*` and `p2pSentiment*`. The score is
    /// derived server-side; combine counts across rows, never the scores.
    ///
    /// **GET** `/message/analytics/by-subgroup?...`
    pub async fn get_analytics_by_subgroup(
        &self,
        params: GetAnalyticsBySubgroupParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("registrationId", params.registration_id.map(String::from)),
            ("phoneNumber", params.phone_number.map(String::from)),
            ("carrier", params.carrier.map(String::from)),
            ("carrierFamily", params.carrier_family.map(String::from)),
            ("startDate", params.start_date.map(String::from)),
            ("endDate", params.end_date.map(String::from)),
            ("page", params.page.map(|n| n.to_string())),
            ("limit", params.limit.map(|n| n.to_string())),
            ("channel", params.channel.map(String::from)),
            ("region", params.region.map(String::from)),
            ("regionScopes", params.region_scopes.map(String::from)),
            ("messageType", params.message_type.map(String::from)),
            ("dateBounds", params.date_bounds.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/message/analytics/by-subgroup{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a paginated breakdown of failed messages grouped by error code. Each row contains
    /// per-channel (sms/mms/p2p) error counts plus an enriched description.
    ///
    /// **GET** `/message/analytics/by-error-code?...`
    pub async fn get_analytics_by_error_code(
        &self,
        params: GetAnalyticsByErrorCodeParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("registrationId", params.registration_id.map(String::from)),
            ("phoneNumber", params.phone_number.map(String::from)),
            ("carrier", params.carrier.map(String::from)),
            ("carrierFamily", params.carrier_family.map(String::from)),
            ("startDate", params.start_date.map(String::from)),
            ("endDate", params.end_date.map(String::from)),
            ("page", params.page.map(|n| n.to_string())),
            ("limit", params.limit.map(|n| n.to_string())),
            ("channel", params.channel.map(String::from)),
            ("region", params.region.map(String::from)),
            ("regionScopes", params.region_scopes.map(String::from)),
            ("dateBounds", params.date_bounds.map(String::from)),
            ("errorCode", params.error_code.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/message/analytics/by-error-code{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get paginated Do Not Call records.
    ///
    /// **GET** `/message/dnc?...`
    ///
    /// Each record carries `senderPhoneNumber` — the first business number that received the
    /// opt-out (set once, never changed by a later STOP or a campaign move; the same value as
    /// `phoneNumber`), `campaignId` (`null` on subgroup-scoped records for non-US recipients),
    /// `optOutKeyword` — the normalized keyword that revoked consent (e.g.
    /// "stop", "opt out"; a repeat opt-out records the most recent word), `null` for opt-outs
    /// recorded before keyword capture shipped — and `carrierFamily`, the resolved carrier family
    /// of the recipient number (Default, ATT, TMobile, Verizon, USCellular, GoogleVoice, ClearSky,
    /// Interop, RogueMobile, Standard, IceWireless, Bell, Rogers, Telus, Videotron, SaskTel, Eastlink, Fibernetics), copied from the inbound message that triggered the opt-out.
    pub async fn get_dnc_records(
        &self,
        params: GetDncRecordsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("registrationId", params.registration_id.map(String::from)),
            ("phoneNumber", params.phone_number.map(String::from)),
            ("carrier", params.carrier.map(String::from)),
            ("startDate", params.start_date.map(String::from)),
            ("endDate", params.end_date.map(String::from)),
            ("page", params.page.map(|p| p.to_string())),
            ("limit", params.limit.map(|l| l.to_string())),
            ("sortField", params.sort_field.map(String::from)),
            ("sortOrder", params.sort_order.map(String::from)),
            ("channel", params.channel.map(String::from)),
            ("region", params.region.map(String::from)),
            ("regionScopes", params.region_scopes.map(String::from)),
            ("dateBounds", params.date_bounds.map(String::from)),
        ]);
        self.http
            .request_json(&format!("/message/dnc{}", qs), Method::GET, None, options)
            .await
    }

    /// Get aggregated DNC (Do Not Contact) opt-out analytics.
    ///
    /// **GET** `/message/dnc/analytics?...`
    ///
    /// Returns `totals`, `byDate`, `byPhoneNumber`, `bySenderNumber`, `byCampaign`, `bySubgroup`,
    /// `byType`, `byCarrier`, `byKeyword`. `bySenderNumber` (`{senderPhoneNumber, total, sms, mms}`)
    /// attributes each opt-out to the first business number that received it, so it is unchanged by
    /// later campaign moves and START replies. `byCampaign` counts campaign-scoped opt-outs only, by
    /// the campaign at the time of the STOP; subgroup-scoped opt-outs (non-US recipients) are never
    /// attributed to a campaign and appear under `bySubgroup` instead. `byType` splits SMS/MMS.
    /// `byKeyword` breaks
    /// opt-outs down by the normalized keyword that revoked consent (`{optOutKeyword, total, sms,
    /// mms}`); opt-outs recorded before keyword capture shipped appear as a single
    /// `optOutKeyword: null` bucket ("not recorded"). The distinct keyword set is open-ended
    /// (campaigns register their own keywords beyond the mandatory floor) — render top-N plus
    /// Other, never a fixed list.
    pub async fn get_dnc_analytics(
        &self,
        params: GetDncAnalyticsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("registrationId", params.registration_id.map(String::from)),
            ("phoneNumber", params.phone_number.map(String::from)),
            ("carrier", params.carrier.map(String::from)),
            ("startDate", params.start_date.map(String::from)),
            ("endDate", params.end_date.map(String::from)),
            ("channel", params.channel.map(String::from)),
            ("region", params.region.map(String::from)),
            ("regionScopes", params.region_scopes.map(String::from)),
            ("dateBounds", params.date_bounds.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/message/dnc/analytics{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Send an SMS message.
    ///
    /// **POST** `/message/sms`
    ///
    /// `recipient_phone_numbers` may be a JSON string or a JSON array of
    /// strings (e.g. `json!("+15559876543")` or `json!(["+15559876543"])`).
    /// `sender_phone_number` accepts a digits-only 5-6 digit Short Code or 10+
    /// digit long number, or a READY alphanumeric sender ID (e.g. `"ACME"`; UK
    /// recipients and SMS only). Every message from an alphanumeric sender gets
    /// `"\nOpt out: sihou.io/o/{code}"` appended by the server, counted in
    /// segments. A Short Code sender permits exactly one 10+ digit
    /// recipient, and it must be a +1 (US/Canada) number (a bare 10-digit
    /// number is treated as +1); the API rejects multiple recipients or a
    /// non-+1 recipient.
    ///
    /// The response reports the send outcome via `requestedRecipientCount`, `enqueuedCount`, and
    /// `failedCount`. `enqueuedCount` is the number of recipients accepted for delivery and is 0 when
    /// the send was fully blocked (by the moderation filter or a campaign opt-out); the request still
    /// succeeds in that case, so treat `enqueuedCount`, not the HTTP status, as the signal that a
    /// message was accepted. Blocked recipients are returned in `insertedMessages` with status
    /// "FAILED"; recipients on the global Do Not Contact list are reported in `dncBlockedNumbers` with
    /// no message record. Each FAILED message carries `errorCode`, a stable machine-readable cause
    /// (campaign opt-out is "OUT"; null on success or on failures with no assigned code) that is also
    /// included on the FAILED status-callback payload.
    pub async fn send_sms(
        &self,
        sender_phone_number: &str,
        recipient_phone_numbers: &Value,
        message_body: &str,
        status_callback_url: Option<&str>,
        enable_shortlink: bool,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.send_sms_filtered(
            sender_phone_number,
            recipient_phone_numbers,
            message_body,
            status_callback_url,
            enable_shortlink,
            false,
            options,
        )
        .await
    }

    /// Send an SMS message with landline/inactive number filtering.
    ///
    /// **POST** `/message/sms`
    ///
    /// Same as [`send_sms`](Self::send_sms) but with an additional
    /// `filter_landlines_and_inactive_numbers` flag. When `true`, landline
    /// and inactive numbers are removed before sending and returned in the
    /// response as `filteredNumbers`.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_sms_filtered(
        &self,
        sender_phone_number: &str,
        recipient_phone_numbers: &Value,
        message_body: &str,
        status_callback_url: Option<&str>,
        enable_shortlink: bool,
        filter_landlines_and_inactive_numbers: bool,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("senderPhoneNumber", Some(sender_phone_number)),
            ("messageBody", Some(message_body)),
        ])?;

        let mut body = json!({
            "senderPhoneNumber": sender_phone_number,
            "recipientPhoneNumber": recipient_phone_numbers,
            "messageBody": message_body,
            "statusCallbackUrl": status_callback_url,
            "enableShortlink": enable_shortlink,
        });
        if filter_landlines_and_inactive_numbers {
            body["filterLandlinesAndInactiveNumbers"] = json!(true);
        }
        self.http
            .request_json("/message/sms", Method::POST, Some(&body), options)
            .await
    }

    /// Send a P2P message via Rogue Mobile SMPP.
    ///
    /// `group_id` is optional: explicit group to send from; defaults to the group in
    /// your auth context. `subgroup_id` is optional: explicit subgroup to attribute
    /// the send (and its inbound reply) to. P2P has no dedicated sender number, so pass
    /// this to record the send against a specific subgroup; it also resolves the group.
    /// Defaults to the group's oldest subgroup.
    ///
    /// `use_signal_house_shortlinks` is optional. When `Some(false)`, SignalHouse
    /// applies no link-shortening or text-spin and your pre-spun links/content are
    /// sent verbatim (bring-your-own spinner). Defaults to true. Pass `None` to omit
    /// the field entirely and preserve the server default.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_p2p(
        &self,
        recipient_phone_numbers: &Value,
        message_body: &str,
        status_callback_url: Option<&str>,
        use_signal_house_shortlinks: Option<bool>,
        group_id: Option<&str>,
        subgroup_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("messageBody", Some(message_body))])?;

        let mut body = json!({
            "recipientPhoneNumber": recipient_phone_numbers,
            "messageBody": message_body,
            "statusCallbackUrl": status_callback_url,
        });
        if let Some(group_id) = group_id {
            body["groupId"] = json!(group_id);
        }
        if let Some(subgroup_id) = subgroup_id {
            body["subgroupId"] = json!(subgroup_id);
        }
        if let Some(use_shortlinks) = use_signal_house_shortlinks {
            body["useSignalHouseShortlinks"] = json!(use_shortlinks);
        }
        self.http
            .request_json("/message/p2p", Method::POST, Some(&body), options)
            .await
    }

    /// Get a P2P batch by its ID.
    ///
    /// **GET** `/message/p2p/batches/:batchId`
    pub async fn get_p2p_batch(
        &self,
        batch_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("batchId", Some(batch_id))])?;
        let safe = urlencoding::encode(batch_id);
        self.http
            .request_json(
                &format!("/message/p2p/batches/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Fetch the latest upstream carrier delivery report for a set of messages by ID.
    ///
    /// **POST** `/message/delivery-report`
    ///
    /// Useful for reconciling messages stuck in a non-terminal state. Outbound SMS and MMS messages sent
    /// from numbers on supported carriers are looked up upstream (reports are retained about
    /// 48 hours); every other message, including P2P messages and messages sent from numbers on other
    /// carriers, is returned with its stored status and `upstreamReportAvailable` false. The request fails
    /// with a 500 when the sending numbers can't be read.
    ///
    /// - `message_ids`: a JSON array of message IDs (1-100).
    pub async fn get_delivery_reports(
        &self,
        message_ids: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({
            "messageIds": message_ids,
        });
        self.http
            .request_json(
                "/message/delivery-report",
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Send an MMS message (multipart).
    ///
    /// **POST** `/message/mms`
    ///
    /// - `images`: optional list of `(bytes, filename)` tuples.
    /// - `sender_phone_number`: digits-only 5-6 digit Short Code or 10+ digit
    ///   long number. A Short Code sender permits exactly one 10+ digit
    ///   recipient, and it must be a +1 (US/Canada) number (a bare 10-digit
    ///   number is treated as +1); the API rejects multiple recipients or a
    ///   non-+1 recipient.
    ///
    /// The response reports the send outcome via `requestedRecipientCount`, `enqueuedCount`, and
    /// `failedCount`. `enqueuedCount` is the number of recipients accepted for delivery and is 0 when
    /// the send was fully blocked (by the moderation filter or a campaign opt-out); the request still
    /// succeeds in that case, so treat `enqueuedCount`, not the HTTP status, as the signal that a
    /// message was accepted. Blocked recipients are returned in `insertedMessages` with status
    /// "FAILED"; recipients on the global Do Not Contact list are reported in `dncBlockedNumbers` with
    /// no message record. Each FAILED message carries `errorCode`, a stable machine-readable cause
    /// (campaign opt-out is "OUT"; null on success or on failures with no assigned code) that is also
    /// included on the FAILED status-callback payload.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_mms(
        &self,
        sender_phone_number: &str,
        recipient_phone_numbers: &[String],
        message_body: &str,
        media_urls: Option<&[String]>,
        status_callback_url: Option<&str>,
        enable_shortlink: bool,
        enable_compression: bool,
        images: Option<Vec<(Vec<u8>, String)>>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.send_mms_filtered(
            sender_phone_number,
            recipient_phone_numbers,
            message_body,
            media_urls,
            status_callback_url,
            enable_shortlink,
            enable_compression,
            false,
            images,
            options,
        )
        .await
    }

    /// Send an MMS message with landline/inactive number filtering.
    ///
    /// **POST** `/message/mms`
    ///
    /// Same as [`send_mms`](Self::send_mms) but with an additional
    /// `filter_landlines_and_inactive_numbers` flag.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_mms_filtered(
        &self,
        sender_phone_number: &str,
        recipient_phone_numbers: &[String],
        message_body: &str,
        media_urls: Option<&[String]>,
        status_callback_url: Option<&str>,
        enable_shortlink: bool,
        enable_compression: bool,
        filter_landlines_and_inactive_numbers: bool,
        images: Option<Vec<(Vec<u8>, String)>>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("senderPhoneNumber", Some(sender_phone_number))])?;

        let form = Self::build_message_form(
            sender_phone_number,
            recipient_phone_numbers,
            message_body,
            media_urls,
            status_callback_url,
            enable_shortlink,
            enable_compression,
            filter_landlines_and_inactive_numbers,
            images,
        );

        self.http
            .request_multipart("/message/mms", Method::POST, form, options)
            .await
    }

    /// Send a group MMS message (multipart).
    ///
    /// **POST** `/message/groupMessage`
    ///
    /// Same parameters as [`send_mms`](Self::send_mms).
    /// Short Codes do not support group messaging; `sender_phone_number` must
    /// be a long number.
    ///
    /// The response reports the send outcome via `requestedRecipientCount`, `enqueuedCount`, and
    /// `failedCount`. `enqueuedCount` is the number of recipients accepted for delivery and is 0 when
    /// the send was fully blocked (by the moderation filter or a campaign opt-out); the request still
    /// succeeds in that case, so treat `enqueuedCount`, not the HTTP status, as the signal that a
    /// message was accepted. Blocked recipients are returned in `insertedMessages` with status
    /// "FAILED"; recipients on the global Do Not Contact list are reported in `dncBlockedNumbers` with
    /// no message record. Each FAILED message carries `errorCode`, a stable machine-readable cause
    /// (campaign opt-out is "OUT"; null on success or on failures with no assigned code) that is also
    /// included on the FAILED status-callback payload.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_group_message(
        &self,
        sender_phone_number: &str,
        recipient_phone_numbers: &[String],
        message_body: &str,
        media_urls: Option<&[String]>,
        status_callback_url: Option<&str>,
        enable_shortlink: bool,
        enable_compression: bool,
        images: Option<Vec<(Vec<u8>, String)>>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.send_group_message_filtered(
            sender_phone_number,
            recipient_phone_numbers,
            message_body,
            media_urls,
            status_callback_url,
            enable_shortlink,
            enable_compression,
            false,
            images,
            options,
        )
        .await
    }

    /// Send a group MMS message with landline/inactive number filtering.
    ///
    /// **POST** `/message/groupMessage`
    ///
    /// Same as [`send_group_message`](Self::send_group_message) but with an
    /// additional `filter_landlines_and_inactive_numbers` flag.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_group_message_filtered(
        &self,
        sender_phone_number: &str,
        recipient_phone_numbers: &[String],
        message_body: &str,
        media_urls: Option<&[String]>,
        status_callback_url: Option<&str>,
        enable_shortlink: bool,
        enable_compression: bool,
        filter_landlines_and_inactive_numbers: bool,
        images: Option<Vec<(Vec<u8>, String)>>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("senderPhoneNumber", Some(sender_phone_number))])?;

        let form = Self::build_message_form(
            sender_phone_number,
            recipient_phone_numbers,
            message_body,
            media_urls,
            status_callback_url,
            enable_shortlink,
            enable_compression,
            filter_landlines_and_inactive_numbers,
            images,
        );

        self.http
            .request_multipart("/message/groupMessage", Method::POST, form, options)
            .await
    }

    /// Look up the carrier for a phone number. Always performs a fresh lookup
    /// (not cached). Billed per request.
    ///
    /// **POST** `/message/carrier-id`
    pub async fn carrier_id_lookup(
        &self,
        phone_number: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;

        let body = json!({ "phoneNumber": phone_number });
        self.http
            .request_json("/message/carrier-id", Method::POST, Some(&body), options)
            .await
    }

    /// Estimate Canadian or UK retail rates and total in microdollars; no charge or send. A Ready Canadian sender
    /// takes SMS or MMS to Canadian or US recipients; a UK (+44) virtual long code takes SMS only to GB mobile
    /// recipients (10-15 digits). The response `region` is `"CA"` or `"GB"`. Unavailable unless that region is
    /// enabled for the environment. The sender may also be a READY alphanumeric sender ID (e.g. `"ACME"`; UK
    /// recipients, SMS only); its estimate counts the `"\nOpt out: sihou.io/o/{code}"` footer the server appends.
    pub async fn estimate_message(
        &self,
        sender_phone_number: &str,
        recipient_phone_numbers: &[String],
        message_body: &str,
        message_type: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({ "senderPhoneNumber": sender_phone_number, "recipientPhoneNumbers": recipient_phone_numbers, "messageBody": message_body, "messageType": message_type });
        self.http
            .request_json("/message/estimate", Method::POST, Some(&body), options)
            .await
    }

    // -- helpers ------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn build_message_form(
        sender_phone_number: &str,
        recipient_phone_numbers: &[String],
        message_body: &str,
        media_urls: Option<&[String]>,
        status_callback_url: Option<&str>,
        enable_shortlink: bool,
        enable_compression: bool,
        filter_landlines_and_inactive_numbers: bool,
        images: Option<Vec<(Vec<u8>, String)>>,
    ) -> Form {
        let mut form = Form::new();

        if let Some(imgs) = images {
            for (bytes, filename) in imgs {
                let part = Part::bytes(bytes).file_name(filename);
                form = form.part("images", part);
            }
        }

        form = form.text("senderPhoneNumber", sender_phone_number.to_string());

        let recipients_json =
            serde_json::to_string(recipient_phone_numbers).unwrap_or_else(|_| "[]".to_string());
        form = form.text("recipientPhoneNumber", recipients_json);

        form = form.text("messageBody", message_body.to_string());

        if let Some(urls) = media_urls {
            if !urls.is_empty() {
                let urls_json = serde_json::to_string(urls).unwrap_or_else(|_| "[]".to_string());
                form = form.text("mediaUrls", urls_json);
            }
        }

        if let Some(cb) = status_callback_url {
            form = form.text("statusCallbackUrl", cb.to_string());
        }

        form = form.text("enableShortlink", enable_shortlink.to_string());
        form = form.text("enableCompression", enable_compression.to_string());
        if filter_landlines_and_inactive_numbers {
            form = form.text("filterLandlinesAndInactiveNumbers", "true");
        }

        form
    }
}
