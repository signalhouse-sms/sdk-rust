use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Landings domain -- brand/campaign landing page management.
pub struct LandingsDomain {
    http: Arc<HttpClient>,
}

impl LandingsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a landing page by its ID.
    ///
    /// **GET** `/landing/:landingId`
    pub async fn get_landings(
        &self,
        landing_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("landingId", Some(landing_id))])?;
        let safe = urlencoding::encode(landing_id);
        self.http
            .request_json(&format!("/landing/{}", safe), Method::GET, None, options)
            .await
    }

    /// Get a public landing page by its ID.
    ///
    /// **GET** `/landing/public/:landingId`
    pub async fn get_public_landing(
        &self,
        landing_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("landingId", Some(landing_id))])?;
        let safe = urlencoding::encode(landing_id);
        self.http
            .request_json(
                &format!("/landing/public/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a new landing page (multipart).
    ///
    /// **POST** `/landing`
    ///
    /// - `landing_data`: JSON object with landing page fields (brandId, description, colours, etc.).
    ///   Optionally set `registrationType` to `"TOLL_FREE"` and supply `tollFree`
    ///   `{useCases, channels, consentDisclosure, consentBoxes[{useCase, channel, text}]}` to build a
    ///   Toll-Free page, which renders one consent checkbox per use case per channel. Set
    ///   `useBrandTemplate` to `true` (Toll-Free only) to build the page from the brand's landing page
    ///   template, taking its colours, logo and webhook URL from there; no logo file is then required.
    /// - `file`: Optional logo image bytes.
    /// - `file_name`: File name for the uploaded logo (required when `file` is provided).
    pub async fn create_landing(
        &self,
        landing_data: &Value,
        file: Option<Vec<u8>>,
        file_name: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let form = Self::build_landing_form(landing_data, file, file_name);
        self.http
            .request_multipart("/landing", Method::POST, form, options)
            .await
    }

    /// Update an existing landing page (multipart).
    ///
    /// **PUT** `/landing/:landingId`
    ///
    /// Set `useBrandTemplate` to `true` (Toll-Free only) to re-copy the brand's landing page template
    /// into this page rather than creating a second one.
    pub async fn update_landing(
        &self,
        landing_id: &str,
        landing_data: &Value,
        file: Option<Vec<u8>>,
        file_name: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("landingId", Some(landing_id))])?;
        let safe = urlencoding::encode(landing_id);
        let form = Self::build_landing_form(landing_data, file, file_name);
        self.http
            .request_multipart(&format!("/landing/{}", safe), Method::PUT, form, options)
            .await
    }

    /// Get a brand's Toll-Free landing page template — the reusable look every page the brand
    /// generates snapshots. Returns an empty array when the brand has no template.
    ///
    /// **GET** `/landing/template/:brandId`
    pub async fn get_landing_template(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(
                &format!("/landing/template/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create or replace a brand's Toll-Free landing page template (multipart). Editing a template
    /// never alters a page that already exists, so a URL already given to a carrier keeps rendering
    /// what was declared.
    ///
    /// **PUT** `/landing/template/:brandId`
    ///
    /// - `template_data`: JSON object with `userDescription` (the customer's own sentence, not the
    ///   finished About paragraph — the services clause is added per campaign), the four colours and
    ///   `webhookUrl`.
    /// - `file`: Logo image bytes. Required the first time only; omit it on a later save to keep the
    ///   stored logo.
    /// - `file_name`: File name for the uploaded logo (required when `file` is provided).
    pub async fn upsert_landing_template(
        &self,
        brand_id: &str,
        template_data: &Value,
        file: Option<Vec<u8>>,
        file_name: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        let form = Self::build_landing_form(template_data, file, file_name);
        self.http
            .request_multipart(
                &format!("/landing/template/{}", safe),
                Method::PUT,
                form,
                options,
            )
            .await
    }

    /// Get a landing page by its associated brand ID.
    ///
    /// **GET** `/landing/brand/:brandId`
    pub async fn get_landing_by_brand_id(
        &self,
        brand_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("brandId", Some(brand_id))])?;
        let safe = urlencoding::encode(brand_id);
        self.http
            .request_json(
                &format!("/landing/brand/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Delete a landing page.
    ///
    /// **DELETE** `/landing/:landingId`
    pub async fn delete_landing(
        &self,
        landing_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("landingId", Some(landing_id))])?;
        let safe = urlencoding::encode(landing_id);
        self.http
            .request_json(&format!("/landing/{}", safe), Method::DELETE, None, options)
            .await
    }

    // -- helpers ------------------------------------------------------------

    fn build_landing_form(
        landing_data: &Value,
        file: Option<Vec<u8>>,
        file_name: Option<&str>,
    ) -> Form {
        let mut form = Form::new();

        if let Some(bytes) = file {
            let name = file_name.unwrap_or("file.png").to_string();
            let part = Part::bytes(bytes).file_name(name);
            form = form.part("file", part);
        }

        if let Value::Object(map) = landing_data {
            for (key, value) in map {
                let text = match value {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                form = form.text(key.clone(), text);
            }
        }

        form
    }
}
