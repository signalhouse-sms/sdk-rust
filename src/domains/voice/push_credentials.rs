use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Mobile push credentials — the Apple APNs auth key and Firebase service
/// account the Voice service uses to wake your mobile app for incoming calls.
///
/// One credential per app. Secrets are encrypted and never returned; responses
/// carry a `fingerprint` instead. Each credential's metadata is
/// `{ id, provider, name, appId, keyId, teamId, bundleId, projectId,
/// clientEmail, fingerprint, createdAt, updatedAt }`.
///
/// Routes are mounted under `/voice/api/v1/push-credentials` on the Voice
/// service. Accessed via `client.voice.push_credentials`.
pub struct PushCredentialsDomain {
    http: Arc<HttpClient>,
}

impl PushCredentialsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// List the account's push credentials (metadata only). Returns
    /// `{ success, data: [{ ...credential }] }`.
    ///
    /// **GET** `/voice/api/v1/push-credentials`
    pub async fn list(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/voice/api/v1/push-credentials", Method::GET, None, options)
            .await
    }

    /// Get one push credential (metadata only). Returns
    /// `{ success, data: { ...credential } }`.
    ///
    /// **GET** `/voice/api/v1/push-credentials/:id`
    pub async fn get(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/push-credentials/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Upload push credentials for one app: an Apple APNs auth key, or a
    /// Firebase service account. The key is checked on upload.
    ///
    /// `credential_data` keys (camelCase): `provider` (required; `"apns"` for
    /// iOS or `"fcm"` for Android) and `name` (optional label, up to 100
    /// characters). For `"apns"` also send `keyId` (the 10-character key ID of
    /// the `.p8` auth key), `teamId` (your 10-character Apple developer team
    /// ID), `bundleId` (your app's bundle ID; VoIP pushes go to its `.voip`
    /// topic) and `privateKey` (the contents of the `.p8` file). For `"fcm"`
    /// also send `serviceAccount` (the Firebase service account key file, as
    /// its JSON text or the parsed object). Returns
    /// `{ success, data: { ...credential } }` (201).
    ///
    /// **POST** `/voice/api/v1/push-credentials`
    pub async fn create(
        &self,
        credential_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/api/v1/push-credentials",
                Method::POST,
                Some(credential_data),
                options,
            )
            .await
    }

    /// Replace a credential's key or service account (and name).
    ///
    /// `credential_data` takes the same keys as [`PushCredentialsDomain::create`];
    /// the provider cannot change. Registered devices keep working with the new
    /// key. Returns `{ success, data: { ...credential } }`.
    ///
    /// **PUT** `/voice/api/v1/push-credentials/:id`
    pub async fn update(
        &self,
        id: &str,
        credential_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/push-credentials/{}", safe),
                Method::PUT,
                Some(credential_data),
                options,
            )
            .await
    }

    /// Delete a credential and every device registered for that app; their
    /// calls stop being pushed. Returns `{ success }`.
    ///
    /// **DELETE** `/voice/api/v1/push-credentials/:id`
    pub async fn delete(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/push-credentials/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }
}
