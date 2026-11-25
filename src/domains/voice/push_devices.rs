use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Mobile push devices — your mobile app registers each device so incoming
/// calls wake it.
///
/// These methods are called by the app with the user's **voice token** (the
/// `token` from [`TokensDomain::create`](super::TokensDomain::create), minted
/// with a `subgroupId`) as the bearer token, not an API key: pass it as
/// [`RequestOptions::token`] on every call. The device is tied to that token's
/// identity.
///
/// Routes are mounted under `/voice/api/v1/push-devices` on the Voice service.
/// Accessed via `client.voice.push_devices`.
pub struct PushDevicesDomain {
    http: Arc<HttpClient>,
}

impl PushDevicesDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Register this device for push. Call it after each voice token mint and
    /// whenever the OS gives the app a new push token. A push token already
    /// registered to another user moves to this one.
    ///
    /// `device_data` keys (camelCase): `provider` (required; `"apns"` for iOS,
    /// a PushKit VoIP token, or `"fcm"` for Android, a Firebase registration
    /// token), `deviceToken` (required; the device's push token),
    /// `environment` (optional, apns only; `"production"` (default) or
    /// `"sandbox"` for development builds) and `appId` (optional; the bundle ID
    /// or Firebase project ID, only needed when the account has several apps
    /// for the provider). `options` must carry the voice token as `token`.
    /// Returns
    /// `{ success, data: { id, identity, provider, environment, appId, registeredAt } }`
    /// (201).
    ///
    /// **POST** `/voice/api/v1/push-devices`
    pub async fn register(
        &self,
        device_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/api/v1/push-devices",
                Method::POST,
                Some(device_data),
                options,
            )
            .await
    }

    /// Unregister this device, e.g. when the user signs out, so calls stop
    /// waking it. The device token is percent-encoded into the path (FCM tokens
    /// contain `:`). `options` must carry the voice token as `token`. Returns
    /// `{ success, data: { removed } }`.
    ///
    /// **DELETE** `/voice/api/v1/push-devices/:deviceToken`
    pub async fn unregister(
        &self,
        device_token: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("deviceToken", Some(device_token))])?;
        let safe = urlencoding::encode(device_token);
        self.http
            .request_json(
                &format!("/voice/api/v1/push-devices/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// The pushes recently sent to the voice token's user, newest first (kept
    /// 7 days), for debugging an app. `limit` is 1 to 100 (default 20).
    /// `options` must carry the voice token as `token`. Returns
    /// `{ success, data: [{ id, callUuid, provider, kind, status, error, createdAt }] }`.
    ///
    /// **GET** `/voice/api/v1/push-devices/deliveries`
    pub async fn list_deliveries(
        &self,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[("limit", limit.map(|l| l.to_string()))]);
        self.http
            .request_json(
                &format!("/voice/api/v1/push-devices/deliveries{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }
}
