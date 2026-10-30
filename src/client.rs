use std::sync::Arc;

use reqwest::multipart::Form;
use serde_json::Value;

use crate::domains::agents::AgentsDomain;
use crate::domains::auth::AuthDomain;
use crate::domains::billing::BillingDomain;
use crate::domains::brands::BrandsDomain;
use crate::domains::campaigns::CampaignsDomain;
use crate::domains::groups::GroupsDomain;
use crate::domains::landings::LandingsDomain;
use crate::domains::messages::MessagesDomain;
use crate::domains::notifications::NotificationsDomain;
use crate::domains::numbers::NumbersDomain;
use crate::domains::onboarding::OnboardingDomain;
use crate::domains::registrations::RegistrationsDomain;
use crate::domains::shortlinks::ShortlinksDomain;
use crate::domains::subgroups::SubgroupsDomain;
use crate::domains::subscriptions::SubscriptionsDomain;
use crate::domains::templates::TemplatesDomain;
use crate::domains::tickets::TicketsDomain;
use crate::domains::users::UsersDomain;
use crate::domains::voice::VoiceDomain;
use crate::domains::webhooks::WebhooksDomain;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

// ---------------------------------------------------------------------------
// Internal HTTP helper shared across all domain structs via Arc
// ---------------------------------------------------------------------------

pub(crate) struct HttpClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl HttpClient {
    /// The API base URL this client was configured with.
    pub(crate) fn base_url(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    /// Send a JSON-body request and return a standardised response.
    pub async fn request_json(
        &self,
        path: &str,
        method: reqwest::Method,
        body: Option<&Value>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let url = format!("{}{}", self.base_url, path);
        let token = options
            .and_then(|o| o.token.as_deref())
            .unwrap_or(&self.api_key);

        let mut builder = self
            .client
            .request(method, &url)
            .bearer_auth(token)
            .header("Content-Type", "application/json");

        if let Some(opts) = options {
            if let Some(ref hdrs) = opts.headers {
                for (k, v) in hdrs {
                    builder = builder.header(k.as_str(), v.as_str());
                }
            }
            if let Some(ref key) = opts.idempotency_key {
                builder = builder.header("Idempotency-Key", key.as_str());
            }
        }

        if let Some(b) = body {
            builder = builder.json(b);
        }

        let resp = builder.send().await?;
        Self::parse_response(resp).await
    }

    /// Send a multipart/form-data request and return a standardised response.
    pub async fn request_multipart(
        &self,
        path: &str,
        method: reqwest::Method,
        form: Form,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let url = format!("{}{}", self.base_url, path);
        let token = options
            .and_then(|o| o.token.as_deref())
            .unwrap_or(&self.api_key);

        let mut builder = self
            .client
            .request(method, &url)
            .bearer_auth(token)
            .multipart(form);

        if let Some(opts) = options {
            if let Some(ref hdrs) = opts.headers {
                for (k, v) in hdrs {
                    builder = builder.header(k.as_str(), v.as_str());
                }
            }
            if let Some(ref key) = opts.idempotency_key {
                builder = builder.header("Idempotency-Key", key.as_str());
            }
        }

        let resp = builder.send().await?;
        Self::parse_response(resp).await
    }

    /// Build a query string from a slice of `(key, Option<value>)` pairs.
    /// Keys whose value is `None` are omitted. Uses camelCase names that match
    /// the SignalHouse REST API.
    pub fn get_query_string(params: &[(&str, Option<String>)]) -> String {
        let parts: Vec<String> = params
            .iter()
            .filter_map(|(k, v)| {
                v.as_ref()
                    .map(|val| format!("{}={}", urlencoding::encode(k), urlencoding::encode(val)))
            })
            .collect();

        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }

    /// Validate that every named field has a non-empty value. Returns
    /// `Err(SignalHouseError::Validation)` on the first missing field.
    pub fn require(fields: &[(&str, Option<&str>)]) -> Result<(), SignalHouseError> {
        for (name, value) in fields {
            match value {
                Some(v) if !v.is_empty() => {}
                _ => {
                    return Err(SignalHouseError::Validation(name.to_string()));
                }
            }
        }
        Ok(())
    }

    // -- private helpers ----------------------------------------------------

    async fn parse_response(
        resp: reqwest::Response,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let status = resp.status().as_u16();
        let success = resp.status().is_success();
        let text = resp.text().await?;

        if success {
            let data: Option<Value> = serde_json::from_str(&text).ok();
            Ok(SignalHouseResponse {
                success: true,
                data,
                error: None,
                status,
            })
        } else {
            // Try to extract a `message` field from the JSON body; fall back
            // to the raw text.
            let message = serde_json::from_str::<Value>(&text)
                .ok()
                .and_then(|v| v.get("message").and_then(|m| m.as_str().map(String::from)))
                .unwrap_or(text);

            Ok(SignalHouseResponse {
                success: false,
                data: None,
                error: Some(message),
                status,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Public SDK entry point
// ---------------------------------------------------------------------------

/// Configuration used to initialise a [`SignalHouseClient`].
pub struct SignalHouseConfig {
    /// Your SignalHouse API key (bearer token).
    pub api_key: String,
    /// Base URL of the SignalHouse API (e.g. `https://v2.signalhouse.io`).
    pub base_url: String,
}

/// Top-level SDK client. Exposes every API domain as a public field.
///
/// ```rust,no_run
/// use signalhouse_sms::{SignalHouseClient, SignalHouseConfig};
///
/// #[tokio::main]
/// async fn main() {
///     let client = SignalHouseClient::new(SignalHouseConfig {
///         api_key: "your_api_key".into(),
///         base_url: "https://v2.signalhouse.io".into(),
///     }).unwrap();
///
///     let resp = client.auth.login("user@example.com", "password", None).await;
/// }
/// ```
pub struct SignalHouseClient {
    pub agents: AgentsDomain,
    pub auth: AuthDomain,
    pub billing: BillingDomain,
    pub brands: BrandsDomain,
    pub campaigns: CampaignsDomain,
    pub groups: GroupsDomain,
    pub landings: LandingsDomain,
    pub messages: MessagesDomain,
    pub notifications: NotificationsDomain,
    pub numbers: NumbersDomain,
    pub onboarding: OnboardingDomain,
    pub registrations: RegistrationsDomain,
    pub shortlinks: ShortlinksDomain,
    pub subgroups: SubgroupsDomain,
    pub subscriptions: SubscriptionsDomain,
    /// Versioned message templates on `/template`.
    pub templates: TemplatesDomain,
    pub tickets: TicketsDomain,
    pub users: UsersDomain,
    pub voice: VoiceDomain,
    pub webhooks: WebhooksDomain,
}

impl SignalHouseClient {
    /// Create a new SDK client.
    ///
    /// Returns `Err` if the API key or base URL is empty.
    pub fn new(config: SignalHouseConfig) -> Result<Self, SignalHouseError> {
        if config.api_key.is_empty() {
            return Err(SignalHouseError::Validation(
                "API key is required to initialize SignalHouseClient".into(),
            ));
        }
        if config.base_url.is_empty() {
            return Err(SignalHouseError::Validation(
                "Base URL is required to initialize SignalHouseClient".into(),
            ));
        }

        let http = Arc::new(HttpClient {
            client: reqwest::Client::new(),
            base_url: config.base_url,
            api_key: config.api_key,
        });

        Ok(Self {
            agents: AgentsDomain::new(Arc::clone(&http)),
            auth: AuthDomain::new(Arc::clone(&http)),
            billing: BillingDomain::new(Arc::clone(&http)),
            brands: BrandsDomain::new(Arc::clone(&http)),
            campaigns: CampaignsDomain::new(Arc::clone(&http)),
            groups: GroupsDomain::new(Arc::clone(&http)),
            landings: LandingsDomain::new(Arc::clone(&http)),
            messages: MessagesDomain::new(Arc::clone(&http)),
            notifications: NotificationsDomain::new(Arc::clone(&http)),
            numbers: NumbersDomain::new(Arc::clone(&http)),
            onboarding: OnboardingDomain::new(Arc::clone(&http)),
            registrations: RegistrationsDomain::new(Arc::clone(&http)),
            shortlinks: ShortlinksDomain::new(Arc::clone(&http)),
            subgroups: SubgroupsDomain::new(Arc::clone(&http)),
            subscriptions: SubscriptionsDomain::new(Arc::clone(&http)),
            templates: TemplatesDomain::new(Arc::clone(&http)),
            tickets: TicketsDomain::new(Arc::clone(&http)),
            users: UsersDomain::new(Arc::clone(&http)),
            voice: VoiceDomain::new(Arc::clone(&http)),
            webhooks: WebhooksDomain::new(Arc::clone(&http)),
        })
    }
}
