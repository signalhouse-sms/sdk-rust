use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Users domain -- user CRUD, service users, and notification preferences.
pub struct UsersDomain {
    http: Arc<HttpClient>,
}

impl UsersDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a list of users with optional filters.
    ///
    /// **GET** `/user?...`
    pub async fn get_users(
        &self,
        id: Option<&str>,
        group_id: Option<&str>,
        user_type: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("id", id.map(String::from)),
            ("groupId", group_id.map(String::from)),
            ("userType", user_type.map(String::from)),
        ]);
        self.http
            .request_json(&format!("/user{}", qs), Method::GET, None, options)
            .await
    }

    /// Create a new user.
    ///
    /// **POST** `/user`
    pub async fn create_user(
        &self,
        data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/user", Method::POST, Some(data), options)
            .await
    }

    /// Create a new service user (API key).
    ///
    /// Customer API keys (role `"api"`) may set `allowedIps` (IPv4/IPv6 addresses or CIDR ranges,
    /// max 100) and `allowedCountries` (ISO 3166-1 alpha-2 codes such as `"US"`, max 250); both must
    /// match when both are set. `allowedDestinationCountries` (ISO 3166-1 alpha-2, max 250) limits the
    /// countries the key can send messages to.
    ///
    /// **POST** `/user/serviceuser`
    pub async fn create_service_user(
        &self,
        data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/user/serviceuser", Method::POST, Some(data), options)
            .await
    }

    /// Update an existing user.
    ///
    /// For a customer API key, `allowedIps` / `allowedCountries` / `allowedDestinationCountries` replace
    /// its restrictions (an empty array removes one); this needs an interactive admin or developer
    /// session, so an API key cannot change its own.
    ///
    /// **PUT** `/user/:id`
    pub async fn update_user(
        &self,
        id: &str,
        data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(&format!("/user/{}", safe), Method::PUT, Some(data), options)
            .await
    }

    /// Delete a user (mark as inactive).
    ///
    /// **DELETE** `/user/:id`
    pub async fn delete_user(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(&format!("/user/{}", safe), Method::DELETE, None, options)
            .await
    }

    /// Get notification preferences for a user.
    ///
    /// **GET** `/user/:id/notification-preferences`
    pub async fn get_notification_preferences(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/user/{}/notification-preferences", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Update notification preferences for a user.
    ///
    /// **PUT** `/user/:id/notification-preferences`
    pub async fn update_notification_preferences(
        &self,
        id: &str,
        notification_preferences: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = json!({ "notificationPreferences": notification_preferences });
        self.http
            .request_json(
                &format!("/user/{}/notification-preferences", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    // -- Admin methods (require "admin" feature) ----------------------------

    /// Get users by email or user type (admin view).
    ///
    /// **GET** `/user?...`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn get_users_admin(
        &self,
        email: Option<&str>,
        user_type: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("email", email.map(String::from)),
            ("userType", user_type.map(String::from)),
        ]);
        self.http
            .request_json(&format!("/user{}", qs), Method::GET, None, options)
            .await
    }

    /// Create a new internal (SignalHouse staff) user.
    ///
    /// **POST** `/user/signalhouse`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn create_internal_user(
        &self,
        data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/user/signalhouse", Method::POST, Some(data), options)
            .await
    }
}
