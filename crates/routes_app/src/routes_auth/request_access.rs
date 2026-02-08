use crate::{LoginError, ENDPOINT_APPS_REQUEST_ACCESS};
use axum::{extract::State, Json};
use objs::{ApiError, API_TAG_AUTH};
use server_core::RouterState;
use services::{
  db::AppClientToolsetConfigRow, AppAccessRequest, AppAccessResponse, AppClientToolset,
  SecretServiceExt,
};
use std::sync::Arc;

/// Request access for an app client to this resource server
#[utoipa::path(
    post,
    path = ENDPOINT_APPS_REQUEST_ACCESS,
    tag = API_TAG_AUTH,
    operation_id = "requestAccess",
    summary = "Request Resource Access",
    description = "Requests access permissions for an application client to access this resource server's protected resources. Supports caching via optional version parameter.",
    request_body(
        content = AppAccessRequest,
        description = "Application client requesting access",
        example = json!({
            "app_client_id": "my_app_client_123",
            "toolset_scope_ids": ["uuid-for-toolset-scope-1"],
            "version": "v1.0.0"
        })
    ),
    responses(
        (status = 200, description = "Access granted successfully", body = AppAccessResponse,
         example = json!({
             "scope": "scope_resource_bodhi-server",
             "toolsets": [{"id": "builtin-exa-web-search", "scope": "scope_toolset-builtin-exa-web-search"}],
             "app_client_config_version": "v1.0.0"
         })),
    ),
    security(
        (),
        ("bearer_api_token" = []),
        ("bearer_oauth_token" = []),
        ("session_auth" = [])
    )
)]
pub async fn request_access_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<AppAccessRequest>,
) -> Result<Json<AppAccessResponse>, ApiError> {
  let app_service = state.app_service();
  let secret_service = app_service.secret_service();
  let auth_service = app_service.auth_service();
  let db_service = app_service.db_service();

  // Get app registration info
  let app_reg_info = secret_service
    .app_reg_info()?
    .ok_or(LoginError::AppRegInfoNotFound)?;

  // Check cache if version is provided
  if let Ok(Some(cached)) = db_service
    .get_app_client_toolset_config(&request.app_client_id)
    .await
  {
    // Require both versions to be Some and matching for cache hit
    if let (Some(ref version), Some(ref cached_version)) =
      (&request.version, &cached.config_version)
    {
      if cached_version == version {
        // Version matches - check if all requested scope_ids are already added to resource-client
        let toolsets: Vec<AppClientToolset> =
          serde_json::from_str(&cached.toolsets_json).unwrap_or_default();

        let all_scopes_added = if let Some(ref requested_scope_ids) = request.toolset_scope_ids {
          // Check if all requested scope_ids have added_to_resource_client=true
          requested_scope_ids.iter().all(|requested_id| {
            toolsets
              .iter()
              .any(|t| &t.scope_id == requested_id && t.added_to_resource_client == Some(true))
          })
        } else {
          // No toolset_scope_ids requested - cache hit
          true
        };

        if all_scopes_added {
          // Full cache hit - return cached data
          return Ok(Json(AppAccessResponse {
            scope: cached.resource_scope,
            toolsets,
            app_client_config_version: cached.config_version.clone(),
          }));
        }
      }
    }
  }

  // Cache miss or scope_ids not yet added - call auth server with toolset_scope_ids
  let auth_response = auth_service
    .request_access(
      &app_reg_info.client_id,
      &app_reg_info.client_secret,
      &request.app_client_id,
      request.toolset_scope_ids.clone(),
    )
    .await?;

  // Log if version mismatch or None cases
  match (&request.version, &auth_response.app_client_config_version) {
    (Some(req_ver), Some(resp_ver)) if req_ver != resp_ver => {
      tracing::warn!(
        "App client {} sent version {} but auth server returned version {}",
        request.app_client_id,
        req_ver,
        resp_ver
      );
    }
    (Some(_), None) | (None, Some(_)) | (None, None) => {
      tracing::warn!(
        "App client {} does not have config versioning configured properly",
        request.app_client_id
      );
    }
    _ => {} // versions match, no log needed
  }

  // Store/update cache
  let toolsets_json =
    serde_json::to_string(&auth_response.toolsets).unwrap_or_else(|_| "[]".to_string());
  let now = db_service.now().timestamp();
  let config_row = AppClientToolsetConfigRow {
    id: 0, // Will be set by DB
    app_client_id: request.app_client_id.clone(),
    config_version: auth_response.app_client_config_version.clone(),
    toolsets_json,
    resource_scope: auth_response.scope.clone(),
    created_at: now,
    updated_at: now,
  };

  if let Err(e) = db_service
    .upsert_app_client_toolset_config(&config_row)
    .await
  {
    tracing::warn!(
      "Failed to cache app client toolset config for {}: {}",
      request.app_client_id,
      e
    );
    // Don't fail the request if caching fails
  }

  Ok(Json(AppAccessResponse {
    scope: auth_response.scope,
    toolsets: auth_response.toolsets,
    app_client_config_version: auth_response.app_client_config_version,
  }))
}
