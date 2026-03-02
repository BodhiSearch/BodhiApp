//! Model metadata API endpoints
//!
//! This module provides endpoints for:
//! - Refreshing metadata for models (bulk async or single sync)
//! - Getting queue status

use crate::models::error::ModelRouteError;
use crate::models::models_api_schemas::{
  ModelAliasResponse, QueueStatusResponse, RefreshRequest, RefreshResponse, RefreshResponseType,
};
use crate::shared::AuthScope;
use crate::API_TAG_MODELS;
use axum::Json;
use services::Alias;
use crate::ApiError;
use services::{extract_and_store_metadata, RefreshTask};
use std::str::FromStr;

/// Refresh metadata for models
#[utoipa::path(
    post,
    path = "/bodhi/v1/models/refresh",
    tag = API_TAG_MODELS,
    operation_id = "refreshModelMetadata",
    summary = "Refresh Model Metadata",
    description = "Refresh metadata for models. Supports two modes via discriminated request body:\n\n\
                   - Bulk async: `{\"source\": \"all\"}` triggers background extraction for all local GGUF models (202 Accepted)\n\n\
                   - Single sync: `{\"source\": \"model\", \"repo\": \"...\", \"filename\": \"...\", \"snapshot\": \"...\"}` performs immediate extraction (200 OK)\n\n\
                   Requires PowerUser permissions.",
    request_body(
        content = RefreshRequest,
        description = "Refresh request - either bulk (source='all') or single model (source='model' with identifiers)",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Metadata refreshed successfully (sync mode)", body = ModelAliasResponse),
        (status = 202, description = "Metadata refresh queued in background (bulk mode)", body = RefreshResponse,
         example = json!({
             "num_queued": "all"
         })
        ),
        (status = 400, description = "Bad request (invalid source, missing required fields, file not found)"),
        (status = 404, description = "Model alias not found for specified repo/filename/snapshot"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not PowerUser role"),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn refresh_metadata_handler(
  auth_scope: AuthScope,
  Json(request): Json<RefreshRequest>,
) -> Result<RefreshResponseType, ApiError> {
  match request {
    RefreshRequest::Model {
      repo,
      filename,
      snapshot,
    } => {
      // Parse and validate repo
      let repo_parsed = services::Repo::from_str(&repo)
        .map_err(|e| ModelRouteError::InvalidRepoFormat(e.to_string()))?;

      // Find the ModelAlias for this GGUF file
      let all_aliases = auth_scope
        .data_service()
        .list_aliases()
        .await
        .map_err(|e| {
          tracing::error!("Failed to list aliases: {}", e);
          ModelRouteError::ListAliasesFailed
        })?;

      let alias = all_aliases
        .into_iter()
        .find(|a| match a {
          Alias::Model(model_alias) => {
            model_alias.repo == repo_parsed
              && model_alias.filename == filename
              && model_alias.snapshot == snapshot
          }
          _ => false,
        })
        .and_then(|a| {
          if let Alias::Model(m) = a {
            Some(m)
          } else {
            None
          }
        })
        .ok_or_else(|| ModelRouteError::AliasNotFound {
          repo: repo.clone(),
          filename: filename.clone(),
          snapshot: snapshot.clone(),
        })?;

      // Extract and store metadata synchronously
      let metadata_row = extract_and_store_metadata(
        &Alias::Model(alias.clone()),
        auth_scope.hub_service().as_ref(),
        auth_scope.db_service().as_ref(),
      )
      .await
      .map_err(|e| {
        tracing::error!(
          "Failed to extract metadata for {}/{}/{}: {}",
          repo,
          filename,
          snapshot,
          e
        );
        ModelRouteError::ExtractionFailed(e.to_string())
      })?;

      // Convert to response with metadata
      let metadata: services::ModelMetadata = metadata_row.into();
      let response = ModelAliasResponse::from(alias).with_metadata(Some(metadata));

      Ok(RefreshResponseType::Sync(response))
    }
    RefreshRequest::All {} => {
      // Bulk async refresh
      let task = RefreshTask::RefreshAll {
        created_at: auth_scope.time_service().utc_now(),
      };

      // Enqueue task via QueueProducer
      if let Err(e) = auth_scope.queue_producer().enqueue(task).await {
        tracing::error!("Failed to enqueue refresh task: {}", e);
        return Err(ModelRouteError::EnqueueFailed)?;
      }

      let response = RefreshResponse {
        num_queued: "all".to_string(),
        alias: None,
      };

      Ok(RefreshResponseType::Async(response))
    }
  }
}

/// Get queue status
#[utoipa::path(
    get,
    path = "/bodhi/v1/queue",
    tag = API_TAG_MODELS,
    operation_id = "getQueueStatus",
    summary = "Get Queue Status",
    description = "Returns the current status of the metadata refresh queue. Requires PowerUser permissions.",
    responses(
        (status = 200, description = "Queue status retrieved successfully", body = QueueStatusResponse,
         example = json!({
             "status": "idle"
         })
        ),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not PowerUser role"),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn queue_status_handler(
  auth_scope: AuthScope,
) -> Result<Json<QueueStatusResponse>, ApiError> {
  let status = auth_scope.queue_status();
  Ok(Json(QueueStatusResponse { status }))
}

#[cfg(test)]
#[path = "test_metadata.rs"]
mod test_metadata;
