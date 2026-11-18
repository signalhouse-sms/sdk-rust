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

    /// Resolve the mapping between a GoHighLevel location and a Signal House subgroup, in either
    /// direction. Provide exactly one of `ghl_location_id` or `subgroup_id`. `group_id` is optional -
    /// customer roles default to their own accessible group(s) and are rejected if they name a group
    /// they don't belong to; a `subgroup_id`/`ghl_location_id` that belongs to a different group than
    /// the caller's returns a 404, identical to a nonexistent one.
    ///
    /// **GET** `/subgroup/ghl-location-lookup?...`
    pub async fn lookup_ghl_location(
        &self,
        group_id: Option<&str>,
        ghl_location_id: Option<&str>,
        subgroup_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let qs = HttpClient::get_query_string(&[
            ("groupId", group_id.map(String::from)),
            ("ghlLocationId", ghl_location_id.map(String::from)),
            ("subgroupId", subgroup_id.map(String::from)),
        ]);
        self.http
            .request_json(
                &format!("/subgroup/ghl-location-lookup{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Link (or, with `ghl_location_id: None`, unlink) a GoHighLevel location to a subgroup.
    /// Requires `signalhouse_api` or `signalhouse_admin` role. At most one subgroup may hold a
    /// given `ghl_location_id`; linking it here steals it from any other subgroup that currently
    /// holds it. `ghl_location_name: None` leaves the subgroup's currently stored name untouched
    /// (the field is omitted from the request entirely, never sent as an explicit null) - but only
    /// when `ghl_location_id` is unchanged, otherwise it is cleared; it is always cleared when
    /// unlinking. `claimed_at` (an ISO 8601 string) is required when `ghl_location_id` is `Some` -
    /// when this link was locally claimed by the caller, used to detect and reject a delayed/
    /// out-of-order push whose claim has since been superseded by a newer one recorded elsewhere;
    /// ignored when unlinking.
    ///
    /// **PUT** `/subgroup/ghl-location-link`
    pub async fn link_ghl_location(
        &self,
        subgroup_id: &str,
        ghl_location_id: Option<&str>,
        ghl_location_name: Option<&str>,
        claimed_at: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("subgroupId", Some(subgroup_id))])?;
        if ghl_location_id.is_some() {
            HttpClient::require(&[("claimedAt", claimed_at)])?;
        }
        let mut body = serde_json::json!({
            "subgroupId": subgroup_id,
            "ghlLocationId": ghl_location_id,
        });
        if let Some(name) = ghl_location_name {
            body["ghlLocationName"] = Value::from(name);
        }
        if let Some(claimed_at) = claimed_at {
            body["claimedAt"] = Value::from(claimed_at);
        }
        self.http
            .request_json(
                "/subgroup/ghl-location-link",
                Method::PUT,
                Some(&body),
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
