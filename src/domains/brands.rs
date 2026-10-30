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
    /// `tollFree.businessVerificationId` names the business verification the brand uses (starts with `BV`). A group
    /// whose toll-free brands need a business verification must send a `VERIFIED` one of the brand's group (in a
    /// group that verifies each subgroup's business separately, of the brand's subgroup); any other group gets 400 if
    /// it sends one. With it, the legal fields (companyName, street, city, state, postalCode, country, firstName,
    /// lastName and tollFree's businessRegistrationType, taxId, countryCode and taxIdIssuingCountry) are optional:
    /// the business verification's values replace any that are sent. It can't be changed after creation.
    ///
    /// **POST** `/brand/toll-free`
    ///
    /// `registrationType` is forced to `TOLL_FREE` server-side; TFN-specific fields live under
    /// `brand_data["tollFree"]` (businessRegistrationType, legalEntityType, taxId, countryCode,
    /// supportPhone, and optional taxIdIssuingCountry / businessDBA / businessWebsite). `supportPhone`
    /// must be E.164 with an optional leading `+` and no spaces, parentheses or hyphens (e.g.
    /// `"17025550100"`). For Toll-Free, `brandId` is a
    /// `TFNB…`-prefixed id when assigned; otherwise `null` — poll by `_id`.
    pub async fn create_toll_free_brand(
        &self,
        brand_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/brand/toll-free", Method::POST, Some(brand_data), options)
            .await
    }

    /// List a group's business verifications, newest first, and whether its toll-free brands need one.
    ///
    /// **GET** `/brand/business-verification/:groupId`
    ///
    /// Returns `{ groupId, required, businessVerifications }`. Each business verification has
    /// `businessVerificationId`, `groupId`, `status` (`SUBMITTING`, `IN_REVIEW`, `VERIFIED` or `NEEDS_CHANGES`),
    /// `details`, `submittedAt`, `reviewedAt` and `rejectionReason`. For a group that verifies each subgroup's
    /// business separately, use
    /// [`get_business_verifications_for_subgroup`](Self::get_business_verifications_for_subgroup).
    pub async fn get_business_verifications(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.get_business_verifications_for_subgroup(group_id, None, options)
            .await
    }

    /// List business verifications, naming the subgroup they are for.
    ///
    /// **GET** `/brand/business-verification/:groupId?subgroupId=...`
    ///
    /// Some groups verify each subgroup's business separately: a platform whose subgroups are separate businesses,
    /// set up by Signal House. For those groups `subgroup_id` is required (400 without it). It is sent as
    /// `subgroupId` only when `Some`, and must be a subgroup of the group (404 otherwise); any other group's
    /// business verifications cover all its subgroups. Returns the
    /// [`get_business_verifications`](Self::get_business_verifications) shape, plus `subgroupId` when they belong
    /// to that one subgroup.
    pub async fn get_business_verifications_for_subgroup(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        let qs = HttpClient::get_query_string(&[("subgroupId", subgroup_id.map(String::from))]);
        self.http
            .request_json(
                &format!("/brand/business-verification/{}{}", safe, qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get one of a group's business verifications by its ID.
    ///
    /// **GET** `/brand/business-verification/:groupId/:businessVerificationId`
    ///
    /// Returns the business verification, as [`get_business_verifications`](Self::get_business_verifications)
    /// lists it, plus `required`. 404 when it isn't one of the group's.
    pub async fn get_business_verification(
        &self,
        group_id: &str,
        business_verification_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(group_id)),
            ("businessVerificationId", Some(business_verification_id)),
        ])?;
        let safe_group_id = urlencoding::encode(group_id);
        let safe_id = urlencoding::encode(business_verification_id);
        self.http
            .request_json(
                &format!("/brand/business-verification/{}/{}", safe_group_id, safe_id),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create and submit a new business verification. Review usually takes 1–2 business days; once it is
    /// `VERIFIED`, a new toll-free brand can use it as `tollFree.businessVerificationId`.
    ///
    /// **POST** `/brand/business-verification/:groupId`
    ///
    /// `details` holds businessName, businessType, industry, registrationType, registrationNumber, website, street,
    /// optional street2, city, state, postalCode, country (`US` or `CA`) and a `representative` object (firstName,
    /// lastName, email, phone, jobTitle, jobLevel). Returns 202 with the business verification, status
    /// `SUBMITTING`. For a group that verifies each subgroup's business separately, use
    /// [`create_business_verification_for_subgroup`](Self::create_business_verification_for_subgroup).
    pub async fn create_business_verification(
        &self,
        group_id: &str,
        details: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.create_business_verification_for_subgroup(group_id, details, None, options)
            .await
    }

    /// Create and submit a new business verification, naming the subgroup it is for.
    ///
    /// **POST** `/brand/business-verification/:groupId?subgroupId=...`
    ///
    /// `subgroup_id` is required for a group that verifies each subgroup's business separately (400 without it).
    /// It is sent as `subgroupId` only when `Some`, and must be an active subgroup of the group (404 when it is
    /// not in the group, 400 when it is inactive); in any other group the business verification covers all its
    /// subgroups. `details` is as for [`create_business_verification`](Self::create_business_verification).
    /// Returns 202 with the business verification, status `SUBMITTING`.
    pub async fn create_business_verification_for_subgroup(
        &self,
        group_id: &str,
        details: &Value,
        subgroup_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        let qs = HttpClient::get_query_string(&[("subgroupId", subgroup_id.map(String::from))]);
        self.http
            .request_json(
                &format!("/brand/business-verification/{}{}", safe, qs),
                Method::POST,
                Some(details),
                options,
            )
            .await
    }

    /// Correct and resubmit a business verification. Allowed only while its status is `NEEDS_CHANGES` (400
    /// otherwise); the toll-free brands that use it stay pending until it is verified.
    ///
    /// **PUT** `/brand/business-verification/:groupId/:businessVerificationId`
    ///
    /// `details` is as for [`create_business_verification`](Self::create_business_verification). Returns 202 with
    /// the business verification, status `SUBMITTING`; 404 when it isn't one of the group's.
    pub async fn update_business_verification(
        &self,
        group_id: &str,
        business_verification_id: &str,
        details: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(group_id)),
            ("businessVerificationId", Some(business_verification_id)),
        ])?;
        let safe_group_id = urlencoding::encode(group_id);
        let safe_id = urlencoding::encode(business_verification_id);
        self.http
            .request_json(
                &format!("/brand/business-verification/{}/{}", safe_group_id, safe_id),
                Method::PUT,
                Some(details),
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
}
