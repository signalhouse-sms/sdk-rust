use std::sync::Arc;

use reqwest::Method;
use serde_json::Value;

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Subgroups domain -- subgroup CRUD operations.
pub struct SubgroupsDomain {
    http: Arc<HttpClient>,
}

impl SubgroupsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a list of subgroups with optional filters.
    ///
    /// **GET** `/subgroup?...`
    pub async fn get_subgroups(
        &self,
        id: Option<&str>,
        group_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("id", id.map(String::from)),
            ("groupId", group_id.map(String::from)),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(&format!("/subgroup{}", qs), Method::GET, None, options)
            .await
    }

    /// Get resource counts grouped by subgroup ID. Only signalhouse_admin can query all groups;
    /// every other allowed role is restricted to its JWT group scope.
    ///
    /// **GET** `/subgroup/counts?...`
    ///
    /// Requires the `admin` crate feature.
    #[cfg(feature = "admin")]
    pub async fn get_subgroup_counts(
        &self,
        group_id: Option<&str>,
        subgroup_id: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", group_id.map(str::to_owned)),
            ("subgroupId", subgroup_id.map(str::to_owned)),
            ("limit", limit.map(|value| value.to_string())),
            ("offset", offset.map(|value| value.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/subgroup/counts{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a new subgroup.
    /// `subgroup_data` may include optional `carrierIdFamily` and `carrierIdRegion` fields.
    /// Setting those fields requires signalhouse_api, signalhouse_admin, signalhouse_user,
    /// api, admin, developer, or billing role.
    ///
    /// **POST** `/subgroup`
    pub async fn create_subgroup(
        &self,
        subgroup_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/subgroup", Method::POST, Some(subgroup_data), options)
            .await
    }

    /// Update an existing subgroup.
    /// `update_data` may include optional `carrierIdFamily` and `carrierIdRegion` fields.
    /// Setting those fields requires signalhouse_api, signalhouse_admin, signalhouse_user,
    /// api, admin, developer, or billing role.
    /// `update_data` may also include optional `cnamAutoApply` (bool, turns caller name auto-apply
    /// on or off) and `cnamBrandId` (string, the verified brand caller names come from for numbers
    /// with no campaign; send `null` to clear it).
    ///
    /// **PUT** `/subgroup/:id`
    pub async fn update_subgroup(
        &self,
        id: &str,
        update_data: &Value,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/subgroup/{}", safe),
                Method::PUT,
                Some(update_data),
                options,
            )
            .await
    }

    /// Delete a subgroup (mark as inactive).
    ///
    /// **DELETE** `/subgroup/:id`
    pub async fn delete_subgroup(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/subgroup/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }
}
