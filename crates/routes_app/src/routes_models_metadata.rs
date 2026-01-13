//! Model metadata API endpoints
//!
//! This module provides endpoints for:
//! - Refreshing metadata for all models (async)
//! - Refreshing metadata for a single model (sync)
//! - Getting queue status

use crate::AliasResponse;
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Json,
};
use chrono::Utc;
use objs::{Alias, ApiError, API_TAG_MODELS};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{extract_and_store_metadata, AliasNotFoundError, RefreshTask};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

/// Query parameters for metadata refresh endpoint
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct RefreshParams {
  /// Scope of refresh operation: "local" for GGUF models only
  #[serde(default = "default_scope")]
  pub scope: String,
}

fn default_scope() -> String {
  "local".to_string()
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

/// Refresh metadata for all models
#[utoipa::path(
    post,
    path = "/bodhi/v1/models/refresh",
    tag = API_TAG_MODELS,
    operation_id = "refreshAllModelMetadata",
    summary = "Refresh Metadata for All Models",
    description = "Triggers background metadata extraction for all local GGUF models. Requires PowerUser permissions. Returns immediately with 202 Accepted status. Metadata extraction happens asynchronously in the background.",
    params(RefreshParams),
    responses(
        (status = 202, description = "Metadata refresh started in background", body = RefreshResponse,
         example = json!({
             "num_queued": "all"
         })
        ),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not PowerUser role"),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn refresh_all_metadata_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(_params): Query<RefreshParams>,
) -> Result<(StatusCode, Json<RefreshResponse>), ApiError> {
  // Create refresh task
  let task = RefreshTask::RefreshAll {
    created_at: Utc::now(),
  };

  // Enqueue task via QueueProducer
  if let Err(e) = state.app_service().queue_producer().enqueue(task).await {
    tracing::error!("Failed to enqueue refresh task: {}", e);
    return Err(ApiError::from(objs::BadRequestError::new(
      "Failed to enqueue metadata refresh task".to_string(),
    )));
  }

  Ok((
    StatusCode::ACCEPTED,
    Json(RefreshResponse {
      num_queued: "all".to_string(),
      alias: None,
    }),
  ))
}

/// Refresh metadata for a single model (synchronous)
#[utoipa::path(
    post,
    path = "/bodhi/v1/models/{id}/refresh",
    tag = API_TAG_MODELS,
    operation_id = "refreshSingleModelMetadata",
    summary = "Refresh Metadata for Single Model",
    description = "Extracts and updates GGUF metadata for a specific model synchronously. Requires PowerUser permissions. Returns 200 OK with updated model data including metadata.",
    params(
        ("id" = String, Path, description = "Model alias identifier")
    ),
    responses(
        (status = 200, description = "Metadata refreshed successfully", body = AliasResponse),
        (status = 400, description = "Bad request (e.g., API alias, file not found, parse error)"),
        (status = 404, description = "Alias not found"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not PowerUser role"),
    ),
    security(
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn refresh_single_metadata_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<AliasResponse>, ApiError> {
  // Verify alias exists
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&id)
    .await
    .ok_or_else(|| ApiError::from(AliasNotFoundError(id.clone())))?;

  // Verify it's a local model (not API)
  if matches!(alias, Alias::Api(_)) {
    return Err(ApiError::from(objs::BadRequestError::new(format!(
      "Cannot refresh metadata for API alias '{}'",
      id
    ))));
  }

  // Extract and store metadata synchronously
  let metadata_row = extract_and_store_metadata(
    &alias,
    state.app_service().hub_service().as_ref(),
    state.app_service().db_service().as_ref(),
  )
  .await
  .map_err(|e| {
    tracing::error!("Failed to extract metadata for {}: {}", id, e);
    ApiError::from(objs::BadRequestError::new(format!(
      "Failed to extract metadata: {}",
      e
    )))
  })?;

  // Convert to response
  let metadata: objs::ModelMetadata = metadata_row.into();
  let response = AliasResponse::from(alias).with_metadata(Some(metadata));

  Ok(Json(response))
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
  use objs::{test_utils::setup_l10n, FluentLocalizationService};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::Value;
  use server_core::{
    test_utils::{router_state_stub, ResponseTestExt},
    DefaultRouterState, MockSharedContext,
  };
  use services::{
    test_utils::{app_service_stub_builder, AppServiceStubBuilder},
    MockQueueProducer,
  };
  use tower::ServiceExt;

  fn test_metadata_router(state: Arc<dyn RouterState>) -> Router {
    Router::new()
      .route("/api/models/refresh", post(refresh_all_metadata_handler))
      .route(
        "/api/models/{id}/refresh",
        post(refresh_single_metadata_handler),
      )
      .with_state(state)
  }

  // ============================================================================
  // refresh_all_metadata_handler tests
  // ============================================================================

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_refresh_all_metadata_handler_returns_202_accepted(
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
          .body(Body::empty())
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
  async fn test_refresh_all_metadata_handler_enqueue_failure_returns_400(
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
          .body(Body::empty())
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

  // ============================================================================
  // refresh_single_metadata_handler tests
  // ============================================================================

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_refresh_single_metadata_handler_alias_not_found_returns_404(
    #[future] router_state_stub: DefaultRouterState,
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let response = test_metadata_router(Arc::new(router_state_stub))
      .oneshot(
        Request::post("/api/models/nonexistent-alias/refresh")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(StatusCode::NOT_FOUND, response.status());

    let body = response.json::<Value>().await?;
    assert_eq!("alias_not_found_error", body["error"]["code"]);

    Ok(())
  }
}
