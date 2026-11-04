use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde_json::{json, Map, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Parameters for [`RcsAgentsDomain::create`]. Unset fields are left out of the body.
#[derive(Debug, Default)]
pub struct CreateRcsAgentParams<'a> {
    /// The 10DLC brand: carrier brand ID, provisional UUID or `_id`. Required.
    pub brand_id: &'a str,
    /// A 10DLC campaign of that brand (campaign ID or `_id`); required before submit.
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
    /// `NON_CONVERSATIONAL` (default) or `CONVERSATIONAL`; submit refuses `CONVERSATIONAL`.
    pub billing_category: Option<&'a str>,
    /// `{ mode (auto, rcs_only, sms_only, none), channel (auto, sms, mms), from }`, where `from` is
    /// a number on the linked campaign.
    pub default_fallback: Option<&'a Value>,
    /// `[]` or `["T_MOBILE"]` (the carrier partner's carrier name).
    pub excluded_carriers: Option<&'a [&'a str]>,
    /// Tester numbers (up to 200), invited once the agent exists at the carrier.
    pub testers: Option<&'a [&'a str]>,
}

/// Filters for [`RcsAgentsDomain::list`].
#[derive(Debug, Default)]
pub struct ListRcsAgentsParams<'a> {
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    /// One status or a comma-separated list from `DRAFT`, `PENDING_REVIEW`, `REJECTED`,
    /// `AGENT_CREATING` and `ARCHIVED`.
    pub status: Option<&'a str>,
    /// Agents linked to this 10DLC brand (carrier brand ID, provisional UUID or `_id`).
    pub brand_id: Option<&'a str>,
    /// Agents linked to this 10DLC campaign (campaign ID or `_id`).
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
    /// Relink to another 10DLC brand of the same subgroup.
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
}

/// RCS domain -- RCS (Google RBM) resources. Templates for RCS live under `client.templates`.
///
/// Sub-resources:
/// * `client.rcs.agents` — RCS agents under `/rcs/agent`.
pub struct RcsDomain {
    pub agents: RcsAgentsDomain,
}

impl RcsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self {
            agents: RcsAgentsDomain::new(http),
        }
    }
}

/// RCS agents under `/rcs/agent`. An agent links one 10DLC brand and one 10DLC campaign: the
/// campaign's numbers are the SMS fallback, and the brand and campaign supply the company identity
/// and consent text for the carrier launch. One brand or campaign can serve many agents. An agent
/// is a registration (type `RCS_AGENT`, id `RCSA…`), so the registrations methods read and edit
/// the same record. 404 while the environment's RCS switch is off; 400 while the group's
/// `rcsEnabled` flag is off.
pub struct RcsAgentsDomain {
    http: Arc<HttpClient>,
}

impl RcsAgentsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Create an agent draft on a 10DLC brand, optionally with its campaign (HTTP 201, status
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

    /// Get one agent with the live state of its brand and campaign links.
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
    /// errors: [{ field, reason }], warnings: [{ field, reason }] }`.
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
    /// profile, a use case and `NON_CONVERSATIONAL` billing; otherwise a 400 whose details carry
    /// `{ code: "REGISTRATION_INCOMPLETE", errors, warnings }`.
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
    /// warnings }`. The agent is a registration, so this is the registration's launch route.
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

    /// Archive an agent (status `ARCHIVED`).
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
    Value::Object(body)
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
