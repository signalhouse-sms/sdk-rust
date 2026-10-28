use std::sync::Arc;

use reqwest::Method;
use serde_json::json;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Authentication domain -- login, password reset, and auth history.
pub struct AuthDomain {
    http: Arc<HttpClient>,
}

impl AuthDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Login with email and password.
    ///
    /// **POST** `/auth`
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = json!({ "email": email, "password": password });
        self.http
            .request_json("/auth", Method::POST, Some(&body), options)
            .await
    }

    /// Reset a user's password.
    ///
    /// **PUT** `/auth/resetpassword/:userId`
    pub async fn reset_password(
        &self,
        user_id: &str,
        new_password: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("userId", Some(user_id)),
            ("newPassword", Some(new_password)),
        ])?;
        let safe = urlencoding::encode(user_id);
        let body = json!({ "newPassword": new_password });
        self.http
            .request_json(
                &format!("/auth/resetpassword/{}", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Request a password-reset link be emailed to the given address (public).
    ///
    /// Always succeeds regardless of whether an account exists for the email, so it cannot
    /// be used to enumerate registered emails.
    ///
    /// **POST** `/auth/forgot-password`
    pub async fn forgot_password(
        &self,
        email: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("email", Some(email))])?;
        let body = json!({ "email": email });
        self.http
            .request_json("/auth/forgot-password", Method::POST, Some(&body), options)
            .await
    }

    /// Reset a password using the single-use token from a reset email (public).
    ///
    /// **POST** `/auth/reset-password`
    pub async fn reset_password_with_token(
        &self,
        token: &str,
        password: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("token", Some(token)), ("password", Some(password))])?;
        let body = json!({ "token": token, "password": password });
        self.http
            .request_json("/auth/reset-password", Method::POST, Some(&body), options)
            .await
    }

    /// Get token login history for a group or user.
    ///
    /// **GET** `/auth/history?...`
    pub async fn get_auth_history(
        &self,
        group_id: Option<&str>,
        user_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let either = group_id.or(user_id);
        HttpClient::require(&[("groupId or userId", either)])?;

        let qs = HttpClient::get_query_string(&[
            ("groupId", group_id.map(String::from)),
            ("userId", user_id.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(&format!("/auth/history{}", qs), Method::GET, None, options)
            .await
    }

    /// Log out all other users in the caller's active group.
    ///
    /// **POST** `/auth/logout-all`
    pub async fn logout_all(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/logout-all", Method::POST, None, options)
            .await
    }

    /// Get the Group ID associated with the caller's JWT (their active group).
    ///
    /// **GET** `/auth/group-id`
    pub async fn get_group_id(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/auth/group-id", Method::GET, None, options)
            .await
    }

    /// Mint a single-use, short-lived external-link token for the authenticated caller.
    ///
    /// The token is handed to the GHL/Shopify backend so it can link the caller's existing
    /// V2 group to its tenant.
    ///
    /// **POST** `/auth/external-link-token`
    pub async fn request_external_link_token(
        &self,
        product: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("product", Some(product))])?;
        let body = json!({ "product": product });
        self.http
            .request_json(
                "/auth/external-link-token",
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }
}
