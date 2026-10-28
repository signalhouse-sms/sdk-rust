use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Subscriptions domain -- subscription plans, templates, and billing cycles.
pub struct SubscriptionsDomain {
    http: Arc<HttpClient>,
}

impl SubscriptionsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get subscription history for a group.
    ///
    /// **GET** `/subscription/user?...`
    pub async fn get_subscriptions(
        &self,
        group_id: &str,
        only_active: Option<bool>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("onlyActive", only_active.map(|b| b.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/subscription/user{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Subscribe a group to a template.
    ///
    /// **POST** `/subscription/user/subscribe/:groupId/:templateId`
    pub async fn subscribe(
        &self,
        group_id: &str,
        template_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(group_id)),
            ("templateId", Some(template_id)),
        ])?;
        let safe_gid = urlencoding::encode(group_id);
        let safe_tid = urlencoding::encode(template_id);
        let body = json!({});
        self.http
            .request_json(
                &format!("/subscription/user/subscribe/{}/{}", safe_gid, safe_tid),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Unsubscribe (cancel) a group's active subscription.
    ///
    /// **POST** `/subscription/user/cancel/:groupId`
    pub async fn unsubscribe(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/subscription/user/cancel/{}", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    // -- Admin methods (require "admin" feature) ----------------------------

    /// Get subscription templates, optionally filtered by template ID.
    ///
    /// **GET** `/subscription/template?...`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn get_templates(
        &self,
        template_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[("templateId", template_id.map(String::from))]);
        self.http
            .request_json(
                &format!("/subscription/template{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a new subscription template.
    ///
    /// **POST** `/subscription/template`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn create_template(
        &self,
        template_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/subscription/template",
                Method::POST,
                Some(template_data),
                options,
            )
            .await
    }

    /// Update an existing subscription template.
    ///
    /// **PUT** `/subscription/template/:templateId`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn update_template(
        &self,
        template_id: &str,
        template_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("templateId", Some(template_id))])?;
        let safe = urlencoding::encode(template_id);
        self.http
            .request_json(
                &format!("/subscription/template/{}", safe),
                Method::PUT,
                Some(template_data),
                options,
            )
            .await
    }

    /// Delete a subscription template.
    ///
    /// **DELETE** `/subscription/template/:templateId`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn delete_template(
        &self,
        template_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("templateId", Some(template_id))])?;
        let safe = urlencoding::encode(template_id);
        self.http
            .request_json(
                &format!("/subscription/template/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Create a custom subscription for a group.
    ///
    /// **POST** `/subscription/user/custom/:groupId`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn create_custom_subscription(
        &self,
        group_id: &str,
        subscription_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/subscription/user/custom/{}", safe),
                Method::POST,
                Some(subscription_data),
                options,
            )
            .await
    }

    /// Update volume tiers, ramp, and monthly minimum on an active custom subscription.
    ///
    /// **PUT** `/subscription/user/custom/:groupId`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn update_custom_subscription(
        &self,
        group_id: &str,
        update_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(
                &format!("/subscription/user/custom/{}", safe),
                Method::PUT,
                Some(update_data),
                options,
            )
            .await
    }
}
