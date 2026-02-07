//! Model metadata API endpoints
//!
//! This module provides endpoints for:
//! - Refreshing metadata for models (bulk async or single sync)
//! - Getting queue status

use crate::ModelAliasResponse;
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use objs::{Alias, ApiError, AppError, ErrorType, API_TAG_MODELS};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{extract_and_store_metadata, RefreshTask};
use std::{str::FromStr, sync::Arc};
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum MetadataError {
  #[error("Invalid repo format: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRepoFormat(String),

  #[error("Failed to list aliases.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ListAliasesFailed,

  #[error("Model alias not found for repo={repo}, filename={filename}, snapshot={snapshot}.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  AliasNotFound {
    repo: String,
    filename: String,
    snapshot: String,
  },

  #[error("Failed to extract metadata: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ExtractionFailed(String),

  #[error("Failed to enqueue metadata refresh task.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  EnqueueFailed,
}

/// Source type discriminator for refresh requests
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RefreshSource {
  /// Refresh all local GGUF models (async)
  All,
  /// Refresh specific GGUF model (sync)
  Model,
  // Future: Api for API model cache refresh
}

/// Refresh request - discriminated union by source field
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum RefreshRequest {
  /// Bulk async refresh for all models - Request: {"source": "all"}
  All {},
  /// Single sync refresh for specific model - Request: {"source": "model", "repo": "...", "filename": "...", "snapshot": "..."}
  Model {
    /// Repository in format "user/repo"
    #[schema(example = "bartowski/Qwen2.5-3B-Instruct-GGUF")]
    repo: String,
    /// Filename of the GGUF model
    #[schema(example = "Qwen2.5-3B-Instruct-Q4_K_M.gguf")]
    filename: String,
    /// Snapshot/commit identifier
    #[schema(example = "8ba1c3c3ee94ba4b86ff92a749ae687dc41fce3f")]
    snapshot: String,
  },
}

/// Response for metadata refresh operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshResponse {
  /// Number of models queued ("all" for bulk refresh, "1" for single)
  pub num_queued: String,
  /// Model alias (only for single model refresh)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub alias: Option<String>,
}

/// Response for queue status operations
#[derive(Debug, Serialize, ToSchema)]
pub struct QueueStatusResponse {
  /// Queue status ("idle" or "processing")
  pub status: String,
}

/// Enum for different refresh response types
pub enum RefreshResponseType {
  Sync(ModelAliasResponse),
  Async(RefreshResponse),
}

impl axum::response::IntoResponse for RefreshResponseType {
  fn into_response(self) -> axum::response::Response {
    match self {
      RefreshResponseType::Sync(response) => (StatusCode::OK, Json(response)).into_response(),
      RefreshResponseType::Async(response) => {
        (StatusCode::ACCEPTED, Json(response)).into_response()
      }
    }
  }
}

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
        (status = 200, description = "Metadata refreshed successfully (sync mode)", body = crate::ModelAliasResponse),
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
  State(state): State<Arc<dyn RouterState>>,
  Json(request): Json<RefreshRequest>,
) -> Result<RefreshResponseType, ApiError> {
  match request {
    RefreshRequest::Model {
      repo,
      filename,
      snapshot,
    } => {
      // Parse and validate repo
      let repo_parsed = objs::Repo::from_str(&repo).map_err(|e| {
        MetadataError::InvalidRepoFormat(e.to_string())
      })?;

      // Find the ModelAlias for this GGUF file
      let all_aliases = state
        .app_service()
        .data_service()
        .list_aliases()
        .await
        .map_err(|e| {
          tracing::error!("Failed to list aliases: {}", e);
          MetadataError::ListAliasesFailed
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
        .ok_or_else(|| {
          MetadataError::AliasNotFound {
            repo: repo.clone(),
            filename: filename.clone(),
            snapshot: snapshot.clone(),
          }
        })?;

      // Extract and store metadata synchronously
      let metadata_row = extract_and_store_metadata(
        &Alias::Model(alias.clone()),
        state.app_service().hub_service().as_ref(),
        state.app_service().db_service().as_ref(),
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
        MetadataError::ExtractionFailed(e.to_string())
      })?;

      // Convert to response with metadata
      let metadata: objs::ModelMetadata = metadata_row.into();
      let response = crate::ModelAliasResponse::from(alias).with_metadata(Some(metadata));

      Ok(RefreshResponseType::Sync(response))
    }
    RefreshRequest::All {} => {
      // Bulk async refresh
      let task = RefreshTask::RefreshAll {
        created_at: Utc::now(),
      };

      // Enqueue task via QueueProducer
      if let Err(e) = state.app_service().queue_producer().enqueue(task).await {
        tracing::error!("Failed to enqueue refresh task: {}", e);
        return Err(MetadataError::EnqueueFailed)?;
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
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<QueueStatusResponse>, ApiError> {
  let status = state.app_service().queue_status();
  Ok(Json(QueueStatusResponse { status }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::{body::Body, http::Request, routing::post, Router};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::Value;
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
  use services::{
    test_utils::{app_service_stub_builder, AppServiceStubBuilder},
    MockQueueProducer,
  };
  use tower::ServiceExt;

  fn test_metadata_router(state: Arc<dyn RouterState>) -> Router {
    Router::new()
      .route("/api/models/refresh", post(refresh_metadata_handler))
      .with_state(state)
  }

  // ============================================================================
  // refresh_metadata_handler tests
  // ============================================================================

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_refresh_metadata_no_params_returns_202_accepted(
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    // Configure mock to succeed on enqueue
    let mut mock_queue = MockQueueProducer::new();
    mock_queue
      .expect_enqueue()
      .returning(|_| Box::pin(async { Ok(()) }));
    mock_queue
      .expect_queue_status()
      .returning(|| "idle".to_string());

    let app_service = app_service_stub_builder
      .queue_producer(Arc::new(mock_queue))
      .build()
      .unwrap();

    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      Arc::new(app_service),
    ));

    let response = test_metadata_router(state)
      .oneshot(
        Request::post("/api/models/refresh")
          .header("content-type", "application/json")
          .body(Body::from(r#"{"source":"all"}"#))
          .unwrap(),
      )
      .await?;

    assert_eq!(StatusCode::ACCEPTED, response.status());

    let body = response.json::<RefreshResponse>().await?;
    assert_eq!("all", body.num_queued);
    assert!(body.alias.is_none());

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_refresh_metadata_enqueue_failure_returns_400(
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    // Configure mock to fail on enqueue
    let mut mock_queue = MockQueueProducer::new();
    mock_queue
      .expect_enqueue()
      .returning(|_| Box::pin(async { Err("Queue full".into()) }));
    mock_queue
      .expect_queue_status()
      .returning(|| "idle".to_string());

    let app_service = app_service_stub_builder
      .queue_producer(Arc::new(mock_queue))
      .build()
      .unwrap();

    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      Arc::new(app_service),
    ));

    let response = test_metadata_router(state)
      .oneshot(
        Request::post("/api/models/refresh")
          .header("content-type", "application/json")
          .body(Body::from(r#"{"source":"all"}"#))
          .unwrap(),
      )
      .await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());

    let body = response.json::<Value>().await?;
    assert!(body["error"]["message"]
      .as_str()
      .unwrap()
      .contains("enqueue"));

    Ok(())
  }
}
