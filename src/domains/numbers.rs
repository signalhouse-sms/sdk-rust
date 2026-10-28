use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Numbers domain -- phone number purchasing, assignment, and management.
pub struct NumbersDomain {
    http: Arc<HttpClient>,
}

impl NumbersDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a list of owned phone numbers with optional filters.
    ///
    /// **GET** `/number?...`
    ///
    /// `registration_id` filters to numbers provisioned under this registration.
    ///
    /// Every record carries a `sentiment` object -- trailing 7-day and 30-day inbound reply sentiment,
    /// refreshed periodically from analytics rather than computed per request: `{ sevenDay, thirtyDay,
    /// updatedAt }`, each window being `{ score, label, scoredCount, positive, neutral, negative }`. The
    /// object is always present and fully populated, so it can be read without a presence check.
    /// `score` is a volume-weighted average in -100..100 and is null when nothing was scored in the
    /// window, which is not the same as a score of 0 -- null means nobody replied, 0 means replies
    /// averaged neutral. `updatedAt` is null until the first rollup writes the record, and it tracks when
    /// the figures last CHANGED, not when the job last ran.
    ///
    /// Every record also carries a `health` object -- trailing 7-day and 30-day delivery/opt-out health from
    /// the same periodic refresh: `{ sevenDay, thirtyDay, updatedAt }`, each window being `{ score,
    /// deliveryRate, optOutRate, deliveryScore, optOutScore, messagesSent, messagesDelivered, messagesFailed,
    /// optOuts }`. `score` is 1.0..10.0 (the mean of the two bucket scores) and is null when nothing was sent
    /// in the window; the volumes say how much traffic sits behind it. A campaign's figure is scored from the
    /// campaign's own summed counters, never averaged from its numbers. For a live figure use GET /number/health
    /// or GET /campaign/health.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_phone_numbers(
        &self,
        phone_number: Option<&str>,
        campaign_id: Option<&str>,
        registration_id: Option<&str>,
        brand_id: Option<&str>,
        subgroup_id: Option<&str>,
        group_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.get_phone_numbers_on_current_toll_free_carrier(
            phone_number,
            campaign_id,
            registration_id,
            brand_id,
            subgroup_id,
            group_id,
            page,
            limit,
            None,
            options,
        )
        .await
    }

    /// Get a list of owned phone numbers with optional filters, including the current toll-free carrier filter.
    ///
    /// **GET** `/number?...`
    ///
    /// `current_toll_free_carrier`: with `subgroup_id` or `group_id`, `Some(true)` returns only the toll-free numbers
    /// a new toll-free campaign in that group can use (those on the group's current toll-free carrier).
    #[allow(clippy::too_many_arguments)]
    pub async fn get_phone_numbers_on_current_toll_free_carrier(
        &self,
        phone_number: Option<&str>,
        campaign_id: Option<&str>,
        registration_id: Option<&str>,
        brand_id: Option<&str>,
        subgroup_id: Option<&str>,
        group_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        current_toll_free_carrier: Option<bool>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("phoneNumber", phone_number.map(String::from)),
            ("campaignId", campaign_id.map(String::from)),
            ("registrationId", registration_id.map(String::from)),
            ("brandId", brand_id.map(String::from)),
            ("subgroupId", subgroup_id.map(String::from)),
            ("groupId", group_id.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
            (
                "currentTollFreeCarrier",
                current_toll_free_carrier.map(|c| c.to_string()),
            ),
        ]);
        self.http
            .request_json(&format!("/number{}", qs), Method::GET, None, options)
            .await
    }

    /// Get the health score, the same one the portal shows, (1–10) for an owned phone number.
    ///
    /// **GET** `/number/health?phoneNumber=...`
    pub async fn get_number_health(
        &self,
        phone_number: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;
        let qs = HttpClient::get_query_string(&[("phoneNumber", Some(phone_number.to_string()))]);
        self.http
            .request_json(&format!("/number/health{}", qs), Method::GET, None, options)
            .await
    }

    /// Get a list of phone numbers available for purchase.
    ///
    /// **GET** `/number/available?...`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_available_phone_numbers(
        &self,
        sms_enabled: Option<bool>,
        mms_enabled: Option<bool>,
        voice_enabled: Option<bool>,
        country: Option<&str>,
        state: Option<&str>,
        npa: Option<&str>,
        nxx: Option<&str>,
        phone_number: Option<&str>,
        limit: Option<u32>,
        page: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.get_available_phone_numbers_in_city(
            sms_enabled,
            mms_enabled,
            voice_enabled,
            country,
            state,
            npa,
            nxx,
            phone_number,
            limit,
            page,
            None,
            options,
        )
        .await
    }

    /// Search by city prefix within a US state, preserving the original method.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_available_phone_numbers_in_city(
        &self,
        sms_enabled: Option<bool>,
        mms_enabled: Option<bool>,
        voice_enabled: Option<bool>,
        country: Option<&str>,
        state: Option<&str>,
        npa: Option<&str>,
        nxx: Option<&str>,
        phone_number: Option<&str>,
        limit: Option<u32>,
        page: Option<u32>,
        city: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("smsEnabled", sms_enabled.map(|b| b.to_string())),
            ("mmsEnabled", mms_enabled.map(|b| b.to_string())),
            ("voiceEnabled", voice_enabled.map(|b| b.to_string())),
            ("country", country.map(String::from)),
            ("state", state.map(String::from)),
            ("city", city.map(String::from)),
            ("npa", npa.map(String::from)),
            ("nxx", nxx.map(String::from)),
            ("phoneNumber", phone_number.map(String::from)),
            ("limit", limit.map(|l| l.to_string())),
            ("page", page.map(|p| p.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/number/available{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Purchase phone numbers and assign them to a subgroup.
    ///
    /// Each requested number is queued and processed independently. Per-number success or
    /// failure is delivered via `NUMBER_PURCHASE_SUCCESSFUL` / `NUMBER_PURCHASE_FAILED`
    /// webhooks — not in this response.
    ///
    /// **POST** `/number`
    pub async fn purchase_phone_number(
        &self,
        phone_numbers: &[String],
        subgroup_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.purchase_phone_number_in_country(phone_numbers, subgroup_id, "US", options)
            .await
    }

    /// Country-aware purchase with an explicit ISO-2 country (US); existing purchase methods retain US defaults.
    pub async fn purchase_phone_number_in_country(
        &self,
        phone_numbers: &[String],
        subgroup_id: &str,
        country: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("subgroupId", Some(subgroup_id))])?;
        let body =
            json!({ "phoneNumbers": phone_numbers, "subgroupId": subgroup_id, "country": country });
        self.http
            .request_json("/number", Method::POST, Some(&body), options)
            .await
    }

    /// Purchase one or more Toll-Free numbers via the asynchronous resource-request flow.
    ///
    /// Toll-Free numbers are ordered by `quantity` (1-10, not picked individually) and provisioned
    /// asynchronously; per-number completion is delivered via webhook/polling, not in this response.
    /// Poll the returned `orderId` with [`get_toll_free_order_status`](Self::get_toll_free_order_status).
    ///
    /// **POST** `/number/toll-free`
    pub async fn purchase_toll_free_numbers(
        &self,
        quantity: u32,
        subgroup_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.purchase_toll_free_numbers_in_country(quantity, subgroup_id, "US", options)
            .await
    }

    /// Country-aware purchase with an explicit ISO-2 country (US); existing purchase methods retain US defaults.
    pub async fn purchase_toll_free_numbers_in_country(
        &self,
        quantity: u32,
        subgroup_id: &str,
        country: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("subgroupId", Some(subgroup_id))])?;
        let body = json!({ "quantity": quantity, "subgroupId": subgroup_id, "country": country });
        self.http
            .request_json("/number/toll-free", Method::POST, Some(&body), options)
            .await
    }

    /// Request a campaign-bound Short Code acquisition or register a customer-owned Registry lease.
    ///
    /// RANDOM and VANITY requests are fulfilled by Signal House staff. EXTERNAL_LEASE creates a pending Short Code
    /// number immediately; it becomes READY only when its campaign becomes ACTIVE. A `YYYY-MM-DD` lease
    /// end date expires at 23:59:59 UTC.
    ///
    /// **POST** `/number/short-code`
    #[allow(clippy::too_many_arguments)]
    pub async fn request_short_code_acquisition(
        &self,
        subgroup_id: &str,
        brand_id: &str,
        campaign_id: &str,
        request_type: &str,
        lease_term_months: Option<u32>,
        requested_vanity_code: Option<&str>,
        actual_code: Option<&str>,
        order_id: Option<&str>,
        lease_end_date: Option<&str>,
        options: Option<&RequestOptions>,
        lease_receipt_file: Option<Vec<u8>>,
        lease_receipt_filename: Option<String>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("subgroupId", Some(subgroup_id)),
            ("brandId", Some(brand_id)),
            ("campaignId", Some(campaign_id)),
            ("requestType", Some(request_type)),
        ])?;
        let acquisition_data = json!({
            "subgroupId": subgroup_id,
            "brandId": brand_id,
            "campaignId": campaign_id,
            "requestType": request_type,
            "leaseTermMonths": lease_term_months,
            "requestedVanityCode": requested_vanity_code,
            "actualCode": actual_code,
            "orderId": order_id,
            "leaseEndDate": lease_end_date,
        });
        let mut form = Form::new().text("acquisitionData", acquisition_data.to_string());
        if let Some(receipt_bytes) = lease_receipt_file {
            let filename = lease_receipt_filename.unwrap_or_else(|| "receipt.pdf".to_string());
            form = form.part(
                "leaseReceipt",
                Part::bytes(receipt_bytes).file_name(filename),
            );
        }
        self.http
            .request_multipart("/number/short-code", Method::POST, form, options)
            .await
    }

    /// Read the outcome of a Toll-Free number purchase by its order ID.
    ///
    /// Reports per-number ready/provisioning/failed counts (plus failure reasons) and the
    /// provisioned numbers; `numbers` fills in as the order completes, so poll until the order
    /// reaches a terminal state.
    ///
    /// **GET** `/number/toll-free/order/:orderId`
    pub async fn get_toll_free_order_status(
        &self,
        order_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("orderId", Some(order_id))])?;
        let safe = urlencoding::encode(order_id);
        self.http
            .request_json(
                &format!("/number/toll-free/order/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Update a phone number's details (e.g. friendly name).
    ///
    /// **PUT** `/number/:phoneNumber`
    pub async fn update_phone_number(
        &self,
        phone_number: &str,
        update_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;
        let safe = urlencoding::encode(phone_number);
        self.http
            .request_json(
                &format!("/number/{}", safe),
                Method::PUT,
                Some(update_data),
                options,
            )
            .await
    }

    /// Read a phone number's outbound caller name (CNAM) and whether one can be set.
    /// Resolves to `{ phoneNumber, cnam, autoApply, eligible, ineligibleReason }`.
    ///
    /// Roles: api, admin, developer, billing, user.
    ///
    /// **GET** `/number/:phoneNumber/cnam`
    pub async fn get_caller_name(
        &self,
        phone_number: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;
        let safe = urlencoding::encode(phone_number);
        self.http
            .request_json(
                &format!("/number/{}/cnam", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Set a phone number's outbound caller name (CNAM). The name may only remove characters from
    /// the registered name of the number's brand (its campaign's brand, else `brand_id`, else the
    /// subgroup's default caller-ID brand), and must be 1-15 letters, digits, spaces, `-` or `_`.
    /// `brand_id` is the verified brand the name comes from; only for numbers with no campaign
    /// (omitted when `None` or empty). Resolves to
    /// `{ phoneNumber, cnam, autoApply, eligible, ineligibleReason }`.
    ///
    /// Roles: api, admin, developer.
    ///
    /// **PUT** `/number/:phoneNumber/cnam`
    pub async fn set_caller_name(
        &self,
        phone_number: &str,
        name: &str,
        brand_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number)), ("name", Some(name))])?;
        let safe = urlencoding::encode(phone_number);
        let mut body = json!({ "name": name });
        if let Some(b) = brand_id.filter(|b| !b.is_empty()) {
            body["brandId"] = Value::String(b.to_string());
        }
        self.http
            .request_json(
                &format!("/number/{}/cnam", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Remove a phone number's own caller name (CNAM). If its subgroup has caller name auto-apply
    /// on, the number goes back to the automatic name. Resolves to
    /// `{ phoneNumber, cnam, autoApply, eligible, ineligibleReason }`.
    ///
    /// Roles: api, admin, developer.
    ///
    /// **DELETE** `/number/:phoneNumber/cnam`
    pub async fn delete_caller_name(
        &self,
        phone_number: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;
        let safe = urlencoding::encode(phone_number);
        self.http
            .request_json(
                &format!("/number/{}/cnam", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Submit a port-in request for one or more phone numbers from another provider.
    ///
    /// **POST** `/number/portin`
    pub async fn port_in_phone_number(
        &self,
        number_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/number/portin", Method::POST, Some(number_data), options)
            .await
    }

    /// Get port-in requests with optional filters.
    /// Signal House staff can view all requests; regular users see only their group's requests.
    ///
    /// `exclude_documents`: `Some(true)` leaves `signatureUrl` and `billingUrl` out of each request.
    ///
    /// **GET** `/number/portin?...`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_port_requests(
        &self,
        group_id: Option<&str>,
        phone_number: Option<&str>,
        status: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        exclude_documents: Option<bool>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", group_id.map(String::from)),
            ("phoneNumber", phone_number.map(String::from)),
            ("status", status.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
            ("excludeDocuments", exclude_documents.map(|e| e.to_string())),
        ]);
        self.http
            .request_json(&format!("/number/portin{}", qs), Method::GET, None, options)
            .await
    }

    /// Assign phone numbers to a campaign. Numbers that are not found, already RELEASED, already assigned
    /// to a different campaign, on a mismatched DCA, or (for a 10DLC campaign without a number pool)
    /// beyond the remaining 49-number-per-campaign limit are not queued; the response `data` resolves to
    /// `{ message, queued, skipped }`, where `skipped` numbers were rejected individually for one of those
    /// reasons rather than failing the whole request. A Toll-Free campaign whose brand reports
    /// `tollFree.maxNumbersPerCampaign` as 1 holds one number: a request that would give it a second is
    /// refused with a 400 and nothing is queued.
    ///
    /// **POST** `/number/assign/:campaignId`
    pub async fn assign_phone_number_to_campaign(
        &self,
        campaign_id: &str,
        phone_numbers: &[String],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("campaignId", Some(campaign_id))])?;
        let safe = urlencoding::encode(campaign_id);
        let body = json!({ "phoneNumbers": phone_numbers });
        self.http
            .request_json(
                &format!("/number/assign/{}", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Unassign phone numbers from any campaign they are assigned to.
    ///
    /// **POST** `/number/unassign`
    pub async fn unassign_phone_numbers_from_campaigns(
        &self,
        phone_numbers: &[String],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({ "phoneNumbers": phone_numbers });
        self.http
            .request_json("/number/unassign", Method::POST, Some(&body), options)
            .await
    }

    /// Delete (release) phone numbers.
    ///
    /// **DELETE** `/number`
    pub async fn delete_phone_numbers(
        &self,
        phone_numbers: &[String],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({ "phoneNumbers": phone_numbers });
        self.http
            .request_json("/number", Method::DELETE, Some(&body), options)
            .await
    }

    /// Deactivate one or more phone numbers. Numbers must be in READY status.
    ///
    /// **POST** `/number/deactivate`
    pub async fn deactivate_phone_numbers(
        &self,
        phone_numbers: &[String],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({ "phoneNumbers": phone_numbers });
        self.http
            .request_json("/number/deactivate", Method::POST, Some(&body), options)
            .await
    }

    /// Reactivate one or more previously deactivated phone numbers.
    ///
    /// **POST** `/number/reactivate`
    pub async fn reactivate_phone_numbers(
        &self,
        phone_numbers: &[String],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({ "phoneNumbers": phone_numbers });
        self.http
            .request_json("/number/reactivate", Method::POST, Some(&body), options)
            .await
    }

    /// Transfer unassigned phone numbers to another subgroup.
    ///
    /// **POST** `/number/transfer`
    pub async fn transfer_phone_numbers(
        &self,
        phone_numbers: &[String],
        new_subgroup_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("newSubgroupId", Some(new_subgroup_id))])?;
        let body = json!({ "phoneNumbers": phone_numbers, "newSubgroupId": new_subgroup_id });
        self.http
            .request_json("/number/transfer", Method::POST, Some(&body), options)
            .await
    }

    /// Search NPA/NXX lookup data with optional filters.
    /// Country may scope NPA/NXX filters; state and city cannot be combined with them.
    ///
    /// **GET** `/number/lookup?...`
    pub async fn search_npa_nxx(
        &self,
        country: Option<&str>,
        state: Option<&str>,
        city: Option<&str>,
        npa: Option<&str>,
        nxx: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("country", country.map(String::from)),
            ("state", state.map(String::from)),
            ("city", city.map(String::from)),
            ("npa", npa.map(String::from)),
            ("nxx", nxx.map(String::from)),
        ]);
        self.http
            .request_json(&format!("/number/lookup{}", qs), Method::GET, None, options)
            .await
    }

    /// Batch lookup city/state for NPA/NXX pairs.
    ///
    /// **POST** `/number/lookup/location`
    pub async fn lookup_locations(
        &self,
        entries: &[Value],
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({ "entries": entries });
        self.http
            .request_json(
                "/number/lookup/location",
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Get a single port-in request by ID.
    ///
    /// **GET** `/number/portin/:id`
    pub async fn get_port_request_by_id(
        &self,
        porting_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("portingId", Some(porting_id))])?;
        let safe = urlencoding::encode(porting_id);
        self.http
            .request_json(
                &format!("/number/portin/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get port-in requests. If `porting_id` is provided, fetches a single request by ID;
    /// otherwise returns a filtered list of port-in requests.
    ///
    /// **GET** `/number/portin/:id` or `/number/portin?...`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_porting_requests(
        &self,
        porting_id: Option<&str>,
        group_id: Option<&str>,
        phone_number: Option<&str>,
        status: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        if let Some(id) = porting_id {
            return self.get_port_request_by_id(id, options).await;
        }
        self.get_port_requests(group_id, phone_number, status, page, limit, None, options)
            .await
    }

    /// Change the status of a port-in request. Infobip exposes no porting-status API or webhook,
    /// so ON_HOLD and its reason are always staff-entered here.
    ///
    /// `status` is one of PENDING, IN_PROGRESS, ON_HOLD, REJECTED. `on_hold_reason` is required
    /// when `status` is ON_HOLD (DOCUMENTATION_MISMATCH, ACCOUNT_INFO_INCORRECT, CARRIER_REJECTED,
    /// PENDING_CUSTOMER_ACTION, OTHER).
    ///
    /// **PUT** `/number/portin/:id/status`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn change_porting_status(
        &self,
        porting_id: &str,
        status: &str,
        description: Option<&str>,
        on_hold_reason: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("portingId", Some(porting_id)), ("status", Some(status))])?;
        let safe = urlencoding::encode(porting_id);
        let mut body = json!({ "status": status });
        if let Some(desc) = description {
            body["description"] = json!(desc);
        }
        if let Some(reason) = on_hold_reason {
            body["onHoldReason"] = json!(reason);
        }
        self.http
            .request_json(
                &format!("/number/portin/{}/status", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Approve a port-in request.
    ///
    /// `sub_group_id` assigns a subgroup when the request has none yet. Ignored if the request
    /// already has a subgroup assigned -- an existing assignment is never overwritten.
    ///
    /// **POST** `/number/portin/:id/approve`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn approve_port_request(
        &self,
        porting_id: &str,
        sub_group_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("portingId", Some(porting_id))])?;
        let safe = urlencoding::encode(porting_id);
        let body = sub_group_id.map(|v| json!({ "subGroupId": v }));
        self.http
            .request_json(
                &format!("/number/portin/{}/approve", safe),
                Method::POST,
                body.as_ref(),
                options,
            )
            .await
    }

    /// List campaign-bound Short Code acquisition requests.
    ///
    /// **GET** `/number/short-code/requests`
    #[cfg(feature = "admin")]
    pub async fn get_short_code_acquisition_requests(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
        status: Option<&str>,
        search: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("page", page.map(|value| value.to_string())),
            ("limit", limit.map(|value| value.to_string())),
            ("status", status.map(String::from)),
            ("search", search.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/number/short-code/requests{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Migrate a phone number (or all numbers in a group) from v1 to v2.
    ///
    /// # Arguments
    /// * `phone_number` - Phone number to migrate (optional). If provided, group_id is ignored.
    /// * `campaign_id` - Campaign ID to migrate all numbers for (optional).
    /// * `group_id` - Group ID to migrate all non-released numbers for (optional).
    /// * `options` - Additional request options.
    pub async fn migrate_phone_number_to_v2(
        &self,
        phone_number: Option<&str>,
        campaign_id: Option<&str>,
        group_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body =
            json!({ "phoneNumber": phone_number, "campaignId": campaign_id, "groupId": group_id });
        self.http
            .request_json("/number/migrate", Method::POST, Some(&body), options)
            .await
    }
}
