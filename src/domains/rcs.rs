use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde_json::{json, Map, Value};

use crate::client::HttpClient;
use crate::domains::registrations::LaunchAcknowledgements;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Parameters for [`RcsAgentsDomain::create`]. Unset fields are left out of the body.
#[derive(Debug, Default)]
pub struct CreateRcsAgentParams<'a> {
    /// The brand: carrier brand ID, provisional UUID or `_id`. Required.
    pub brand_id: &'a str,
    /// A campaign of that brand (campaign ID or `_id`); required before submit.
    pub campaign_id: Option<&'a str>,
    /// The profile: `{ name, description, color, logoUrl, heroUrl, phone, website, email,
    /// phoneLabel, websiteLabel, emailLabel, privacyPolicyUrl, termsUrl }`. The labels are the
    /// captions on the Call, Website and Email actions (max 40; submit defaults them to
    /// `Call`, `Website` and `Email`).
    pub agent: Option<&'a Value>,
    /// The launch text: `{ optInDetails, optInUrl, optInResponse, optOutDetails, optOutResponse,
    /// helpResponse, campaignDescription, messageExamples, estimatedMonthlyVolume, launchVideoUrl }`.
    /// Every field is required before the launch request. Blank fields fill from the linked campaign.
    pub consent: Option<&'a Value>,
    /// The launch contact: `{ firstName, lastName, role, email, mobile }`, the individual at the
    /// brand the carrier partner's reviewers may write to. Required in full before the launch request.
    pub contact: Option<&'a Value>,
    /// `OTP`, `TRANSACTIONAL`, `PROMOTIONAL` or `MULTI_USE`; required before submit.
    pub use_case: Option<&'a str>,
    /// `NON_CONVERSATIONAL` (default, per message) or `CONVERSATIONAL` (per interactive session).
    pub billing_category: Option<&'a str>,
    /// `{ mode (auto, rcs_only, sms_only, none), channel (auto, sms, mms), from }`, where `from` is
    /// a number on the linked campaign.
    pub default_fallback: Option<&'a Value>,
    /// `[]` or `["T_MOBILE"]` (the carrier partner's carrier name).
    pub excluded_carriers: Option<&'a [&'a str]>,
    /// Tester numbers (up to 200), invited once the agent exists at the carrier.
    pub testers: Option<&'a [&'a str]>,
    /// Where the fields you prefilled came from, by path (`consent.optInDetails`):
    /// `{ source (brand, campaign or landing), sourceId, at, edited }`. The API adds its own for
    /// the fields it fills.
    pub autofill: Option<&'a Value>,
}

/// Filters for [`RcsAgentsDomain::list`].
#[derive(Debug, Default)]
pub struct ListRcsAgentsParams<'a> {
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    /// One status or a comma-separated list from `DRAFT`, `PENDING_REVIEW`, `REJECTED`,
    /// `AGENT_CREATING`, `ARCHIVE_PENDING` and `ARCHIVED`.
    pub status: Option<&'a str>,
    /// Agents linked to this brand (carrier brand ID, provisional UUID or `_id`).
    pub brand_id: Option<&'a str>,
    /// Agents linked to this campaign (campaign ID or `_id`).
    pub campaign_id: Option<&'a str>,
    /// Defaults to 1.
    pub page: Option<u32>,
    /// Defaults to 25, at most 100.
    pub limit: Option<u32>,
}

/// Parameters for [`RcsAgentsDomain::update`]. Unset fields are left out of the body.
#[derive(Debug, Default)]
pub struct UpdateRcsAgentParams<'a> {
    /// The agent ID (e.g. `RCSA7K2M1QZ8RP`). Required.
    pub agent_id: &'a str,
    /// Relink to another brand of the same subgroup.
    pub brand_id: Option<&'a str>,
    /// `None` leaves the link unchanged; `Some(Some(id))` links or relinks; `Some(None)` sends
    /// `null` to unlink the campaign.
    pub campaign_id: Option<Option<&'a str>>,
    /// Profile fields to change; merged field by field.
    pub agent: Option<&'a Value>,
    /// Consent fields to change; merged field by field.
    pub consent: Option<&'a Value>,
    /// Launch contact fields to change; merged field by field.
    pub contact: Option<&'a Value>,
    /// `Some(None)` sends `null` to clear the use case.
    pub use_case: Option<Option<&'a str>>,
    pub billing_category: Option<&'a str>,
    /// `Some(None)` sends `null` to remove the fallback.
    pub default_fallback: Option<Option<&'a Value>>,
    /// `[]` or `["T_MOBILE"]` (the carrier partner's carrier name).
    pub excluded_carriers: Option<&'a [&'a str]>,
    pub testers: Option<&'a [&'a str]>,
    /// Replaces the prefill markers; `None` keeps the stored ones, and a changed field's marker
    /// turns edited.
    pub autofill: Option<&'a Value>,
}

/// Parameters for [`RcsMessagesDomain::send`], [`RcsMessagesDomain::send_batch`] and
/// [`RcsMessagesDomain::validate`]. Unset fields are left out of the body.
#[derive(Debug, Default)]
pub struct SendRcsMessageParams<'a> {
    /// The agent that sends (`RCSA…`). Required.
    pub agent_id: &'a str,
    /// Up to 100 recipients (10,000 on `send_batch`), each `{ phoneNumber, variables?,
    /// clientMessageId?, metadata? }`. Required.
    pub recipients: &'a [Value],
    /// A template to send; exactly one of `template_id` or `content`.
    pub template_id: Option<&'a str>,
    /// A published version of `template_id`.
    pub template_version: Option<u32>,
    /// Inline content; exactly one of `template_id` or `content`.
    pub content: Option<&'a Value>,
    /// `content.v1` (default) or `google.rcs`.
    pub content_format: Option<&'a str>,
    /// `AUTHENTICATION`, `TRANSACTION`, `SERVICEREQUEST`, `ACKNOWLEDGEMENT` or `PROMOTION`; required
    /// on a `MULTI_USE` agent when the template sets none.
    pub traffic_type: Option<&'a str>,
    /// Seconds the RCS message may wait for delivery; at most one of `ttl_seconds` or `expire_time`.
    pub ttl_seconds: Option<u32>,
    /// ISO-8601 time the RCS message expires.
    pub expire_time: Option<&'a str>,
    /// `{ mode? (auto, rcs_only, sms_only), channel? (auto, sms, mms), from?, content?: { text,
    /// mediaUrls? }, validitySeconds? }`. `from` is a Ready number on the agent's linked Active
    /// campaign. If `content` is omitted the template's fallback, else a generated one, is used.
    pub fallback: Option<&'a Value>,
    /// URL that receives this message's status events.
    pub status_callback_url: Option<&'a str>,
    /// Shorten the links in the message.
    pub enable_shortlink: Option<bool>,
    /// Send as a test; approved testers only.
    pub is_test: Option<bool>,
    /// `not_capable`, `ttl_expired`, `delivered` or `read`; approved testers only.
    pub simulate: Option<&'a str>,
}

/// Parameters for [`RcsMessagesDomain::estimate`]. Unset fields are left out of the body.
#[derive(Debug, Default)]
pub struct EstimateRcsMessageParams<'a> {
    /// The agent that would send (`RCSA…`). Required.
    pub agent_id: &'a str,
    /// Named recipients; exactly one of `recipients` or `recipient_count`.
    pub recipients: Option<&'a [Value]>,
    /// A recipient count; carriers are unknown, so the default fees apply.
    pub recipient_count: Option<u32>,
    /// A template to price; exactly one of `template_id` or `content`.
    pub template_id: Option<&'a str>,
    /// A published version of `template_id`.
    pub template_version: Option<u32>,
    /// Inline content; exactly one of `template_id` or `content`.
    pub content: Option<&'a Value>,
    /// `content.v1` (default) or `google.rcs`.
    pub content_format: Option<&'a str>,
    /// As on [`SendRcsMessageParams`].
    pub traffic_type: Option<&'a str>,
    /// Seconds the RCS message may wait for delivery; at most one of `ttl_seconds` or `expire_time`.
    pub ttl_seconds: Option<u32>,
    /// ISO-8601 time the RCS message expires.
    pub expire_time: Option<&'a str>,
    /// As on [`SendRcsMessageParams`], but its `from` is optional: with none, the leg is priced on
    /// the channel of the agent's linked campaign.
    pub fallback: Option<&'a Value>,
}

/// Parameters for [`RcsMessagesDomain::simulate_inbound`]. Unset fields are left out of the body.
#[derive(Debug, Default)]
pub struct SimulateRcsInboundParams<'a> {
    /// The agent (`RCSA…`). Required.
    pub agent_id: &'a str,
    /// The approved tester's number. Required.
    pub from: &'a str,
    /// TEXT, SUGGESTION, LOCATION, FILE or UNSUBSCRIBE; sent as `type`. Required.
    pub event_type: &'a str,
    /// The text; required for `TEXT`.
    pub text: Option<&'a str>,
    /// The tapped suggestion's postback; `SUGGESTION` needs this or `button_id`.
    pub postback_data: Option<&'a str>,
    /// The message a tap answers; the latest message delivered to the tester by default.
    pub message_id: Option<&'a str>,
    /// The suggestion's position, as webhooks report it.
    pub button_id: Option<&'a str>,
    /// Required for `LOCATION`.
    pub latitude: Option<f64>,
    /// Required for `LOCATION`.
    pub longitude: Option<f64>,
    /// The file's name, for `FILE`.
    pub file_name: Option<&'a str>,
}

/// Filters for [`RcsMessagesDomain::list`].
#[derive(Debug, Default)]
pub struct ListRcsMessagesParams<'a> {
    /// Messages this agent sent; sent as `registrationId`.
    pub agent_id: Option<&'a str>,
    /// ISO-8601 date or timestamp; sent as `startDate`. Required.
    pub from: &'a str,
    /// ISO-8601 date or timestamp; sent as `endDate`. Required.
    pub to: &'a str,
    /// The group whose messages to list. Required.
    pub group_id: &'a str,
    /// `ENQUEUED`, `DEQUEUED`, `SENT`, `FAILED`, `DELIVERED`, `READ`, `EXPIRED`, `FALLBACK` or
    /// `BLOCKED`.
    pub status: Option<&'a str>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub template_id: Option<&'a str>,
    pub template_version: Option<u32>,
    /// The messages of one send.
    pub bulk_id: Option<&'a str>,
    /// `NOT_CAPABLE`, `CARRIER_NOT_LAUNCHED`, `TTL_EXPIRED`, `RCS_ERROR`, `PROVIDER_ERROR` or
    /// `SMS_ONLY`.
    pub fallback_reason: Option<&'a str>,
    /// `RICH` or `RICH_MEDIA`.
    pub billing_class: Option<&'a str>,
    /// The channel that delivered the message: `rcs`, `sms` or `mms`.
    pub delivered_channel: Option<&'a str>,
    /// `true` for test messages only, `false` to leave them out.
    pub is_test: Option<bool>,
}

/// RCS domain -- RCS (Google RBM) resources. Templates for RCS live under `client.templates`.
///
/// Sub-resources:
/// * `client.rcs.agents` — RCS agents under `/rcs/agent`.
/// * `client.rcs.messages` — RCS messages: send, validate, estimate, capabilities, get and list.
pub struct RcsDomain {
    pub agents: RcsAgentsDomain,
    pub messages: RcsMessagesDomain,
}

impl RcsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self {
            agents: RcsAgentsDomain::new(http.clone()),
            messages: RcsMessagesDomain::new(http),
        }
    }
}

/// RCS agents under `/rcs/agent`. An agent links one brand and one campaign, of any registration
/// type: the campaign's numbers are the SMS fallback, and the brand and campaign supply the company
/// identity and consent text for the carrier launch. One brand or campaign can serve many agents.
/// An agent is a registration (type `RCS_AGENT`, id `RCSA…`), so the registrations methods read and
/// edit the same record. 404 while the environment's RCS switch is off; 400 while the group's
/// `rcsEnabled` flag is off.
pub struct RcsAgentsDomain {
    http: Arc<HttpClient>,
}

impl RcsAgentsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Create an agent draft on a brand, optionally with its campaign (HTTP 201, status
    /// `DRAFT`). The agent's subgroup is the brand's. Set `RequestOptions.idempotency_key` to make
    /// a retry safe.
    ///
    /// **POST** `/rcs/agent`
    pub async fn create(
        &self,
        params: CreateRcsAgentParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = create_body(&params)?;
        self.http
            .request_json("/rcs/agent", Method::POST, Some(&payload), options)
            .await
    }

    /// List agents, newest first, as `{ data, total, page, limit }`.
    ///
    /// **GET** `/rcs/agent`
    pub async fn list(
        &self,
        params: ListRcsAgentsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("status", params.status.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("page", params.page.map(|n| n.to_string())),
            ("limit", params.limit.map(|n| n.to_string())),
        ]);
        self.http
            .request_json(&format!("/rcs/agent{}", qs), Method::GET, None, options)
            .await
    }

    /// Get one agent with the live state of its brand and campaign links. `googleAgentId` is
    /// Google's id for the agent, `null` until Google has created it.
    ///
    /// **GET** `/rcs/agent/:agentId`
    pub async fn get(
        &self,
        agent_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let path = agent_path(agent_id)?;
        self.http
            .request_json(&path, Method::GET, None, options)
            .await
    }

    /// Edit an agent while it may change: a draft, a Signal House rejection, a request for changes,
    /// or an agent in testing (the change is pushed to the carrier partner). `agent`, `consent` and
    /// `contact` merge field by field; every other key replaces. 409 in any other status; `brand_id`
    /// is locked once the agent exists at the partner and `billing_category` once a launch was requested.
    /// After a carrier rejected a launch other carriers took (`PARTIALLY_LAUNCHED` with a `REJECTED`
    /// carrier), only `consent` and `contact` may change, and nothing is pushed: send them with
    /// `request_carrier_update`.
    ///
    /// **PUT** `/rcs/agent/:agentId`
    pub async fn update(
        &self,
        params: UpdateRcsAgentParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let path = agent_path(params.agent_id)?;
        let payload = update_body(&params);
        self.http
            .request_json(&path, Method::PUT, Some(&payload), options)
            .await
    }

    /// Check an agent against every submit requirement without writing; returns `{ valid,
    /// errors: [{ field, reason, code? }], warnings: [{ field, reason }] }`. A copy-rule error at the
    /// launch gate carries the carrier partner's key as `code` (for example
    /// `OPT_IN_MSG_RATE_DISCL_MISSING`).
    ///
    /// **POST** `/rcs/agent/:agentId/validate`
    pub async fn validate(
        &self,
        agent_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let path = format!("{}/validate", agent_path(agent_id)?);
        self.http
            .request_json(&path, Method::POST, None, options)
            .await
    }

    /// Submit a draft: create the agent at the carrier partner, with no review first (status
    /// `AGENT_CREATING`, then `IN_TESTING` once the partner reports it ready). It needs the full
    /// profile, a use case and a billing category; otherwise a 400 whose details carry
    /// `{ code: "REGISTRATION_INCOMPLETE", errors, warnings }`.
    /// A group that has not finished account verification gets a 403 with code `GROUP_UNVERIFIED`,
    /// and a rejected one a 403 with code `GROUP_REJECTED`; a pending verification passes.
    ///
    /// **POST** `/rcs/agent/:agentId/submit`
    pub async fn submit(
        &self,
        agent_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let path = format!("{}/submit", agent_path(agent_id)?);
        self.http
            .request_json(&path, Method::POST, None, options)
            .await
    }

    /// Request the agent's launch: the launch requirements are checked and the agent goes to Signal
    /// House review (status `PENDING_REVIEW`), whose approval files the launch with the carrier
    /// partner. Allowed from `IN_TESTING`, `REJECTED` and `CARRIER_REJECTED` (409 otherwise); a
    /// failed requirement is a 400 whose details carry `{ code: "REGISTRATION_INCOMPLETE", errors,
    /// warnings }`, and a copy-rule error carries the carrier partner's key as its `code` (for
    /// example `OPT_IN_MSG_RATE_DISCL_MISSING`). A group that has not finished account verification
    /// gets a 403 with code `GROUP_UNVERIFIED`, and a rejected one a 403 with code `GROUP_REJECTED`.
    /// The agent is a registration, so this is the registration's launch route.
    /// The request charges the agent's launch fees: a wallet that cannot cover them gets a 402, and the agent stays where it is. When any fee is due the request must set the acknowledgement (see `launch_acknowledging_fees`), or it gets a 400 with code FEE_ACKNOWLEDGEMENT_REQUIRED and nothing is held; a launch with nothing due needs no acknowledgement. The agent's launchFees quotes what the next request charges.
    /// When the agent's brand vertical is healthcare or insurance, the launch needs the protected health information (PHI) disclosure acknowledged: use `launch_acknowledging` with `phi` set, or the request gets a 400 REGISTRATION_INCOMPLETE whose errors name `consent.phiAcknowledgement`. The API records the acknowledgement as `consent.phiAcknowledgement { acknowledgedAt, acknowledgedBy }` from the caller; the client sends only the boolean, and an agent update cannot set it. Once recorded, a later launch request needs no new acknowledgement.
    ///
    /// **POST** `/registration/:agentId/launch`
    pub async fn launch(
        &self,
        agent_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentId", Some(agent_id))])?;
        let path = format!("/registration/{}/launch", urlencoding::encode(agent_id));
        self.http
            .request_json(&path, Method::POST, None, options)
            .await
    }

    /// `launch`, accepting the fees due. Required when any fee is due.
    ///
    /// **POST** `/registration/:agentId/launch`
    pub async fn launch_acknowledging_fees(
        &self,
        agent_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentId", Some(agent_id))])?;
        let path = format!("/registration/{}/launch", urlencoding::encode(agent_id));
        let payload = json!({ "acknowledgeFees": true });
        self.http
            .request_json(&path, Method::POST, Some(&payload), options)
            .await
    }

    /// Send Signal House the corrected launch answers after a carrier rejected a launch other
    /// carriers took. The carrier partner takes no change through its API once a carrier launched,
    /// so Signal House emails it the update; edit `consent` and `contact` first. The agent's
    /// `carrierUpdate` reads `REQUESTED` for the rejected carriers, then `SENT` once Signal House
    /// sent it. 409 when no carrier is `REJECTED` after a partial launch, or an update already
    /// waits. Returns the agent's registration, with `data.carrierUpdate`.
    ///
    /// **POST** `/registration/:agentId/carrier-update`
    pub async fn request_carrier_update(
        &self,
        agent_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentId", Some(agent_id))])?;
        let path = format!(
            "/registration/{}/carrier-update",
            urlencoding::encode(agent_id)
        );
        self.http
            .request_json(&path, Method::POST, None, options)
            .await
    }

    /// `launch`, with what the request accepts: the fees due, and the PHI disclosure.
    /// When the agent's brand vertical is healthcare or insurance, the launch needs the protected health information (PHI) disclosure acknowledged: set `phi`, or the request gets a 400 REGISTRATION_INCOMPLETE whose errors name `consent.phiAcknowledgement`. The API records the acknowledgement as `consent.phiAcknowledgement { acknowledgedAt, acknowledgedBy }` from the caller; the client sends only the boolean, and an agent update cannot set it. Once recorded, a later launch request needs no new acknowledgement.
    ///
    /// **POST** `/registration/:agentId/launch`
    pub async fn launch_acknowledging(
        &self,
        agent_id: &str,
        acknowledgements: LaunchAcknowledgements,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentId", Some(agent_id))])?;
        let path = format!("/registration/{}/launch", urlencoding::encode(agent_id));
        let payload = acknowledgements.body();
        self.http
            .request_json(&path, Method::POST, payload.as_ref(), options)
            .await
    }

    /// Archive an agent (status `ARCHIVE_PENDING` or `ARCHIVED`). One the carrier partner created stops
    /// sending at once and stays `ARCHIVE_PENDING` until Signal House confirms the partner cancelled it,
    /// then `ARCHIVED`. Archiving cannot be undone; an agent already `ARCHIVE_PENDING` answers 409.
    ///
    /// **POST** `/rcs/agent/:agentId/archive`
    pub async fn archive(
        &self,
        agent_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let path = format!("{}/archive", agent_path(agent_id)?);
        self.http
            .request_json(&path, Method::POST, None, options)
            .await
    }

    /// Make the links that start a chat with the agent and return `{ googleAgentId, deepLink,
    /// shortLink }`: an `sms:` deep link in Google's format, and a short link that redirects to it,
    /// for a QR code. With the agent's `defaultFallback.from`, the deep link names that number, so a
    /// handset that cannot reach the agent texts it. `body` (max 1,000 characters) is the message
    /// the chat opens with. The same request returns the same short link. The links reach approved
    /// testers until a carrier launches the agent. 409 with code `GOOGLE_AGENT_PENDING` before
    /// Google has created the agent.
    ///
    /// **POST** `/rcs/agent/:agentId/entry-points`
    pub async fn create_entry_points(
        &self,
        agent_id: &str,
        body: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let path = format!("{}/entry-points", agent_path(agent_id)?);
        let mut payload = Map::new();
        set(&mut payload, "body", body);
        self.http
            .request_json(&path, Method::POST, Some(&Value::Object(payload)), options)
            .await
    }

    /// Resend a tester's expired invitation, which starts a new 24 hours for the tester to accept
    /// it. The number must be a tester on the agent (404) and its invitation must be `EXPIRED`
    /// (409); up to 20 invitations go out a day (429). `phone_number` is a US number, with or
    /// without the `+`.
    ///
    /// **POST** `/rcs/agent/:agentId/testers/:phoneNumber/resend`
    pub async fn resend_tester_invite(
        &self,
        agent_id: &str,
        phone_number: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;
        let path = format!(
            "{}/testers/{}/resend",
            agent_path(agent_id)?,
            urlencoding::encode(phone_number)
        );
        self.http
            .request_json(&path, Method::POST, None, options)
            .await
    }

    /// Remove a tester at once, at any point before the agent is archived. The number must be a
    /// tester on the agent (404); an archived agent answers 409. `phone_number` is a US number,
    /// with or without the `+`.
    ///
    /// **DELETE** `/rcs/agent/:agentId/testers/:phoneNumber`
    pub async fn remove_tester(
        &self,
        agent_id: &str,
        phone_number: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;
        let path = format!(
            "{}/testers/{}",
            agent_path(agent_id)?,
            urlencoding::encode(phone_number)
        );
        self.http
            .request_json(&path, Method::DELETE, None, options)
            .await
    }

    /// Host the agent's logo (224 × 224 px, 50 KB max) or hero image (1440 × 448 px, 200 KB max),
    /// PNG or JPEG, and set it on the draft (multipart `kind` + `file`). `kind` is `"logo"` or
    /// `"hero"`. Images of any other size are refused.
    ///
    /// **POST** `/rcs/agent/:agentId/asset`
    pub async fn upload_asset(
        &self,
        agent_id: &str,
        kind: &str,
        file: Vec<u8>,
        file_name: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let path = format!("{}/asset", agent_path(agent_id)?);
        let form = asset_form(kind, file, file_name)?;
        self.http
            .request_multipart(&path, Method::POST, form, options)
            .await
    }
}

/// RCS messages. Sends go through an agent to `POST /message/rcs`. A message is a Message record
/// (`messageType` `RCS`), so `client.messages` reads the same record. 404 while the environment's
/// RCS switch is off; 403 while the group's `rcsEnabled` flag is off; 409 while the agent cannot
/// send yet.
pub struct RcsMessagesDomain {
    http: Arc<HttpClient>,
}

impl RcsMessagesDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Send an RCS message to up to 100 recipients (HTTP 201) and return `{ requested, enqueued,
    /// failed, bulkId, messages }`. A check that blocks one recipient writes a
    /// `BLOCKED` message for it and the rest still send; its row carries `blockReason`, `reason`
    /// and `issues`. An archived template answers 409 `TEMPLATE_ARCHIVED`. A recipient from
    /// 19999990000 to 19999990359 makes the whole request a simulation, as on SMS: it returns
    /// `{ simulated: true, webhooks }` and nothing is stored, sent or charged. Set
    /// `RequestOptions.idempotency_key` to make a retry safe.
    ///
    /// **POST** `/message/rcs`
    pub async fn send(
        &self,
        params: SendRcsMessageParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = send_body(&params)?;
        self.http
            .request_json("/message/rcs", Method::POST, Some(&payload), options)
            .await
    }

    /// Send an RCS message to up to 10,000 recipients in the background (HTTP 202) and return
    /// `{ bulkId, status: "ACCEPTED", recipientCount }`. The request runs `send`'s request checks,
    /// splits the recipients into chunks of 100 and queues each chunk. Every message carries the
    /// shared `bulkId`; track progress with the messages list filtered by `bulkId`
    /// (`GET /message?bulkId=`) and per-message webhooks. A chunk the wallet cannot cover, or that
    /// is refused after acceptance, gets `BLOCKED` messages with `blockReason`
    /// `INSUFFICIENT_FUNDS` or `SEND_REFUSED`; a chunk that keeps failing gets `SEND_FAILED`. A
    /// recipient from 19999990000 to 19999990359 makes the whole request a simulation, as on
    /// `send`. Set `RequestOptions.idempotency_key` to make a retry safe (same `bulkId`, no
    /// duplicate sends); a retry without one may send duplicates.
    ///
    /// **POST** `/message/rcs/batch`
    pub async fn send_batch(
        &self,
        params: SendRcsMessageParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = send_body(&params)?;
        self.http
            .request_json("/message/rcs/batch", Method::POST, Some(&payload), options)
            .await
    }

    /// Run a send's checks without writing anything or holding funds. Returns `{ valid, errors,
    /// warnings, billingClass, segments, estimatedCostMicros, unsupportedFiles, recipients }`.
    /// `valid` is true only with no errors; each blocked recipient adds an error and carries
    /// `blockReason`, `reason` and `issues`.
    ///
    /// **POST** `/message/rcs/validate`
    pub async fn validate(
        &self,
        params: SendRcsMessageParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = send_body(&params)?;
        self.http
            .request_json(
                "/message/rcs/validate",
                Method::POST,
                Some(&payload),
                options,
            )
            .await
    }

    /// Price an RCS draft in microdollars and return `{ billingClass, segments, recipientCount,
    /// rcsLeg, fallbackLeg, totals, balance, issues, warnings }`, `issues` listing what would block the
    /// send and `warnings` what an iPhone draws differently;
    /// no charge or send. `channel` `rcs` is added to the
    /// body. The fallback's `from` is optional for an estimate: with none, the fallback leg is
    /// priced on the channel of the agent's linked campaign. `rcsLeg.mediaSurcharge` is the
    /// carrier's surcharge on media over 5 MB for one recipient, and `rcsLeg.cost` already holds it for every recipient; `rcsLeg.cost` is null with the fallback mode sms_only; a message over 100 MB
    /// adds a `MEDIA_TOO_LARGE` issue.
    ///
    /// **POST** `/message/estimate`
    pub async fn estimate(
        &self,
        params: EstimateRcsMessageParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = estimate_body(&params)?;
        self.http
            .request_json("/message/estimate", Method::POST, Some(&payload), options)
            .await
    }

    /// Report which numbers (up to 500, 10-15 digits, with or without the `+`) can receive RCS from
    /// an agent as `{ results: [{ phoneNumber, capability, rcsEnabled, reason, features, carrier, checkedAt, cached }] }`,
    /// capability being ENABLED, UNREACHABLE or UNKNOWN, and reason CAPABLE, NOT_CAPABLE, NOT_LAUNCHED_ON_CARRIER or UNKNOWN; rcsEnabled is true only for CAPABLE, carrier and checkedAt may be null, and features is empty today.
    ///
    /// **POST** `/rcs/capabilities`
    pub async fn capabilities(
        &self,
        agent_id: &str,
        phone_numbers: &[&str],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentId", Some(agent_id))])?;
        if phone_numbers.is_empty() {
            return Err(SignalHouseError::Validation("phoneNumbers".to_string()));
        }
        let payload = json!({ "agentId": agent_id, "phoneNumbers": phone_numbers });
        self.http
            .request_json("/rcs/capabilities", Method::POST, Some(&payload), options)
            .await
    }

    /// Fake an inbound event from an approved tester of the agent, with no carrier involved: it runs the real inbound handlers and webhooks, as free test traffic. A simulated STOP really opts the tester out; a simulated START clears it. Anyone else gets a 403 with code NOT_TESTER. Returns 202 with { status, messageId }.
    ///
    /// **POST** `/message/rcs/simulate-inbound`
    pub async fn simulate_inbound(
        &self,
        params: SimulateRcsInboundParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentId", Some(params.agent_id)),
            ("from", Some(params.from)),
            ("type", Some(params.event_type)),
        ])?;
        let mut body = Map::new();
        set(&mut body, "agentId", Some(params.agent_id));
        set(&mut body, "from", Some(params.from));
        set(&mut body, "type", Some(params.event_type));
        set(&mut body, "text", params.text);
        set(&mut body, "postbackData", params.postback_data);
        set(&mut body, "messageId", params.message_id);
        set(&mut body, "buttonId", params.button_id);
        set(&mut body, "latitude", params.latitude);
        set(&mut body, "longitude", params.longitude);
        set(&mut body, "fileName", params.file_name);
        self.http
            .request_json(
                "/message/rcs/simulate-inbound",
                Method::POST,
                Some(&Value::Object(body)),
                options,
            )
            .await
    }

    /// Get one message by its ID; an unknown ID is a 404. The same route as
    /// `client.messages.get_message`. A settled RCS message carries `carrierFeeMicros`, the carrier
    /// pass-through and media surcharge it was charged.
    ///
    /// **GET** `/message/:messageId`
    pub async fn get(
        &self,
        message_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("messageId", Some(message_id))])?;
        let path = format!("/message/{}", urlencoding::encode(message_id));
        self.http
            .request_json(&path, Method::GET, None, options)
            .await
    }

    /// List RCS messages. `messageType=RCS` is added to the query. A fallback leg is an SMS or MMS
    /// message: read it with `client.messages.get_message` and the parent's `fallbackMessageId`.
    ///
    /// **GET** `/message`
    pub async fn list(
        &self,
        params: ListRcsMessagesParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(params.group_id)),
            ("from", Some(params.from)),
            ("to", Some(params.to)),
        ])?;
        let qs = HttpClient::get_query_string(&[
            ("registrationId", params.agent_id.map(String::from)),
            ("startDate", Some(params.from.to_string())),
            ("endDate", Some(params.to.to_string())),
            ("groupId", Some(params.group_id.to_string())),
            ("status", params.status.map(String::from)),
            ("page", params.page.map(|n| n.to_string())),
            ("limit", params.limit.map(|n| n.to_string())),
            ("templateId", params.template_id.map(String::from)),
            (
                "templateVersion",
                params.template_version.map(|n| n.to_string()),
            ),
            ("bulkId", params.bulk_id.map(String::from)),
            ("fallbackReason", params.fallback_reason.map(String::from)),
            ("billingClass", params.billing_class.map(String::from)),
            (
                "deliveredChannel",
                params.delivered_channel.map(String::from),
            ),
            ("isTest", params.is_test.map(|flag| flag.to_string())),
            ("messageType", Some("RCS".to_string())),
            ("dateBounds", Some("exact".to_string())),
        ]);
        self.http
            .request_json(&format!("/message{}", qs), Method::GET, None, options)
            .await
    }
}

// -- helpers ------------------------------------------------------------------

/// Build the encoded path of one agent.
fn agent_path(agent_id: &str) -> Result<String, SignalHouseError> {
    HttpClient::require(&[("agentId", Some(agent_id))])?;
    Ok(format!("/rcs/agent/{}", urlencoding::encode(agent_id)))
}

/// Set `key` on the body when `value` is present.
fn set<T: serde::Serialize>(body: &mut Map<String, Value>, key: &str, value: Option<T>) {
    if let Some(value) = value {
        body.insert(key.to_string(), json!(value));
    }
}

/// Build a create body; unset fields are dropped.
fn create_body(params: &CreateRcsAgentParams<'_>) -> Result<Value, SignalHouseError> {
    HttpClient::require(&[("brandId", Some(params.brand_id))])?;
    let mut body = Map::new();
    set(&mut body, "brandId", Some(params.brand_id));
    set(&mut body, "campaignId", params.campaign_id);
    set(&mut body, "agent", params.agent);
    set(&mut body, "consent", params.consent);
    set(&mut body, "contact", params.contact);
    set(&mut body, "useCase", params.use_case);
    set(&mut body, "billingCategory", params.billing_category);
    set(&mut body, "defaultFallback", params.default_fallback);
    set(&mut body, "excludedCarriers", params.excluded_carriers);
    set(&mut body, "testers", params.testers);
    set(&mut body, "autofill", params.autofill);
    Ok(Value::Object(body))
}

/// Build an update body; unset fields are dropped and `Some(None)` on `campaign_id`, `use_case` or
/// `default_fallback` sends `null`.
fn update_body(params: &UpdateRcsAgentParams<'_>) -> Value {
    let mut body = Map::new();
    set(&mut body, "brandId", params.brand_id);
    set(&mut body, "campaignId", params.campaign_id);
    set(&mut body, "agent", params.agent);
    set(&mut body, "consent", params.consent);
    set(&mut body, "contact", params.contact);
    set(&mut body, "useCase", params.use_case);
    set(&mut body, "billingCategory", params.billing_category);
    set(&mut body, "defaultFallback", params.default_fallback);
    set(&mut body, "excludedCarriers", params.excluded_carriers);
    set(&mut body, "testers", params.testers);
    set(&mut body, "autofill", params.autofill);
    Value::Object(body)
}

/// Validate a send and build its body; unset fields are dropped.
fn send_body(params: &SendRcsMessageParams<'_>) -> Result<Value, SignalHouseError> {
    HttpClient::require(&[("agentId", Some(params.agent_id))])?;
    if params.recipients.is_empty() {
        return Err(SignalHouseError::Validation("recipients".to_string()));
    }
    let mut body = Map::new();
    set(&mut body, "agentId", Some(params.agent_id));
    set(&mut body, "recipients", Some(params.recipients));
    set(&mut body, "templateId", params.template_id);
    set(&mut body, "templateVersion", params.template_version);
    set(&mut body, "content", params.content);
    set(&mut body, "contentFormat", params.content_format);
    set(&mut body, "trafficType", params.traffic_type);
    set(&mut body, "ttlSeconds", params.ttl_seconds);
    set(&mut body, "expireTime", params.expire_time);
    set(&mut body, "fallback", params.fallback);
    set(&mut body, "statusCallbackUrl", params.status_callback_url);
    set(&mut body, "enableShortlink", params.enable_shortlink);
    set(&mut body, "isTest", params.is_test);
    set(&mut body, "simulate", params.simulate);
    Ok(Value::Object(body))
}

/// Validate an estimate and build its body with `channel` `rcs`; unset fields are dropped.
fn estimate_body(params: &EstimateRcsMessageParams<'_>) -> Result<Value, SignalHouseError> {
    HttpClient::require(&[("agentId", Some(params.agent_id))])?;
    let mut body = Map::new();
    set(&mut body, "channel", Some("rcs"));
    set(&mut body, "agentId", Some(params.agent_id));
    set(&mut body, "recipients", params.recipients);
    set(&mut body, "recipientCount", params.recipient_count);
    set(&mut body, "templateId", params.template_id);
    set(&mut body, "templateVersion", params.template_version);
    set(&mut body, "content", params.content);
    set(&mut body, "contentFormat", params.content_format);
    set(&mut body, "trafficType", params.traffic_type);
    set(&mut body, "ttlSeconds", params.ttl_seconds);
    set(&mut body, "expireTime", params.expire_time);
    set(&mut body, "fallback", params.fallback);
    Ok(Value::Object(body))
}

/// Build the multipart `kind` + `file` upload form. A missing file name is sent as `"file"` so the
/// server reads the part as a file.
fn asset_form(
    kind: &str,
    file: Vec<u8>,
    file_name: Option<&str>,
) -> Result<Form, SignalHouseError> {
    HttpClient::require(&[("kind", Some(kind))])?;
    if file.is_empty() {
        return Err(SignalHouseError::Validation("file".to_string()));
    }
    let file_name = file_name.filter(|name| !name.is_empty()).unwrap_or("file");
    Ok(Form::new()
        .text("kind", kind.to_string())
        .part("file", Part::bytes(file).file_name(file_name.to_string())))
}
