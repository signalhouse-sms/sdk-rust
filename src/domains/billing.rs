use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Billing domain -- wallets, payment methods, funds, invoices, and fees.
///
/// Date-range parameters (`start_date`, `end_date`) on list and history
/// methods accept ISO-8601 date or timestamp strings. The server normalizes
/// them to full UTC-day boundaries: `start_date` -> start-of-UTC-day,
/// `end_date` -> end-of-UTC-day. The end day is always included; hourly
/// resolution is not supported.
pub struct BillingDomain {
    http: Arc<HttpClient>,
}

impl BillingDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get transaction history with optional filters.
    ///
    /// **GET** `/billing/wallet/transactionHistory?...`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_transaction_history(
        &self,
        group_id: Option<&str>,
        subgroup_id: Option<&str>,
        brand_id: Option<&str>,
        campaign_id: Option<&str>,
        entry_type: Option<&str>,
        transaction_type: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", group_id.map(String::from)),
            ("subgroupId", subgroup_id.map(String::from)),
            ("brandId", brand_id.map(String::from)),
            ("campaignId", campaign_id.map(String::from)),
            ("entryType", entry_type.map(String::from)),
            ("transactionType", transaction_type.map(String::from)),
            ("startDate", start_date.map(String::from)),
            ("endDate", end_date.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/billing/wallet/transactionHistory{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get payment history (wallet recharge transactions) with server-side pagination.
    ///
    /// **GET** `/billing/wallet/paymentHistory?...`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_payment_history(
        &self,
        group_id: &str,
        start_date: &str,
        end_date: &str,
        page: Option<u32>,
        limit: Option<u32>,
        timezone: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("startDate", Some(start_date.to_string())),
            ("endDate", Some(end_date.to_string())),
            ("page", page.map(|v| v.to_string())),
            ("limit", limit.map(|v| v.to_string())),
            ("timezone", timezone.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/billing/wallet/paymentHistory{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get wallet information for a group.
    ///
    /// **GET** `/billing/wallet/:groupId`
    pub async fn get_wallet(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/billing/wallet/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Update wallet settings for a group.
    ///
    /// **PUT** `/billing/wallet/:groupId`
    pub async fn update_wallet(
        &self,
        group_id: &str,
        body: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/billing/wallet/{}", safe),
                Method::PUT,
                Some(body),
                options,
            )
            .await
    }

    /// Get payment methods for a group.
    ///
    /// **GET** `/billing/wallet/paymentMethods/:groupId`
    pub async fn get_payment_methods(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/billing/wallet/paymentMethods/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Add funds to a group's wallet.
    ///
    /// **POST** `/billing/wallet/addfunds/:groupId`
    pub async fn add_funds(
        &self,
        group_id: &str,
        amount: i64,
        payment_method_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        let body = json!({
            "amount": amount,
            "paymentMethodId": payment_method_id,
        });
        self.http
            .request_json(
                &format!("/billing/wallet/addfunds/{}", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Add a payment method to a group's wallet.
    ///
    /// **POST** `/billing/wallet/paymentMethod/:groupId`
    pub async fn add_payment_method(
        &self,
        group_id: &str,
        payment_method_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(group_id)),
            ("paymentMethodId", Some(payment_method_id)),
        ])?;
        let safe = urlencoding::encode(group_id);
        let body = json!({ "paymentMethodId": payment_method_id });
        self.http
            .request_json(
                &format!("/billing/wallet/paymentMethod/{}", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Remove a payment method from a group's wallet.
    ///
    /// **DELETE** `/billing/wallet/paymentMethod/:groupId/:paymentMethodId`
    pub async fn remove_payment_method(
        &self,
        group_id: &str,
        payment_method_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(group_id)),
            ("paymentMethodId", Some(payment_method_id)),
        ])?;
        let safe_gid = urlencoding::encode(group_id);
        let safe_pmid = urlencoding::encode(payment_method_id);
        self.http
            .request_json(
                &format!("/billing/wallet/paymentMethod/{}/{}", safe_gid, safe_pmid),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Get invoice details with optional filters.
    ///
    /// **GET** `/billing/invoiceDetails?...`
    pub async fn get_invoice_details(
        &self,
        group_id: Option<&str>,
        subgroup_id: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", group_id.map(String::from)),
            ("subgroupId", subgroup_id.map(String::from)),
            ("startDate", start_date.map(String::from)),
            ("endDate", end_date.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/billing/invoiceDetails{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get the fee schedule for a group.
    /// When the group is on a custom volume-tiered plan, the response also includes volumePricing: tiers, the current period's dates, count and minimum, ramp, and current rate (rates in microdollars per segment).
    ///
    /// **GET** `/billing/fees/:groupId`
    pub async fn get_fees(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/billing/fees/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }
}
