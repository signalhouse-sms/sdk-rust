use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Webhooks domain -- webhook CRUD operations.
pub struct WebhooksDomain {
    http: Arc<HttpClient>,
}

impl WebhooksDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a list of webhooks with optional filters. Each returned endpoint carries
    /// `hasSigningSecret` (bool) instead of the signing secret itself — see `create_webhook`.
    ///
    /// **GET** `/webhook?...`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_webhooks(
        &self,
        id: Option<&str>,
        group_id: Option<&str>,
        endpoint_type: Option<&str>,
        phone_number: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("id", id.map(String::from)),
            ("groupId", group_id.map(String::from)),
            ("endpointType", endpoint_type.map(String::from)),
            ("phoneNumber", phone_number.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(&format!("/webhook{}", qs), Method::GET, None, options)
            .await
    }

    /// Create a new webhook. The response includes a `signingSecret` (plaintext HMAC-SHA256 secret)
    /// used to sign every delivery to this endpoint via the X-SignalHouse-Signature header. It is
    /// returned exactly once, here — store it now, it is never exposed again by any read.
    ///
    /// `authType` in `webhook_data` is one of `NONE` (default), `BASIC`, `API_KEY`, `OAUTH2` or `HMAC`.
    /// `HMAC` sends no credentials (the receiver authenticates by the signature alone) and must be sent
    /// without `credentials` or `apiHeaderPrefix`.
    ///
    /// `eventSettings` in `webhook_data` holds per-event options keyed by event name:
    /// `NUMBER_HEALTH_BELOW_THRESHOLD` and `CAMPAIGN_HEALTH_BELOW_THRESHOLD` each take `{ "threshold": n }`,
    /// the 7-day health score (1.5–10, in 0.5 steps) below which the event fires. A subscribed health event
    /// without a threshold is stored with the default, 6.
    ///
    /// **POST** `/webhook`
    pub async fn create_webhook(
        &self,
        webhook_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/webhook", Method::POST, Some(webhook_data), options)
            .await
    }

    /// Update an existing webhook. Like every other read of a webhook, the response carries
    /// `hasSigningSecret` (bool), never the signing secret itself — see `create_webhook`.
    ///
    /// Switching `authType` to `NONE` or `HMAC` clears the stored credentials. `HMAC` is rejected for an
    /// endpoint with no signing secret; call `rotate_webhook_secret` first.
    ///
    /// `eventSettings` in `update_data` changes per-event options by event name (`NUMBER_HEALTH_BELOW_THRESHOLD`,
    /// `CAMPAIGN_HEALTH_BELOW_THRESHOLD`, each `{ "threshold": n }`, 1.5–10 in 0.5 steps); events you omit keep
    /// their stored settings.
    ///
    /// **PUT** `/webhook/:id`
    pub async fn update_webhook(
        &self,
        id: &str,
        update_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/webhook/{}", safe),
                Method::PUT,
                Some(update_data),
                options,
            )
            .await
    }

    /// Rotate a webhook's signing secret, or give a webhook created before signing its first one.
    ///
    /// The response includes the new plaintext `signingSecret`, returned this one time only. The
    /// replaced secret keeps working for `grace_seconds` (default 86400, max 604800, 0 = stop now).
    ///
    /// **POST** `/webhook/:id/rotate-secret`
    pub async fn rotate_webhook_secret(
        &self,
        id: &str,
        grace_seconds: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = match grace_seconds {
            Some(grace) => serde_json::json!({ "graceSeconds": grace }),
            None => serde_json::json!({}),
        };
        self.http
            .request_json(
                &format!("/webhook/{}/rotate-secret", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Delete a webhook (mark as inactive).
    ///
    /// **DELETE** `/webhook/:id`
    pub async fn delete_webhook(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(&format!("/webhook/{}", safe), Method::DELETE, None, options)
            .await
    }
}
