use std::sync::Arc;

use reqwest::Method;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Shortlinks domain -- URL shortener redirect and details.
pub struct ShortlinksDomain {
    http: Arc<HttpClient>,
}

impl ShortlinksDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get the redirect URL for a shortlink.
    ///
    /// **GET** `/shortlink/redirect/:shortlinkId`
    pub async fn get_shortlink_redirect(
        &self,
        shortlink_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("shortlinkId", Some(shortlink_id))])?;
        let safe = urlencoding::encode(shortlink_id);
        self.http
            .request_json(
                &format!("/shortlink/redirect/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get the recipient opt-out details (`{code, senderName, maskedRecipient, optedOut}`)
    /// behind an alphanumeric-sender opt-out link (`sihou.io/o/{code}`). Public.
    ///
    /// **GET** `/shortlink/optout/:code`
    pub async fn get_opt_out(
        &self,
        code: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("code", Some(code))])?;
        let safe = urlencoding::encode(code);
        self.http
            .request_json(
                &format!("/shortlink/optout/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Confirm the recipient opt-out behind an alphanumeric-sender opt-out link; idempotent. Public.
    ///
    /// **POST** `/shortlink/optout/:code`
    pub async fn confirm_opt_out(
        &self,
        code: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("code", Some(code))])?;
        let safe = urlencoding::encode(code);
        self.http
            .request_json(
                &format!("/shortlink/optout/{}", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Get shortlink details with optional filters.
    ///
    /// **GET** `/shortlink?...`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_shortlink(
        &self,
        shortlink_id: Option<&str>,
        message_id: Option<&str>,
        phone_number: Option<&str>,
        campaign_id: Option<&str>,
        brand_id: Option<&str>,
        subgroup_id: Option<&str>,
        group_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("shortlinkId", shortlink_id.map(String::from)),
            ("messageId", message_id.map(String::from)),
            ("phoneNumber", phone_number.map(String::from)),
            ("campaignId", campaign_id.map(String::from)),
            ("brandId", brand_id.map(String::from)),
            ("subgroupId", subgroup_id.map(String::from)),
            ("groupId", group_id.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(&format!("/shortlink{}", qs), Method::GET, None, options)
            .await
    }
}
