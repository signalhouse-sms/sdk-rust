use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

#[cfg(feature = "admin")]
use serde_json::json;

/// What a launch request accepts, for [`RegistrationsDomain::launch_registration_acknowledging`] and
/// `RcsAgentsDomain::launch_acknowledging`. Each `true` field is sent; neither sends no body.
#[derive(Debug, Default, Clone, Copy)]
pub struct LaunchAcknowledgements {
    /// Accept the fees due; required when any are.
    pub fees: bool,
    /// Accept the protected health information (PHI) disclosure; required for a healthcare or
    /// insurance brand.
    pub phi: bool,
}

impl LaunchAcknowledgements {
    /// The launch request body: the accepted acknowledgements, or `None` when there are none.
    pub(crate) fn body(self) -> Option<Value> {
        let mut body = serde_json::Map::new();
        if self.fees {
            body.insert("acknowledgeFees".into(), Value::Bool(true));
        }
        if self.phi {
            body.insert("acknowledgePhi".into(), Value::Bool(true));
        }
        (!body.is_empty()).then_some(Value::Object(body))
    }
}

/// Optional filters for [`RegistrationsDomain::get_registrations`].
#[derive(Debug, Default)]
pub struct GetRegistrationsParams<'a> {
    /// Scopes the list; staff may omit it for the review queue.
    pub group_id: Option<&'a str>,
    pub subgroup_id: Option<&'a str>,
    /// One jurisdiction or a comma-separated list, e.g. `"GB,CA"` (US, GB, CA, AU; any case of
    /// `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB).
    pub region: Option<&'a str>,
    /// The registration type, e.g. `"VIRTUAL_LONG_CODE"`.
    pub type_: Option<&'a str>,
    /// One of `identity`, `program` or `sender`.
    pub kind: Option<&'a str>,
    /// One registration status or a comma-separated list, e.g. `"PENDING_PROVIDER,APPROVED"`,
    /// from DRAFT, SIGNAL_HOUSE_REVIEW, SIGNAL_HOUSE_APPROVED, SIGNAL_HOUSE_REJECTED, SUBMITTING,
    /// PENDING_PROVIDER, NEEDS_INFORMATION, PAYMENT_REQUIRED, APPROVED, REJECTED, CANCELLED,
    /// PENDING_DELETE, DELETED, EXPIRED or RECONCILIATION_REQUIRED.
    pub status: Option<&'a str>,
    /// The parent registration (e.g. a program's identity).
    pub parent_registration_id: Option<&'a str>,
    /// Filter to registrations holding this phone number.
    pub phone_number: Option<&'a str>,
    /// RCS agents only: agents linked to this brand (carrier ID, provisional UUID or `_id`).
    pub brand_id: Option<&'a str>,
    /// RCS agents only: agents linked to this campaign (campaign ID or `_id`).
    pub campaign_id: Option<&'a str>,
    /// One of `createdAt` (default), `subgroupId`, `registrationId`, `region` or `status`.
    pub sort_by: Option<&'a str>,
    /// `asc` or `desc` (default).
    pub sort_order: Option<&'a str>,
    pub page: Option<u32>,
    /// At most 100.
    pub limit: Option<u32>,
}

/// Registrations domain -- the catalog, quotes, submission and status tracking for every
/// jurisdiction. Staff review methods require the `admin` crate feature; enabling it grants no
/// server permission.
pub struct RegistrationsDomain {
    http: Arc<HttpClient>,
}

impl RegistrationsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get the registration types a jurisdiction offers and whether each can be acquired right now.
    ///
    /// **GET** `/registration/catalog`
    ///
    /// `region` is the jurisdiction (US, GB, CA, AU; any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB). The data is
    /// `{ region, jurisdiction, sendingRequiresRegistration, registeredThrough, provisioning, types: [{ type, kind,
    /// channel, bundles, quantity: { min, max }, requiresParent, available }] }`.
    pub async fn get_catalog(
        &self,
        region: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("region", Some(region))])?;
        let qs = HttpClient::get_query_string(&[("region", Some(region.to_string()))]);
        self.http
            .request_json(
                &format!("/registration/catalog{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get setup and monthly prices, in integer microdollars, for each capability bundle of a type.
    ///
    /// **GET** `/registration/quotes`
    ///
    /// `region` is the jurisdiction (any case of `"UK"`/`"GB"`/`"GBR"`/`"United Kingdom"` is accepted for GB), `type_` the registration type
    /// (e.g. `"VIRTUAL_LONG_CODE"`, or `"ALPHANUMERIC_SENDER_ID"` for GB: one `["SMS"]` bundle, 6000000
    /// setup and 6000000 monthly by default) and `quantity` how many senders the order covers (1-100).
    /// The data is one quote per bundle: `capabilities`, `quantity`, `currency`, `setupAmount`,
    /// `monthlyAmount`, `totalSetupAmount`, `totalMonthlyAmount`, `etaDays`.
    /// `setupAmount`/`monthlyAmount` are per sender; the totals cover the quantity. `group_id` is
    /// staff-only and quotes on behalf of another group.
    pub async fn get_quotes(
        &self,
        region: &str,
        type_: &str,
        quantity: u32,
        group_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("region", Some(region)), ("type", Some(type_))])?;
        if quantity == 0 {
            return Err(SignalHouseError::Validation("quantity".to_string()));
        }
        let qs = HttpClient::get_query_string(&[
            ("region", Some(region.to_string())),
            ("type", Some(type_.to_string())),
            ("quantity", Some(quantity.to_string())),
            ("groupId", group_id.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/registration/quotes{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// List registrations with optional filters and pagination.
    ///
    /// **GET** `/registration`
    ///
    /// The response carries `data`, `totalCount`, `page` and `limit`.
    pub async fn get_registrations(
        &self,
        params: GetRegistrationsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("region", params.region.map(String::from)),
            ("type", params.type_.map(String::from)),
            ("kind", params.kind.map(String::from)),
            ("status", params.status.map(String::from)),
            (
                "parentRegistrationId",
                params.parent_registration_id.map(String::from),
            ),
            ("phoneNumber", params.phone_number.map(String::from)),
            ("brandId", params.brand_id.map(String::from)),
            ("campaignId", params.campaign_id.map(String::from)),
            ("sortBy", params.sort_by.map(String::from)),
            ("sortOrder", params.sort_order.map(String::from)),
            ("page", params.page.map(|n| n.to_string())),
            ("limit", params.limit.map(|n| n.to_string())),
        ]);
        self.http
            .request_json(&format!("/registration{}", qs), Method::GET, None, options)
            .await
    }

    /// Get a single registration with its type data, review outcome and status history.
    ///
    /// **GET** `/registration/:registrationId`
    pub async fn get_registration(
        &self,
        registration_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        let safe = urlencoding::encode(registration_id);
        self.http
            .request_json(
                &format!("/registration/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Edit a draft, or a registration Signal House rejected, before submitting it (types that keep
    /// drafts, such as `RCS_AGENT`).
    ///
    /// **PUT** `/registration/:registrationId`
    ///
    /// `data` is a partial of the type's create fields, sent as the body: for `RCS_AGENT` `agent`
    /// and `consent` merge field by field, every other key replaces, and `campaignId: null` unlinks
    /// the campaign. 409 once submitted.
    pub async fn update_registration(
        &self,
        registration_id: &str,
        data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        if data.is_null() {
            return Err(SignalHouseError::Validation("data".to_string()));
        }
        let safe = urlencoding::encode(registration_id);
        self.http
            .request_json(
                &format!("/registration/{}", safe),
                Method::PUT,
                Some(data),
                options,
            )
            .await
    }

    /// Check a draft against every submit requirement and the live state of its links, without
    /// writing; returns `{ valid, errors: [{ field, reason }], warnings: [{ field, reason }] }`.
    ///
    /// **POST** `/registration/:registrationId/validate`
    pub async fn validate_registration(
        &self,
        registration_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        let safe = urlencoding::encode(registration_id);
        self.http
            .request_json(
                &format!("/registration/{}/validate", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Submit a draft; returns the registration. A type Signal House reviews goes to
    /// `SIGNAL_HOUSE_REVIEW`; a type it does not (`RCS_AGENT`) goes to `SUBMITTING` and is filed
    /// with the provider at once. A draft that fails a requirement gets a 400 whose details carry
    /// `{ code: "REGISTRATION_INCOMPLETE", errors, warnings }`; a second submit, or one of a
    /// registration the provider already holds, gets a 409.
    ///
    /// **POST** `/registration/:registrationId/submit`
    pub async fn submit_registration(
        &self,
        registration_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        let safe = urlencoding::encode(registration_id);
        self.http
            .request_json(
                &format!("/registration/{}/submit", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Request the launch of a registration the provider already holds (an RCS agent in testing):
    /// the launch requirements are checked and the registration goes to `SIGNAL_HOUSE_REVIEW`,
    /// whose approval files the provider's launch request. Allowed while the provider reports the
    /// agent ready for testing, after a Signal House rejection, or after every carrier refused an
    /// earlier launch (409 otherwise); a failed requirement is a 400 whose details carry
    /// `{ code: "REGISTRATION_INCOMPLETE", errors, warnings }`.
    /// An RCS agent's launch request charges its launch fees: a wallet that cannot cover them gets a 402, and nothing changes. When any fee is due the request must set the acknowledgement (see `launch_registration_acknowledging_fees`), or it gets a 400 with code FEE_ACKNOWLEDGEMENT_REQUIRED and nothing is held; a launch with nothing due needs no acknowledgement.
    /// When the agent's brand vertical is healthcare or insurance, the launch needs the protected health information (PHI) disclosure acknowledged: use `launch_registration_acknowledging` with `phi` set, or the request gets a 400 REGISTRATION_INCOMPLETE whose errors name `consent.phiAcknowledgement`. The API records the acknowledgement as `consent.phiAcknowledgement { acknowledgedAt, acknowledgedBy }` from the caller; the client sends only the boolean, and an agent update cannot set it. Once recorded, a later launch request needs no new acknowledgement.
    ///
    /// **POST** `/registration/:registrationId/launch`
    pub async fn launch_registration(
        &self,
        registration_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        let safe = urlencoding::encode(registration_id);
        self.http
            .request_json(
                &format!("/registration/{}/launch", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// `launch_registration`, accepting the fees due. Required when any fee is due.
    ///
    /// **POST** `/registration/:registrationId/launch`
    pub async fn launch_registration_acknowledging_fees(
        &self,
        registration_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        let safe = urlencoding::encode(registration_id);
        let payload = serde_json::json!({ "acknowledgeFees": true });
        self.http
            .request_json(
                &format!("/registration/{}/launch", safe),
                Method::POST,
                Some(&payload),
                options,
            )
            .await
    }

    /// `launch_registration`, with what the request accepts: the fees due, and the PHI disclosure.
    /// When the agent's brand vertical is healthcare or insurance, the launch needs the protected health information (PHI) disclosure acknowledged: set `phi`, or the request gets a 400 REGISTRATION_INCOMPLETE whose errors name `consent.phiAcknowledgement`. The API records the acknowledgement as `consent.phiAcknowledgement { acknowledgedAt, acknowledgedBy }` from the caller; the client sends only the boolean, and an agent update cannot set it. Once recorded, a later launch request needs no new acknowledgement.
    ///
    /// **POST** `/registration/:registrationId/launch`
    pub async fn launch_registration_acknowledging(
        &self,
        registration_id: &str,
        acknowledgements: LaunchAcknowledgements,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        let safe = urlencoding::encode(registration_id);
        let payload = acknowledgements.body();
        self.http
            .request_json(
                &format!("/registration/{}/launch", safe),
                Method::POST,
                payload.as_ref(),
                options,
            )
            .await
    }

    /// Cancel a registration. Allowed from SIGNAL_HOUSE_REVIEW, SIGNAL_HOUSE_APPROVED,
    /// PENDING_PROVIDER, NEEDS_INFORMATION, PAYMENT_REQUIRED and APPROVED (409 otherwise), and for
    /// types that keep drafts also from DRAFT and SIGNAL_HOUSE_REJECTED. Numbers already allocated
    /// stay on the account and stop sending.
    ///
    /// **DELETE** `/registration/:registrationId`
    pub async fn cancel_registration(
        &self,
        registration_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        let safe = urlencoding::encode(registration_id);
        self.http
            .request_json(
                &format!("/registration/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Submit a registration for Signal House review (HTTP 201).
    ///
    /// **POST** `/registration`
    ///
    /// `registration_data` is `{ region, type, subgroupId, capabilities
    /// (one of the type's bundles), data }`. The type selects the data shape; for
    /// `VIRTUAL_LONG_CODE` it is `{ quantity (1-100), useCases (1-10), voice? }` with `voice`
    /// required exactly when `capabilities` include `VOICE`. For GB `ALPHANUMERIC_SENDER_ID`
    /// (`capabilities` `["SMS"]`) it is `{ requestedSenderId (3-11 letters/digits/space/dot/dash,
    /// at least one letter), trafficOrigin (LOCAL | INTERNATIONAL), trafficType (TRANSACTIONAL |
    /// PROMOTIONAL), companyName, companyCountry, companyWebsite, industry, messageExample,
    /// senderRelationship? }` with `senderRelationship` required when `requestedSenderId` differs
    /// from `companyName`; quantity is always 1 and `data.phoneNumbers` carries the granted sender
    /// (e.g. `["ACME"]`) once APPROVED. Set `RequestOptions.idempotency_key` to
    /// make a retry safe.
    pub async fn create_registration(
        &self,
        registration_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/registration",
                Method::POST,
                Some(registration_data),
                options,
            )
            .await
    }

    /// Approve or reject a registration awaiting Signal House review.
    ///
    /// **POST** `/registration/:registrationId/review`
    ///
    /// `action` is `"APPROVE"` (submits the registration to the provider) or `"REJECT"`;
    /// `reason` is required for `"REJECT"`. Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn review_registration(
        &self,
        registration_id: &str,
        action: &str,
        reason: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.review_registration_with_reasons(registration_id, action, reason, None, None, options)
            .await
    }

    /// Approve or reject a registration, with the reason code and field-level reasons an RCS
    /// agent's rejection takes.
    ///
    /// **POST** `/registration/:registrationId/review`
    ///
    /// `reason_code` is the rejection catalogue code, required to reject an RCS agent and refused
    /// for any other type; the agent's `reason` is 10 to 256 characters. `rejection_reasons` is an
    /// optional array of `{ field, code, reason, recommendedAction }`. Requires the `admin` crate
    /// feature.
    #[cfg(feature = "admin")]
    pub async fn review_registration_with_reasons(
        &self,
        registration_id: &str,
        action: &str,
        reason: Option<&str>,
        reason_code: Option<&str>,
        rejection_reasons: Option<&Value>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("registrationId", Some(registration_id)),
            ("action", Some(action)),
        ])?;
        let safe = urlencoding::encode(registration_id);
        let mut body = json!({ "action": action });
        if let Some(reason) = reason {
            body["reason"] = json!(reason);
        }
        if let Some(reason_code) = reason_code {
            body["reasonCode"] = json!(reason_code);
        }
        if let Some(rejection_reasons) = rejection_reasons {
            body["rejectionReasons"] = rejection_reasons.clone();
        }
        self.http
            .request_json(
                &format!("/registration/{}/review", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Save corrections for a registration the provider sent back for more information.
    ///
    /// **POST** `/registration/:registrationId/corrections`
    ///
    /// `data` holds the type's correctable fields (`VIRTUAL_LONG_CODE`: `useCases` and, for
    /// SMS + Voice, `voice`). Owner, quantity, capabilities and the provider order are fixed.
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn correct_registration(
        &self,
        registration_id: &str,
        reason: &str,
        data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("registrationId", Some(registration_id)),
            ("reason", Some(reason)),
        ])?;
        let safe = urlencoding::encode(registration_id);
        let body = json!({ "reason": reason, "data": data });
        self.http
            .request_json(
                &format!("/registration/{}/corrections", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Allocate a provider-owned number to a registration through the staff recovery path.
    ///
    /// **POST** `/registration/:registrationId/numbers`
    ///
    /// `phone_number` is the allocated number in E.164 digits (10-15 digits, optional leading
    /// `+`). Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn add_number(
        &self,
        registration_id: &str,
        phone_number: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("registrationId", Some(registration_id)),
            ("phoneNumber", Some(phone_number)),
        ])?;
        let safe = urlencoding::encode(registration_id);
        let body = json!({ "phoneNumber": phone_number });
        self.http
            .request_json(
                &format!("/registration/{}/numbers", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Link a RECONCILIATION_REQUIRED registration to the provider order that was actually placed.
    /// Never place a second order to retry an ambiguous submission.
    ///
    /// **POST** `/registration/:registrationId/reconcile`
    ///
    /// `external_id` is the provider's identifier for the existing order. Requires the `admin`
    /// crate feature.
    #[cfg(feature = "admin")]
    pub async fn reconcile(
        &self,
        registration_id: &str,
        external_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("registrationId", Some(registration_id)),
            ("externalId", Some(external_id)),
        ])?;
        let safe = urlencoding::encode(registration_id);
        let body = json!({ "externalId": external_id });
        self.http
            .request_json(
                &format!("/registration/{}/reconcile", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Get the completed carrier application as `{ filename, contentType, base64 }`; decode
    /// `base64` to save the file.
    ///
    /// **GET** `/registration/:registrationId/application?format=json`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn get_application(
        &self,
        registration_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("registrationId", Some(registration_id))])?;
        let safe = urlencoding::encode(registration_id);
        self.http
            .request_json(
                &format!("/registration/{}/application?format=json", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }
}
