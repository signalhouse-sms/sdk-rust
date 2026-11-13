use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Map, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Verify domain -- carrier phone-number checks on `/verify`: SIM Swap and Number Verify.
///
/// Every check the carrier answers is billed to the group's wallet (402 on insufficient funds);
/// a check the carrier cannot run for the number is a 422 and is not charged.
pub struct VerifyDomain {
    http: Arc<HttpClient>,
}

impl VerifyDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Check whether a number's SIM card changed recently, as reported by its carrier.
    ///
    /// `phone_number` is in international format (e.g. `+12025550123`). `max_age` is the lookback
    /// window in hours, 1-2400 (`None` uses the carrier's default); `include_swap_date` also returns
    /// `latestSimChange` at no extra charge (`None` uses the API default, false). Billed
    /// once when the check answers; 422 when the check is not available for the number (not
    /// charged). Returns `{ checkId, phoneNumber, swapped, maxAge, latestSimChange?, cost }`.
    /// Roles: api, admin, developer.
    ///
    /// **POST** `/verify/sim-swap`
    pub async fn check_sim_swap(
        &self,
        phone_number: &str,
        max_age: Option<u32>,
        include_swap_date: Option<bool>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;
        let mut body = Map::new();
        body.insert("phoneNumber".into(), json!(phone_number));
        if let Some(value) = max_age {
            body.insert("maxAge".into(), json!(value));
        }
        if let Some(value) = include_swap_date {
            body.insert("includeSwapDate".into(), json!(value));
        }
        let body = Value::Object(body);
        self.http
            .request_json("/verify/sim-swap", Method::POST, Some(&body), options)
            .await
    }

    /// Start a Number Verify check.
    ///
    /// The end user's phone must open the returned `authorizationUrl` on mobile data (Wi-Fi off)
    /// within 10 minutes; read the result with `get_number_verification`. `device_phone_number` is
    /// the phone's own number, sent to the carrier as a login hint; `redirect_url` is an https URL
    /// the end user's browser is sent to after the check, with `?checkId=...`. Billed only when the
    /// check is answered (`VERIFIED` or `NOT_VERIFIED`). Returns `{ checkId, phoneNumber, status:
    /// "PENDING", authorizationUrl, expiresAt }`. Roles: api, admin, developer.
    ///
    /// **POST** `/verify/number`
    pub async fn start_number_verification(
        &self,
        phone_number: &str,
        device_phone_number: Option<&str>,
        redirect_url: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("phoneNumber", Some(phone_number))])?;
        let mut body = Map::new();
        body.insert("phoneNumber".into(), json!(phone_number));
        if let Some(value) = device_phone_number {
            body.insert("devicePhoneNumber".into(), json!(value));
        }
        if let Some(value) = redirect_url {
            body.insert("redirectUrl".into(), json!(value));
        }
        let body = Value::Object(body);
        self.http
            .request_json("/verify/number", Method::POST, Some(&body), options)
            .await
    }

    /// Get a Number Verify check's status and result. A `PENDING` check past its `expiresAt` reads
    /// as `EXPIRED`.
    ///
    /// Returns `{ checkId, phoneNumber, status, devicePhoneNumberVerified, failureReason, createdAt,
    /// expiresAt, completedAt, cost }`. Roles: api, admin, developer, billing, user.
    ///
    /// **GET** `/verify/number/:checkId`
    pub async fn get_number_verification(
        &self,
        check_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("checkId", Some(check_id))])?;
        let path = format!("/verify/number/{}", urlencoding::encode(check_id));
        self.http
            .request_json(&path, Method::GET, None, options)
            .await
    }
}
