use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Brands domain -- 10DLC, Toll-Free, and local Short Code brand registration, vetting, and management.
///
/// Brand lookup id: carrier Brand ID (`B…` or `TFNB…`), record `_id`, or internal reference UUID.
/// Used on the update, delete, revet, vetting, appeal, and transfer routes. Pending brands return
/// `brandId: null` in read/create responses — poll `GET /brand?id=<record _id>` until a carrier
/// Brand ID is assigned.
pub struct BrandsDomain {
    http: Arc<HttpClient>,
}

impl BrandsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a list of brands with optional filters.
    ///
    /// **GET** `/brand?...`
    ///
    /// `registration_type` optionally filters by `"TEN_DLC"`, `"TOLL_FREE"`, or `"SHORT_CODE"`.
    ///
    /// Each brand carries `carrierSuspensions` (carrier brand suspension rules; active while liftedAt is null).
    /// Each brand also carries `cnamName` (its caller name for numbers, or null; see `get_caller_name`).
    ///
    /// Each toll-free brand's `tollFree` carries `reviewsBusinessOnSubmit`: whether the brand is reviewed when it's
    /// submitted and keeps its business details from then on. When it's true, `tollFree` also carries the business
    /// details the brand was submitted with (`businessStructure`, `industry`, `contactEmail`, `contactJobTitle` and
    /// `contactJobLevel`) and `rejectionReason`: why the brand was rejected, in plain English, or null.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_brands(
        &self,
        id: Option<&str>,
        subgroup_id: Option<&str>,
        group_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        status: Option<&str>,
        registration_type: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.get_brands_on_current_toll_free_carrier(
            id,
            subgroup_id,
            group_id,
            page,
            limit,
            status,
            registration_type,
            None,
            options,
        )
        .await
    }

    /// Get a list of brands with optional filters, including the current toll-free carrier filter.
    ///
    /// **GET** `/brand?...`
    ///
    /// `current_toll_free_carrier`: with `subgroup_id` or `group_id`, `Some(true)` returns only the toll-free brands
    /// a new toll-free campaign in that group can use (those on the group's current toll-free carrier).
    #[allow(clippy::too_many_arguments)]
    pub async fn get_brands_on_current_toll_free_carrier(
        &self,
        id: Option<&str>,
        subgroup_id: Option<&str>,
        group_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        status: Option<&str>,
        registration_type: Option<&str>,
        current_toll_free_carrier: Option<bool>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("id", id.map(String::from)),
            ("subgroupId", subgroup_id.map(String::from)),
            ("groupId", group_id.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
            ("status", status.map(String::from)),
            ("registrationType", registration_type.map(String::from)),
            (
                "currentTollFreeCarrier",
                current_toll_free_carrier.map(|c| c.to_string()),
            ),
        ]);
        self.http
            .request_json(&format!("/brand{}", qs), Method::GET, None, options)
            .await
    }

    /// Get external vetting information for a brand.
    ///
    /// **GET** `/brand/externalvetting/:brandId`
    pub async fn get_external_vetting(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(
                &format!("/brand/externalvetting/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a new brand.
    ///
    /// **POST** `/brand`
    pub async fn create_brand(
        &self,
        brand_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/brand", Method::POST, Some(brand_data), options)
            .await
    }

    /// Create a new Toll-Free (TFN) brand.
    ///
    /// **POST** `/brand/toll-free`
    ///
    /// `registrationType` is forced to `TOLL_FREE` server-side; TFN-specific fields live under
    /// `brand_data["tollFree"]` (businessRegistrationType, legalEntityType, taxId, countryCode,
    /// supportPhone, and optional taxIdIssuingCountry / businessDBA / businessWebsite). `supportPhone`
    /// must be E.164 with an optional leading `+` and no spaces, parentheses or hyphens (e.g.
    /// `"17025550100"`). For Toll-Free, `brandId` is a
    /// `TFNB…`-prefixed id when assigned; otherwise `null` — poll by `_id`.
    ///
    /// When [`get_toll_free_requirements`](Self::get_toll_free_requirements) returns `reviewsBusinessOnSubmit: true`
    /// for the group, `tollFree.countryCode` must be `US` or `CA`, `state` must be a 2-letter state or province code,
    /// and `brand_data["tollFree"]` also needs these five business details:
    ///
    /// - `businessStructure`: `CO_OPERATIVE`, `CORPORATION`, `LIMITED_LIABILITY_CORPORATION`,
    ///   `NON_PROFIT_CORPORATION` or `PARTNERSHIP`.
    /// - `industry`: `AGRICULTURE`, `AUTOMOTIVE`, `BANKING`, `CONSTRUCTION`, `CONSUMER`, `EDUCATION`, `ELECTRONICS`,
    ///   `ENGINEERING`, `ENERGY`, `FAST_MOVING_CONSUMER_GOODS`, `FINANCIAL`, `FINTECH`, `FOOD_AND_BEVERAGE`,
    ///   `GOVERNMENT`, `HEALTHCARE`, `HOSPITALITY`, `INSURANCE`, `JEWELRY`, `LEGAL`, `MANUFACTURING`, `MEDIA`,
    ///   `NOT_FOR_PROFIT`, `OIL_AND_GAS`, `ONLINE`, `PROFESSIONAL_SERVICES`, `RAW_MATERIALS`, `REAL_ESTATE`,
    ///   `RELIGION`, `RETAIL`, `TECHNOLOGY`, `TELECOMMUNICATIONS`, `TRANSPORTATION` or `TRAVEL`.
    /// - `contactEmail`: the contact's email address (at most 100 characters).
    /// - `contactJobTitle`: the contact's job title (at most 100 characters).
    /// - `contactJobLevel`: `DIRECTOR`, `GM`, `VP`, `CEO`, `CFO`, `GENERAL_COUNSEL` or `OTHER`.
    ///
    /// Any other group gets 400 if it sends `businessStructure`, `industry`, `contactEmail`, `contactJobTitle` or
    /// `contactJobLevel`. A brand reviewed on submit stays `DCA_PENDING` until its business is reviewed (usually 1–2
    /// business days), then becomes `DCA_ACTIVE`, or `DCA_REJECTED` with `tollFree.rejectionReason` saying why; a
    /// rejected brand is fixed by creating the brand again with corrected details. Its business details can't be
    /// changed after it's submitted: companyName, street, city, state, postalCode, country, firstName, lastName and
    /// tollFree's businessRegistrationType, taxId, countryCode, taxIdIssuingCountry, businessStructure, industry,
    /// contactEmail, contactJobTitle and contactJobLevel.
    pub async fn create_toll_free_brand(
        &self,
        brand_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/brand/toll-free", Method::POST, Some(brand_data), options)
            .await
    }

    /// Get whether a group's new toll-free brands are reviewed when they're submitted, which decides the details
    /// they need.
    ///
    /// **GET** `/brand/toll-free-requirements/:groupId`
    ///
    /// Returns `{ groupId, reviewsBusinessOnSubmit }`. When `reviewsBusinessOnSubmit` is true, a new toll-free
    /// brand needs `businessStructure`, `industry`, `contactEmail`, `contactJobTitle` and `contactJobLevel` in
    /// `tollFree`, a `tollFree.countryCode` of `US` or `CA` and a 2-letter `state`; it stays `DCA_PENDING` until its
    /// business is reviewed, and its business details can't be changed after it's submitted. When it's false, a new
    /// toll-free brand gets 400 if it sends any of those five fields. 403 when the caller can't view the group.
    pub async fn get_toll_free_requirements(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/brand/toll-free-requirements/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a local Short Code brand for manual Signal House review.
    ///
    /// **POST** `/brand/short-code`
    ///
    /// The route forces `SHORT_CODE`; the response is a synchronous 201-created record in
    /// `PENDING_APPROVAL` with an `SCB…`-prefixed brand ID.
    pub async fn create_short_code_brand(
        &self,
        brand_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/brand/short-code", Method::POST, Some(brand_data), options)
            .await
    }

    /// Approve a pending Short Code brand. signalhouse_admin only.
    ///
    /// **POST** `/brand/approve/:brandId`
    pub async fn approve_short_code_brand(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        self.http
            .request_json(
                &format!("/brand/approve/{}", urlencoding::encode(brand_id)),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Reject a pending Short Code brand with a required customer-visible reason. signalhouse_admin only.
    ///
    /// **POST** `/brand/reject/:brandId`
    pub async fn reject_short_code_brand(
        &self,
        brand_id: &str,
        internal_rejection_reason: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("brandId", Some(brand_id)),
            ("internalRejectionReason", Some(internal_rejection_reason)),
        ])?;
        let body = serde_json::json!({ "internalRejectionReason": internal_rejection_reason });
        self.http
            .request_json(
                &format!("/brand/reject/{}", urlencoding::encode(brand_id)),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Transfer one or more brands to a different subgroup.
    ///
    /// **POST** `/brand/transfer/:subgroupId`
    pub async fn transfer_brand(
        &self,
        subgroup_id: &str,
        brand_ids: &[String],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("subgroupId", Some(subgroup_id))])?;
        let safe = urlencoding::encode(subgroup_id);
        let body = serde_json::json!({ "brandIds": brand_ids });
        self.http
            .request_json(
                &format!("/brand/transfer/{}", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Create external vetting for a brand.
    ///
    /// `vetting_provider_id` is AEGIS or WMC; `vetting_class` is STANDARD, ENHANCED, AUTHPLUS, or
    /// RCS (AEGIS accepts all four; WMC accepts STANDARD only). `additional_data` is
    /// provider-specific data forwarded to TCR; pass `None` to omit it.
    ///
    /// **POST** `/brand/externalvetting/:brandId`
    pub async fn create_external_vetting(
        &self,
        brand_id: &str,
        vetting_provider_id: &str,
        vetting_class: &str,
        additional_data: Option<&Value>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("brandId", Some(brand_id)),
            ("vettingProviderId", Some(vetting_provider_id)),
            ("vettingClass", Some(vetting_class)),
        ])?;
        let safe = urlencoding::encode(brand_id);
        let mut body = serde_json::json!({
            "vettingProviderId": vetting_provider_id,
            "vettingClass": vetting_class,
        });
        if let Some(data) = additional_data {
            body["additionalData"] = data.clone();
        }
        self.http
            .request_json(
                &format!("/brand/externalvetting/{}", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Import an existing external vetting record for a brand.
    ///
    /// Unlike [`create_external_vetting`](Self::create_external_vetting) (which orders a new,
    /// billable vetting), this attaches a vetting the brand already completed directly with the
    /// provider, using the provider-issued vettingId and vettingToken. It is synchronous and not
    /// billable.
    ///
    /// **POST** `/brand/externalvetting/import/:brandId`
    ///
    /// `vetting_token` is required by some providers (e.g. AEGIS); pass `None` to omit it.
    pub async fn import_external_vetting(
        &self,
        brand_id: &str,
        vetting_provider_id: &str,
        vetting_id: &str,
        vetting_token: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("brandId", Some(brand_id)),
            ("vettingProviderId", Some(vetting_provider_id)),
            ("vettingId", Some(vetting_id)),
        ])?;
        let safe = urlencoding::encode(brand_id);
        let mut body = serde_json::json!({
            "vettingProviderId": vetting_provider_id,
            "vettingId": vetting_id,
        });
        if let Some(t) = vetting_token {
            body["vettingToken"] = Value::String(t.to_string());
        }
        self.http
            .request_json(
                &format!("/brand/externalvetting/import/{}", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Update a brand's information.
    ///
    /// **PUT** `/brand/:brandId`
    ///
    /// For a Toll-Free brand, pass the editable Toll-Free fields under a `tollFree` sub-object in
    /// `brand_data` (legalEntityType, businessRegistrationType, taxId, countryCode, supportPhone,
    /// taxIdIssuingCountry, businessDBA, businessWebsite); `supportPhone` must be E.164 with an
    /// optional leading `+` and no spaces, parentheses or hyphens (e.g. `"17025550100"`);
    /// `subgroupId` is immutable.
    pub async fn update_brand(
        &self,
        brand_id: &str,
        brand_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(
                &format!("/brand/{}", safe),
                Method::PUT,
                Some(brand_data),
                options,
            )
            .await
    }

    /// Re-vet a brand that is in UNVERIFIED status.
    ///
    /// **PUT** `/brand/revet/:brandId`
    pub async fn revet_brand(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(
                &format!("/brand/revet/{}", safe),
                Method::PUT,
                None,
                options,
            )
            .await
    }

    /// Delete a brand (mark as DELETED).
    ///
    /// **DELETE** `/brand/:brandId`
    pub async fn delete_brand(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(&format!("/brand/{}", safe), Method::DELETE, None, options)
            .await
    }

    /// Get appeal history for a brand.
    ///
    /// **GET** `/brand/appeal/:brandId`
    pub async fn get_appeal_history(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(
                &format!("/brand/appeal/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Submit an appeal for a brand (multipart).
    ///
    /// **POST** `/brand/appeal/:brandId`
    ///
    /// - `appeal_categories`: list of appeal category strings.
    /// - `explanation`: explanation text for the appeal.
    /// - `file`: optional supporting document as `(bytes, filename)`.
    pub async fn submit_appeal(
        &self,
        brand_id: &str,
        appeal_categories: &[String],
        explanation: &str,
        file: Option<(Vec<u8>, String)>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);

        let mut form = Form::new();

        if let Some((bytes, filename)) = file {
            let part = Part::bytes(bytes).file_name(filename);
            form = form.part("file", part);
        }

        let categories_json =
            serde_json::to_string(appeal_categories).unwrap_or_else(|_| "[]".to_string());
        form = form.text("appealCategories", categories_json);
        form = form.text("explanation", explanation.to_string());

        self.http
            .request_multipart(
                &format!("/brand/appeal/{}", safe),
                Method::POST,
                form,
                options,
            )
            .await
    }

    /// Read a brand's caller name (CNAM) and the numbers it applies to: every number whose caller
    /// name comes from this brand (its campaign numbers, numbers it was picked on, and campaign-less
    /// numbers of subgroups that use it as their default caller-ID brand). Resolves to
    /// `{ brandId, companyName, cnamName, eligible, numbers: { total, items } }`; `items` lists the
    /// first 100.
    ///
    /// Roles: api, admin, developer, billing, user.
    ///
    /// **GET** `/brand/cnam/:brandId`
    pub async fn get_caller_name(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(&format!("/brand/cnam/{}", safe), Method::GET, None, options)
            .await
    }

    /// Set a brand's caller name (CNAM). It replaces the caller name on every number whose caller
    /// name comes from this brand, names set on a number included, and numbers that later come under
    /// the brand get it too. The name may only remove characters from the brand's registered name and
    /// must be 1-15 letters, digits, spaces, `-` or `_`; the brand must be verified. Numbers are
    /// updated at the carrier in the background. Resolves to
    /// `{ brandId, companyName, cnamName, eligible, numbers: { total, items } }`.
    ///
    /// Roles: api, admin, developer.
    ///
    /// **PUT** `/brand/cnam/:brandId`
    pub async fn set_caller_name(
        &self,
        brand_id: &str,
        name: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id)), ("name", Some(name))])?;
        let safe = urlencoding::encode(brand_id);
        let body = serde_json::json!({ "name": name });
        self.http
            .request_json(
                &format!("/brand/cnam/{}", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Remove a brand's caller name (CNAM). Numbers that had it go back to the automatic name if
    /// their subgroup has caller name auto-apply on, otherwise to no caller name. Names set on a
    /// number are kept. Resolves to
    /// `{ brandId, companyName, cnamName, eligible, numbers: { total, items } }`.
    ///
    /// Roles: api, admin, developer.
    ///
    /// **DELETE** `/brand/cnam/:brandId`
    pub async fn delete_caller_name(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(
                &format!("/brand/cnam/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }
}
