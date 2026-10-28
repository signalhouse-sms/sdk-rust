use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

#[cfg(feature = "admin")]
use serde_json::json;

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

    /// Cancel a registration. Allowed from SIGNAL_HOUSE_REVIEW, SIGNAL_HOUSE_APPROVED,
    /// PENDING_PROVIDER, NEEDS_INFORMATION, PAYMENT_REQUIRED and APPROVED (409 otherwise). Numbers
    /// already allocated stay on the account and stop sending.
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
        HttpClient::require(&[
            ("registrationId", Some(registration_id)),
            ("action", Some(action)),
        ])?;
        let safe = urlencoding::encode(registration_id);
        let mut body = json!({ "action": action });
        if let Some(reason) = reason {
            body["reason"] = json!(reason);
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
