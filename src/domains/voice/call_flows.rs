use std::sync::Arc;

use reqwest::Method;
use serde_json::{json, Value};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Optional filters for [`CallFlowsDomain::list`].
///
/// All fields are optional. None-valued fields are omitted from the query
/// string. Field names are snake_case in Rust but serialize to the camelCase
/// query keys the Voice service `/voice/api/v1/call-flows` API expects.
#[derive(Debug, Default)]
pub struct ListCallFlowsParams<'a> {
    /// 1-indexed page (default 1).
    pub page: Option<u32>,
    /// Page size, 1-100 (default 20).
    pub limit: Option<u32>,
    /// Match against the flow name.
    pub q: Option<&'a str>,
    /// Only flows in this subgroup.
    pub subgroup_id: Option<&'a str>,
    /// Include deleted flows (default false).
    pub include_deleted: Option<bool>,
    /// Sort field. One of `createdAt`, `updatedAt`, or `name` (default `createdAt`).
    pub sort_by: Option<&'a str>,
    /// Sort order. One of `asc` or `desc` (default `desc`).
    pub sort_order: Option<&'a str>,
}

/// Call Flows — customer-built call trees (greeting, menu, forward, voicemail,
/// hang up) that run when a number's route, or its Programmable Voice Profile,
/// names them with `routeAction` `"CALL_FLOW"`.
///
/// Each flow has one working draft, edited with [`CallFlowsDomain::save_draft`],
/// and published versions: [`CallFlowsDomain::publish`] validates the draft and
/// makes it the version calls run, while the draft stays editable without
/// affecting calls.
///
/// Routes are mounted under `/voice/api/v1/call-flows` on the Voice service.
/// Accessed via `client.voice.call_flows`.
pub struct CallFlowsDomain {
    http: Arc<HttpClient>,
}

impl CallFlowsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// List the account's call flows, newest first by default. Returns
    /// `{ data: [...], meta: { total, page, limit, totalPages } }`.
    ///
    /// **GET** `/voice/api/v1/call-flows`
    pub async fn list(
        &self,
        params: ListCallFlowsParams<'_>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("page", params.page.map(|p| p.to_string())),
            ("limit", params.limit.map(|l| l.to_string())),
            ("q", params.q.map(String::from)),
            ("subgroupId", params.subgroup_id.map(String::from)),
            (
                "includeDeleted",
                params.include_deleted.map(|d| d.to_string()),
            ),
            ("sortBy", params.sort_by.map(String::from)),
            ("sortOrder", params.sort_order.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/voice/api/v1/call-flows{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// List the step types this service can run, with the outcomes each one
    /// offers. Returns `{ data: { nodeTypes: [{ type, ports, configurable }] } }`.
    ///
    /// **GET** `/voice/api/v1/call-flows/catalog`
    pub async fn catalog(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/api/v1/call-flows/catalog",
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a call flow with its working draft and the issues that stop it being
    /// published. Returns
    /// `{ data: { ...flow, draft: { versionId, nodes, edges, issues } } }`.
    ///
    /// **GET** `/voice/api/v1/call-flows/:id`
    pub async fn get(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/call-flows/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a call flow, with its working draft and the Inbound Call step
    /// every flow starts from.
    ///
    /// `flow_data` keys: `name` (required, unique within the subgroup) and
    /// `subgroupId` (required). The subgroup is fixed once created; only numbers
    /// and Programmable Voice Profiles in it can run the flow. Returns
    /// `{ data: { ...flow, draft } }`.
    ///
    /// **POST** `/voice/api/v1/call-flows`
    pub async fn create(
        &self,
        flow_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json(
                "/voice/api/v1/call-flows",
                Method::POST,
                Some(flow_data),
                options,
            )
            .await
    }

    /// Rename a call flow, or enable or disable it (partial update).
    ///
    /// `update_data` keys, all optional: `name`, `status` (`"ACTIVE"` |
    /// `"DISABLED"`). A disabled flow does not run; numbers routed to it fall
    /// through to their next routing layer. Returns `{ data: { ...flow, draft } }`.
    ///
    /// **PATCH** `/voice/api/v1/call-flows/:id`
    pub async fn update(
        &self,
        id: &str,
        update_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/call-flows/{}", safe),
                Method::PATCH,
                Some(update_data),
                options,
            )
            .await
    }

    /// Delete a call flow. Reversible with [`CallFlowsDomain::restore`]: while
    /// deleted, numbers and profiles routed to it fall through to their next
    /// routing layer. Returns `{ data: { ...flow } }`.
    ///
    /// **DELETE** `/voice/api/v1/call-flows/:id`
    pub async fn delete(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/call-flows/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Restore a deleted call flow. Returns `{ data: { ...flow, draft } }`.
    ///
    /// Sends an empty JSON object: the client always declares JSON, and the
    /// Voice service rejects a JSON request with no body.
    ///
    /// **POST** `/voice/api/v1/call-flows/:id/restore`
    pub async fn restore(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = json!({});
        self.http
            .request_json(
                &format!("/voice/api/v1/call-flows/{}/restore", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Copy a call flow and its working draft into a new flow in the same
    /// subgroup. The copy is not published. `name` defaults to
    /// `"<name> (copy)"`. Returns `{ data: { ...flow, draft } }`.
    ///
    /// **POST** `/voice/api/v1/call-flows/:id/duplicate`
    pub async fn duplicate(
        &self,
        id: &str,
        name: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = match name {
            Some(name) if !name.is_empty() => json!({ "name": name }),
            _ => json!({}),
        };
        self.http
            .request_json(
                &format!("/voice/api/v1/call-flows/{}/duplicate", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Replace the working draft.
    ///
    /// `draft` is `{ "nodes": [...], "edges": [...] }`. Each node is
    /// `{ id, type, name, config, positionX, positionY }` and each edge
    /// `{ id, fromNodeId, toNodeId, port }`; ids are UUIDs you generate and are
    /// kept across saves. The whole draft is saved even when it is not finished;
    /// the issues that would stop it being published come back in the response.
    /// Returns `{ data: { nodes, edges, issues } }`.
    ///
    /// **PUT** `/voice/api/v1/call-flows/:id/draft`
    pub async fn save_draft(
        &self,
        id: &str,
        draft: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/voice/api/v1/call-flows/{}/draft", safe),
                Method::PUT,
                Some(draft),
                options,
            )
            .await
    }

    /// Publish the working draft, making it the version calls run. A draft that
    /// is not ready returns 400 with `error.issues` listing each step that needs
    /// attention. Returns `{ data: { id, versionNumber, status, publishedAt } }`.
    ///
    /// Sends an empty JSON object, for the same reason as
    /// [`CallFlowsDomain::restore`].
    ///
    /// **POST** `/voice/api/v1/call-flows/:id/publish`
    pub async fn publish(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = json!({});
        self.http
            .request_json(
                &format!("/voice/api/v1/call-flows/{}/publish", safe),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }
}
