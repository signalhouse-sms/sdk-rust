use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Tickets domain — staff-only customer tickets → Jira. All methods require
/// the `admin` crate feature.
pub struct TicketsDomain {
    http: Arc<HttpClient>,
}

impl TicketsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Load Jira form metadata for the admin ticket form. Staff-only.
    ///
    /// **GET** `/admin/ticket/jira/metadata`
    #[cfg(feature = "admin")]
    pub async fn get_jira_metadata(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/admin/ticket/jira/metadata", Method::GET, None, options)
            .await
    }

    /// Search open epics. Staff-only.
    ///
    /// **GET** `/admin/ticket/jira/epics`
    #[cfg(feature = "admin")]
    pub async fn search_jira_epics(
        &self,
        query: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[("query", query.map(String::from))]);
        self.http
            .request_json(
                &format!("/admin/ticket/jira/epics{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Search open non-Epic, non-Subtask issues for the Subtask parent picker. Staff-only.
    ///
    /// **GET** `/admin/ticket/jira/parent-issues`
    #[cfg(feature = "admin")]
    pub async fn search_jira_parent_issues(
        &self,
        query: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[("query", query.map(String::from))]);
        self.http
            .request_json(
                &format!("/admin/ticket/jira/parent-issues{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Search assignable Jira users. Staff-only.
    ///
    /// **GET** `/admin/ticket/jira/assignees`
    #[cfg(feature = "admin")]
    pub async fn search_jira_assignees(
        &self,
        query: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[("query", query.map(String::from))]);
        self.http
            .request_json(
                &format!("/admin/ticket/jira/assignees{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Suggest Jira labels. Staff-only.
    ///
    /// **GET** `/admin/ticket/jira/labels`
    #[cfg(feature = "admin")]
    pub async fn search_jira_labels(
        &self,
        query: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[("query", query.map(String::from))]);
        self.http
            .request_json(
                &format!("/admin/ticket/jira/labels{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a Jira ticket from the admin customer ticket form. Staff-only.
    ///
    /// **POST** `/admin/ticket/jira`
    #[cfg(feature = "admin")]
    pub async fn create_jira_ticket(
        &self,
        data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/admin/ticket/jira", Method::POST, Some(data), options)
            .await
    }
}
