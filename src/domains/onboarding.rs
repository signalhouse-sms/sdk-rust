use std::sync::Arc;

use reqwest::Method;
use serde_json::json;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Onboarding domain — staff-only onboarding/KYC data. All methods require
/// the `admin` crate feature.
pub struct OnboardingDomain {
    http: Arc<HttpClient>,
}

impl OnboardingDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get all onboarding records (one per group). Staff-only.
    ///
    /// **GET** `/group/onboarding`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn get_onboardings(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/group/onboarding", Method::GET, None, options)
            .await
    }

    /// Get a single onboarding record by group ID. Staff-only.
    ///
    /// **GET** `/group/onboarding/:groupId`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn get_onboarding(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/group/onboarding/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Set where a group's new toll-free brands and numbers go (`infobip` or `twilio`). Staff-only.
    /// Existing brands, numbers and campaigns stay where they are.
    ///
    /// **PUT** `/group/onboarding/:groupId/toll-free-provider`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn set_toll_free_provider(
        &self,
        group_id: &str,
        toll_free_provider: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(group_id)),
            ("tollFreeProvider", Some(toll_free_provider)),
        ])?;
        let safe = urlencoding::encode(group_id);
        let body = json!({ "tollFreeProvider": toll_free_provider });
        self.http
            .request_json(
                &format!("/group/onboarding/{}/toll-free-provider", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Add an internal comment/note to a group's admin Pricing editor. Staff-only.
    ///
    /// **POST** `/group/onboarding/:groupId/pricing-comments`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn add_pricing_comment(
        &self,
        group_id: &str,
        comment: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id)), ("comment", Some(comment))])?;
        let safe = urlencoding::encode(group_id);
        let body = json!({ "comment": comment });
        self.http
            .request_json(
                &format!("/group/onboarding/{}/pricing-comments", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Add an internal comment/note to a group's admin Credits page. Staff-only.
    ///
    /// **POST** `/group/onboarding/:groupId/credit-comments`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn add_credit_comment(
        &self,
        group_id: &str,
        comment: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id)), ("comment", Some(comment))])?;
        let safe = urlencoding::encode(group_id);
        let body = json!({ "comment": comment });
        self.http
            .request_json(
                &format!("/group/onboarding/{}/credit-comments", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }
}
