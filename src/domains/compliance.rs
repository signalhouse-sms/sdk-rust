use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;
use serde_json::{json, Map, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::{SignalHouseFileResponse, SignalHouseResponse};

/// Filters for [`ComplianceDomain::get_cases`]. Unset fields are left out of the query string.
#[derive(Debug, Default)]
pub struct GetComplianceCasesParams<'a> {
    /// A group you can view; defaults to your active group.
    pub group_id: Option<&'a str>,
    /// `OPEN`, `LIFT_REQUESTED` or `CLOSED`.
    pub status: Option<&'a str>,
    /// `new`, `customer`, `ready`, `lift_requested` or `closed`.
    pub stage: Option<&'a str>,
    /// `CAMPAIGN`, `BRAND`, `NUMBER`, `RCS_AGENT` or `SUBGROUP`.
    pub object_type: Option<&'a str>,
    /// The object's id.
    pub object_id: Option<&'a str>,
    /// Only cases in this subgroup.
    pub subgroup_id: Option<&'a str>,
    /// `true` for cases with an open suspension or hold, `false` for the rest.
    pub suspended: Option<bool>,
    /// Opened on or after this ISO 8601 time.
    pub from: Option<&'a str>,
    /// Opened on or before this ISO 8601 time.
    pub to: Option<&'a str>,
    /// `createdAt` (default), `fixBy` or `severity`.
    pub sort: Option<&'a str>,
    /// Page number, from 1.
    pub page: Option<u32>,
    /// Cases per page, 1-200.
    pub limit: Option<u32>,
}

/// Compliance domain -- compliance cases on `/compliance`.
///
/// Signal House opens a case when a carrier suspends one of your objects, a carrier escalates a
/// message, or Signal House needs something from you. You read your cases, answer their
/// requirements and upload files; only Signal House opens, works and closes them.
pub struct ComplianceDomain {
    http: Arc<HttpClient>,
}

impl ComplianceDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// List the compliance cases you can see, newest first. `counts.waitingOnYou` is the open
    /// cases with requirements still to answer.
    ///
    /// Returns `{ data, total, page, limit, totalPages, counts: { open, waitingOnYou } }`.
    /// Roles: api, admin, developer, billing, user.
    ///
    /// **GET** `/compliance/cases`
    pub async fn get_cases(
        &self,
        params: GetComplianceCasesParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", params.group_id.map(String::from)),
            ("status", params.status.map(String::from)),
            ("stage", params.stage.map(String::from)),
            ("objectType", params.object_type.map(String::from)),
            ("objectId", params.object_id.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            ("suspended", params.suspended.map(|b| b.to_string())),
            ("from", params.from.map(String::from)),
            ("to", params.to.map(String::from)),
            ("sort", params.sort.map(String::from)),
            ("page", params.page.map(|n| n.to_string())),
            ("limit", params.limit.map(|n| n.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/compliance/cases{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get one compliance case: its stage, suspensions, tickets and requirements.
    ///
    /// `case_id` is the case (`CASE-<n>`). Roles: api, admin, developer, billing, user.
    ///
    /// **GET** `/compliance/cases/:caseId`
    pub async fn get_case(
        &self,
        case_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("caseId", Some(case_id))])?;
        let path = format!("/compliance/cases/{}", urlencoding::encode(case_id));
        self.http
            .request_json(&path, Method::GET, None, options)
            .await
    }

    /// Answer one requirement with a note, files, or both. A `RECORDS` requirement needs a file or
    /// a note.
    ///
    /// `requirement_id` is the requirement (`CASE-<n>-<m>`). `answer` is what you did, up to 2,000
    /// characters; `file_ids` is up to 10 `fileId`s from `upload_file`, on this case. Unset fields
    /// are left out of the body. Returns the case. Roles: api, admin, developer, user.
    ///
    /// **PUT** `/compliance/cases/:caseId/requirements/:requirementId`
    pub async fn answer_requirement(
        &self,
        case_id: &str,
        requirement_id: &str,
        answer: Option<&str>,
        file_ids: Option<&[String]>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("caseId", Some(case_id)),
            ("requirementId", Some(requirement_id)),
        ])?;
        let mut body = Map::new();
        if let Some(value) = answer {
            body.insert("answer".into(), json!(value));
        }
        if let Some(value) = file_ids {
            body.insert("fileIds".into(), json!(value));
        }
        let body = Value::Object(body);
        let path = format!(
            "/compliance/cases/{}/requirements/{}",
            urlencoding::encode(case_id),
            urlencoding::encode(requirement_id)
        );
        self.http
            .request_json(&path, Method::PUT, Some(&body), options)
            .await
    }

    /// Upload a file to a compliance case (CSV, XLSX, PDF, PNG, JPG or TXT, up to 10 MB). Pass the
    /// returned `fileId` in `file_ids` when you answer the requirement.
    ///
    /// **POST** `/compliance/files` (multipart)
    ///
    /// - `case_id`: the case the file answers.
    /// - `requirement_id`: the requirement the file answers; sent only when given.
    /// - `file`: the file bytes.
    /// - `filename`: the filename to send with the `file` form field.
    ///
    /// Response: `{ fileId, fileName, mimeType, size }`. Roles: api, admin, developer, user.
    pub async fn upload_file(
        &self,
        case_id: &str,
        requirement_id: Option<&str>,
        file: Vec<u8>,
        filename: String,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("caseId", Some(case_id))])?;
        if file.is_empty() {
            return Err(SignalHouseError::Validation("file".to_string()));
        }
        let mut form = Form::new().text("caseId", case_id.to_string());
        if let Some(requirement_id) = requirement_id.filter(|id| !id.is_empty()) {
            form = form.text("requirementId", requirement_id.to_string());
        }
        let form = form.part("file", Part::bytes(file).file_name(filename));
        self.http
            .request_multipart("/compliance/files", Method::POST, form, options)
            .await
    }

    /// Download a compliance file as exact bytes in `SignalHouseFileResponse::data`.
    ///
    /// **GET** `/compliance/files/:fileId`
    ///
    /// - `file_id`: the `fileId` returned by `upload_file`.
    ///
    /// Roles: api, admin, developer, billing, user.
    pub async fn get_file(
        &self,
        file_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseFileResponse, SignalHouseError> {
        HttpClient::require(&[("fileId", Some(file_id))])?;
        let path = format!("/compliance/files/{}", urlencoding::encode(file_id));
        self.http.request_file(&path, options).await
    }
}
