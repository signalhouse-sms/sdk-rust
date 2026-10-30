use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Groups domain -- group retrieval and management.
pub struct GroupsDomain {
    http: Arc<HttpClient>,
}

impl GroupsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get resource counts grouped by group ID. Only signalhouse_admin can query all groups;
    /// every other allowed role is restricted to its JWT group scope.
    ///
    /// **GET** `/group/counts?...`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn get_group_counts(
        &self,
        group_id: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", group_id.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
            ("offset", offset.map(|value| value.to_string())),
        ]);
        self.http
            .request_json(&format!("/group/counts{}", qs), Method::GET, None, options)
            .await
    }

    /// Get a group by its ID.
    ///
    /// **GET** `/group?id=...`
    pub async fn get_group(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[("id", Some(id.to_string()))]);
        self.http
            .request_json(&format!("/group{}", qs), Method::GET, None, options)
            .await
    }

    /// Update a group. `group_data` may include optional CNP fields: `cspId` and `defaultCnpSubgroupId` (string or null).
    ///
    /// **PUT** `/group/:id`
    pub async fn update_group(
        &self,
        id: &str,
        group_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/group/{}", safe),
                Method::PUT,
                Some(group_data),
                options,
            )
            .await
    }

    /// Get a paginated list of all groups.
    ///
    /// **GET** `/group?...`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn get_groups(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(&format!("/group{}", qs), Method::GET, None, options)
            .await
    }

    /// Create a new group.
    ///
    /// `group_data` optionally accepts `psPartnerKey` and `ownerEmail` to credit a referring partner.
    ///
    /// **POST** `/group`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn create_group(
        &self,
        group_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/group", Method::POST, Some(group_data), options)
            .await
    }

    /// Delete a group.
    ///
    /// **DELETE** `/group/:groupId`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn delete_group(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let safe = urlencoding::encode(group_id);
        self.http
            .request_json(&format!("/group/{}", safe), Method::DELETE, None, options)
            .await
    }

    /// Set a group's KYC compliance status. Only flips the field -- it does not
    /// trigger any lockout side effects (halted sends, revoked API keys, etc.) for "rejected".
    ///
    /// **PUT** `/group/:groupId/compliance-status`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn update_compliance_status(
        &self,
        group_id: &str,
        compliance_status: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(group_id)),
            ("complianceStatus", Some(compliance_status)),
        ])?;
        let safe = urlencoding::encode(group_id);
        let body = json!({ "complianceStatus": compliance_status });
        self.http
            .request_json(
                &format!("/group/{}/compliance-status", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Link an external tenant (GHL/Shopify) to a V2 group (server-to-server).
    ///
    /// Exchanges a single-use link token for a canonical group, adopting an empty portal
    /// group, repointing to an existing group, or flagging for manual review.
    ///
    /// **POST** `/group/link-external`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn link_external(
        &self,
        link_token: &str,
        external_system: &str,
        external_id: &str,
        existing_group_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("linkToken", Some(link_token)),
            ("externalSystem", Some(external_system)),
            ("externalId", Some(external_id)),
        ])?;
        let mut body = json!({
            "linkToken": link_token,
            "externalSystem": external_system,
            "externalId": external_id,
        });
        if let Some(existing) = existing_group_id {
            body["existingGroupId"] = Value::String(existing.to_string());
        }
        self.http
            .request_json("/group/link-external", Method::POST, Some(&body), options)
            .await
    }

    /// Get a group's Enterprise SSO configuration: whether SSO is available, the redirect URL to
    /// register at the identity provider, the saved provider (never its client secret), and each
    /// claimed domain with the DNS TXT record that verifies it.
    ///
    /// Returns `{ groupId, available, callbackUrl, connection, domains }`.
    ///
    /// **GET** `/group/sso?groupId=...`
    pub async fn get_sso_configuration(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[("groupId", Some(group_id.to_string()))]);
        self.http
            .request_json(&format!("/group/sso{}", qs), Method::GET, None, options)
            .await
    }

    /// Create or update a group's OpenID Connect identity provider.
    ///
    /// `issuer` is the provider's https issuer URL and `client_id` the client ID registered at
    /// the provider. `client_secret` is required the first time; pass `None` to keep the saved
    /// one. `enabled` controls whether sign-in through the provider is on (server default true).
    /// Requires a fresh `auth.confirm_two_factor_identity` from the same session. Returns the SSO
    /// configuration.
    ///
    /// **PUT** `/group/sso`
    pub async fn save_sso_connection(
        &self,
        group_id: &str,
        issuer: &str,
        client_id: &str,
        client_secret: Option<&str>,
        enabled: Option<bool>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("groupId", Some(group_id)),
            ("issuer", Some(issuer)),
            ("clientId", Some(client_id)),
        ])?;
        let mut body = json!({
            "groupId": group_id,
            "issuer": issuer,
            "clientId": client_id,
        });
        if let Some(secret) = client_secret {
            body["clientSecret"] = json!(secret);
        }
        if let Some(on) = enabled {
            body["enabled"] = json!(on);
        }
        self.http
            .request_json("/group/sso", Method::PUT, Some(&body), options)
            .await
    }

    /// Remove a group's identity provider and its domains, and unlink every account that signed
    /// in through it. Requires a fresh `auth.confirm_two_factor_identity` from the same session.
    /// Returns the (now empty) SSO configuration.
    ///
    /// **DELETE** `/group/sso?groupId=...`
    pub async fn delete_sso_connection(
        &self,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[("groupId", Some(group_id.to_string()))]);
        self.http
            .request_json(&format!("/group/sso{}", qs), Method::DELETE, None, options)
            .await
    }

    /// Claim an email domain (e.g. "acme.com") for a group's SSO. The response (the SSO
    /// configuration) includes the DNS TXT record that verifies it.
    ///
    /// **POST** `/group/sso/domains`
    pub async fn add_sso_domain(
        &self,
        group_id: &str,
        domain: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id)), ("domain", Some(domain))])?;
        let body = json!({ "groupId": group_id, "domain": domain });
        self.http
            .request_json("/group/sso/domains", Method::POST, Some(&body), options)
            .await
    }

    /// Verify a claimed domain by its DNS TXT record. Returns the SSO configuration.
    ///
    /// **POST** `/group/sso/domains/:domain/verify`
    pub async fn verify_sso_domain(
        &self,
        group_id: &str,
        domain: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id)), ("domain", Some(domain))])?;
        let safe = urlencoding::encode(domain);
        let body = json!({ "groupId": group_id });
        self.http
            .request_json(
                &format!("/group/sso/domains/{}/verify", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Remove a domain from a group's SSO. Returns the SSO configuration.
    ///
    /// **DELETE** `/group/sso/domains/:domain?groupId=...`
    pub async fn remove_sso_domain(
        &self,
        group_id: &str,
        domain: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id)), ("domain", Some(domain))])?;
        let safe = urlencoding::encode(domain);
        let qs = HttpClient::get_query_string(&[("groupId", Some(group_id.to_string()))]);
        self.http
            .request_json(
                &format!("/group/sso/domains/{}{}", safe, qs),
                Method::DELETE,
                None,
                options,
            )
            .await
    }
}
