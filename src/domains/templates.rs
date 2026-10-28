use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Filters for [`TemplatesDomain::list`].
#[derive(Debug, Default)]
pub struct GetTemplatesParams<'a> {
    pub group_id: &'a str,
    /// `DRAFT`, `PUBLISHED` or `ARCHIVED`.
    pub state: Option<&'a str>,
    /// `"rcs"`.
    pub channel: Option<&'a str>,
    pub tag: Option<&'a str>,
    pub search: Option<&'a str>,
    /// `name`, `createdAt` or `updatedAt`.
    pub sort_by: Option<&'a str>,
    /// `asc` or `desc`.
    pub sort_order: Option<&'a str>,
    /// Defaults to 1.
    pub page: Option<u32>,
    /// Defaults to 25, at most 100.
    pub limit: Option<u32>,
}

/// Parameters for [`TemplatesDomain::update`].
#[derive(Debug, Default)]
pub struct UpdateTemplateParams<'a> {
    pub template_id: &'a str,
    pub group_id: &'a str,
    pub name: Option<&'a str>,
    /// `None` leaves it unchanged; `Some(None)` sends `null` to clear it.
    pub tag: Option<Option<&'a str>>,
    /// A full replacement `RcsTemplateBody`, sent verbatim.
    pub body: Option<&'a Value>,
}

/// Templates domain -- versioned message templates on `/template`. Every
/// call is 404 while the API's RCS switch is off.
/// Bodies are the raw `RcsTemplateBody`, sent verbatim. Writes apply directly (last write wins);
/// an archived template is a 409 `TEMPLATE_ARCHIVED` and a taken name a 409 `TEMPLATE_NAME_TAKEN`.
pub struct TemplatesDomain {
    http: Arc<HttpClient>,
}

impl TemplatesDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Create a template with draft version 1; returns `{ head, version }` (HTTP 201).
    ///
    /// **POST** `/template`
    pub async fn create(
        &self,
        group_id: &str,
        name: &str,
        body: &Value,
        tag: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = create_body(group_id, name, body, tag)?;
        self.http
            .request_json("/template", Method::POST, Some(&payload), options)
            .await
    }

    /// List template summaries as `{ data, total, page, limit }`.
    ///
    /// **GET** `/template`
    pub async fn list(
        &self,
        params: GetTemplatesParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = list_query(&params)?;
        self.http
            .request_json(&format!("/template{}", qs), Method::GET, None, options)
            .await
    }

    /// Get `{ head, version }` for one template version.
    ///
    /// **GET** `/template/:templateId`
    ///
    /// `version` is a positive integer, `"draft"` or `"published"`; `None` returns the published
    /// version, else the draft. An explicit version never falls back (404).
    pub async fn get(
        &self,
        template_id: &str,
        group_id: &str,
        version: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = version_query(template_id, group_id, version)?;
        let safe = urlencoding::encode(template_id);
        self.http
            .request_json(
                &format!("/template/{}{}", safe, qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Edit the draft (creating one from the published version if needed); returns `{ head, version }`.
    ///
    /// **PUT** `/template/:templateId`
    pub async fn update(
        &self,
        params: UpdateTemplateParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = update_body(&params)?;
        let safe = urlencoding::encode(params.template_id);
        self.http
            .request_json(
                &format!("/template/{}", safe),
                Method::PUT,
                Some(&payload),
                options,
            )
            .await
    }

    /// Publish the current draft; returns `{ head, version }`. No draft is a 409 `NO_DRAFT`, and a
    /// draft that changed while it was being published is a 409 `DRAFT_CHANGED` (publish again);
    /// an incomplete draft is a 400 `TEMPLATE_INVALID` carrying `errors` and `warnings`.
    ///
    /// **POST** `/template/:templateId/publish`
    pub async fn publish(
        &self,
        template_id: &str,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = publish_body(template_id, group_id)?;
        let safe = urlencoding::encode(template_id);
        self.http
            .request_json(
                &format!("/template/{}/publish", safe),
                Method::POST,
                Some(&payload),
                options,
            )
            .await
    }

    /// Archive a template; returns its head.
    ///
    /// **POST** `/template/:templateId/archive`
    pub async fn archive(
        &self,
        template_id: &str,
        group_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = archive_body(template_id, group_id)?;
        let safe = urlencoding::encode(template_id);
        self.http
            .request_json(
                &format!("/template/{}/archive", safe),
                Method::POST,
                Some(&payload),
                options,
            )
            .await
    }

    /// List a template's version summaries (no bodies) as `{ data, total, page, limit }`.
    ///
    /// **GET** `/template/:templateId/versions`
    pub async fn versions(
        &self,
        template_id: &str,
        group_id: &str,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = versions_query(template_id, group_id, page, limit)?;
        let safe = urlencoding::encode(template_id);
        self.http
            .request_json(
                &format!("/template/{}/versions{}", safe, qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Validate exactly one of `body` or a stored `version` without writing; returns `{ valid,
    /// contentHash, validatorVersion, errors, warnings }`.
    ///
    /// **POST** `/template/:templateId/validate`
    ///
    /// A numeric `version` is sent as a JSON number.
    pub async fn validate(
        &self,
        template_id: &str,
        group_id: &str,
        body: Option<&Value>,
        version: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let payload = validate_body(template_id, group_id, body, version)?;
        let safe = urlencoding::encode(template_id);
        self.http
            .request_json(
                &format!("/template/{}/validate", safe),
                Method::POST,
                Some(&payload),
                options,
            )
            .await
    }

    /// Get the block kinds, action types, limits and media rules the validator enforces.
    ///
    /// **GET** `/template/capabilities`
    ///
    /// `channel` defaults to `"rcs"`; `provider` (`"infobip"` or `"sinch"`) adds provider limits.
    pub async fn capabilities(
        &self,
        group_id: &str,
        channel: Option<&str>,
        provider: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = capabilities_query(group_id, channel, provider)?;
        self.http
            .request_json(
                &format!("/template/capabilities{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Upload one template media file (multipart `file` + `groupId`); returns the MediaAsset (HTTP 201).
    ///
    /// **POST** `/template/media`
    pub async fn upload_media(
        &self,
        group_id: &str,
        file: Vec<u8>,
        file_name: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let form = media_form(group_id, file, file_name)?;
        self.http
            .request_multipart("/template/media", Method::POST, form, options)
            .await
    }
}

// -- helpers ------------------------------------------------------------------

/// Validate the identifiers every per-template call needs.
fn require_template(template_id: &str, group_id: &str) -> Result<(), SignalHouseError> {
    HttpClient::require(&[
        ("templateId", Some(template_id)),
        ("groupId", Some(group_id)),
    ])
}

/// A version selector as JSON: a number when numeric, else the string.
fn version_selector(version: &str) -> Value {
    match version.parse::<u64>() {
        Ok(n) => json!(n),
        Err(_) => json!(version),
    }
}

/// Build a create body.
pub(crate) fn create_body(
    group_id: &str,
    name: &str,
    body: &Value,
    tag: Option<&str>,
) -> Result<Value, SignalHouseError> {
    HttpClient::require(&[("groupId", Some(group_id)), ("name", Some(name))])?;
    if body.is_null() {
        return Err(SignalHouseError::Validation("body".to_string()));
    }
    let mut payload = json!({ "groupId": group_id, "name": name, "body": body });
    if let Some(tag) = tag {
        payload["tag"] = json!(tag);
    }
    Ok(payload)
}

/// Build the list query string; unset filters are dropped.
pub(crate) fn list_query(params: &GetTemplatesParams<'_>) -> Result<String, SignalHouseError> {
    HttpClient::require(&[("groupId", Some(params.group_id))])?;
    Ok(HttpClient::get_query_string(&[
        ("groupId", Some(params.group_id.to_string())),
        ("state", params.state.map(String::from)),
        ("channel", params.channel.map(String::from)),
        ("tag", params.tag.map(String::from)),
        ("search", params.search.map(String::from)),
        ("sortBy", params.sort_by.map(String::from)),
        ("sortOrder", params.sort_order.map(String::from)),
        ("page", params.page.map(|n| n.to_string())),
        ("limit", params.limit.map(|n| n.to_string())),
    ]))
}

/// Build the `groupId` + `version` query for a per-template read.
pub(crate) fn version_query(
    template_id: &str,
    group_id: &str,
    version: Option<&str>,
) -> Result<String, SignalHouseError> {
    require_template(template_id, group_id)?;
    Ok(HttpClient::get_query_string(&[
        ("groupId", Some(group_id.to_string())),
        ("version", version.map(String::from)),
    ]))
}

/// Build the version-list query string.
pub(crate) fn versions_query(
    template_id: &str,
    group_id: &str,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<String, SignalHouseError> {
    require_template(template_id, group_id)?;
    Ok(HttpClient::get_query_string(&[
        ("groupId", Some(group_id.to_string())),
        ("page", page.map(|n| n.to_string())),
        ("limit", limit.map(|n| n.to_string())),
    ]))
}

/// Build the update body; unchanged fields are omitted.
pub(crate) fn update_body(params: &UpdateTemplateParams<'_>) -> Result<Value, SignalHouseError> {
    require_template(params.template_id, params.group_id)?;
    let mut payload = json!({ "groupId": params.group_id });
    if let Some(name) = params.name {
        payload["name"] = json!(name);
    }
    match params.tag {
        Some(Some(tag)) => payload["tag"] = json!(tag),
        Some(None) => payload["tag"] = Value::Null,
        None => {}
    }
    if let Some(body) = params.body {
        payload["body"] = body.clone();
    }
    Ok(payload)
}

/// Build the publish body.
pub(crate) fn publish_body(template_id: &str, group_id: &str) -> Result<Value, SignalHouseError> {
    require_template(template_id, group_id)?;
    Ok(json!({ "groupId": group_id }))
}

/// Build the archive body.
pub(crate) fn archive_body(template_id: &str, group_id: &str) -> Result<Value, SignalHouseError> {
    require_template(template_id, group_id)?;
    Ok(json!({ "groupId": group_id }))
}

/// Build the validate body from exactly one of a body or a version selector.
pub(crate) fn validate_body(
    template_id: &str,
    group_id: &str,
    body: Option<&Value>,
    version: Option<&str>,
) -> Result<Value, SignalHouseError> {
    require_template(template_id, group_id)?;
    let version = version.filter(|v| !v.is_empty());
    match (body, version) {
        (Some(body), None) => Ok(json!({ "groupId": group_id, "body": body })),
        (None, Some(version)) => {
            Ok(json!({ "groupId": group_id, "version": version_selector(version) }))
        }
        _ => Err(SignalHouseError::Validation(
            "exactly one of body or version".to_string(),
        )),
    }
}

/// Build the capabilities query string.
pub(crate) fn capabilities_query(
    group_id: &str,
    channel: Option<&str>,
    provider: Option<&str>,
) -> Result<String, SignalHouseError> {
    HttpClient::require(&[("groupId", Some(group_id))])?;
    Ok(HttpClient::get_query_string(&[
        ("groupId", Some(group_id.to_string())),
        ("channel", channel.map(String::from)),
        ("provider", provider.map(String::from)),
    ]))
}

/// Build the multipart `file` + `groupId` upload form.
pub(crate) fn media_form(
    group_id: &str,
    file: Vec<u8>,
    file_name: &str,
) -> Result<Form, SignalHouseError> {
    HttpClient::require(&[("groupId", Some(group_id)), ("fileName", Some(file_name))])?;
    if file.is_empty() {
        return Err(SignalHouseError::Validation("file".to_string()));
    }
    Ok(Form::new()
        .part("file", Part::bytes(file).file_name(file_name.to_string()))
        .text("groupId", group_id.to_string()))
}
