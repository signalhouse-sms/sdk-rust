use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Notifications domain -- in-app notification retrieval and management.
pub struct NotificationsDomain {
    http: Arc<HttpClient>,
}

impl NotificationsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get notifications by id or by groupId with optional filters.
    ///
    /// **GET** `/notification?...`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_notifications(
        &self,
        id: Option<&str>,
        group_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        status: Option<&str>,
        event_types: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let either = id.or(group_id);
        HttpClient::require(&[("id or groupId", either)])?;

        let qs = HttpClient::get_query_string(&[
            ("id", id.map(String::from)),
            ("groupId", group_id.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
            ("status", status.map(String::from)),
            ("eventTypes", event_types.map(String::from)),
        ]);
        self.http
            .request_json(&format!("/notification{}", qs), Method::GET, None, options)
            .await
    }

    /// Update the status of one or more notifications.
    ///
    /// **PUT** `/notification/status`
    ///
    /// `ids` may be a single string or a JSON array of strings.
    pub async fn update_notification_status(
        &self,
        ids: &Value,
        status: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("status", Some(status))])?;
        let body = json!({ "ids": ids, "status": status });
        self.http
            .request_json("/notification/status", Method::PUT, Some(&body), options)
            .await
    }

    /// Delete a notification by id.
    ///
    /// **DELETE** `/notification/:id`
    pub async fn delete_notification(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/notification/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }
}
