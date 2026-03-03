use crate::models::error::ModelRouteError;
use crate::shared::AuthScope;
use crate::{ApiError, OpenAIApiError, ValidatedJson};
use crate::{PaginationSortParams, API_TAG_MODELS, ENDPOINT_MODEL_PULL};
use axum::http::StatusCode;
use axum::{
  extract::{Path, Query},
  Json,
};
use services::Repo;
use services::{
  AppService, DatabaseProgress, DownloadRequest, DownloadStatus, NewDownloadRequest,
  PaginatedDownloadResponse, Progress,
};
use std::sync::Arc;
use tokio::spawn;
use tracing::debug;

/// List all model download requests
#[utoipa::path(
    get,
    path = ENDPOINT_MODEL_PULL,
    tag = API_TAG_MODELS,
    operation_id = "listDownloads",
    summary = "List Model Download Requests",
    description = "Retrieves paginated list of all model download requests with their current status, progress, and metadata. Includes both active downloads and completed/failed requests.",
    params(
        PaginationSortParams
    ),
    responses(
        (status = 200, description = "Model download requests retrieved successfully", body = PaginatedDownloadResponse,
         example = json!({
             "data": [{
                 "id": "download_123",
                 "repo": "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
                 "filename": "mistral-7b-instruct-v0.1.Q4_K_M.gguf",
                 "status": "downloading",
                 "progress": 45.5,
                 "created_at": "2024-01-15T10:30:00Z",
                 "updated_at": "2024-01-15T10:35:00Z"
             }],
             "total": 1,
             "page": 1,
             "page_size": 10
         })
        ),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_pull_index(
  auth_scope: AuthScope,
  Query(query): Query<PaginationSortParams>,
) -> Result<Json<PaginatedDownloadResponse>, ApiError> {
  let result = auth_scope
    .downloads()
    .list(query.page, query.page_size)
    .await?;
  Ok(Json(result))
}

/// Start a new model file download
#[utoipa::path(
    post,
    path = ENDPOINT_MODEL_PULL,
    tag = API_TAG_MODELS,
    operation_id = "pullModelFile",
    summary = "Start Model File Download",
    description = "Initiates a new model file download from HuggingFace repository. Creates a download request that can be tracked for progress. Returns existing request if download is already in progress.",
    request_body(
        content = NewDownloadRequest,
        description = "Model file download specification with repository and filename",
        example = json!({
            "repo": "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
            "filename": "mistral-7b-instruct-v0.1.Q8_0.gguf"
        })
    ),
    responses(
        (status = 201, description = "Download request created", body = DownloadRequest,
         example = json!({
             "id": "550e8400-e29b-41d4-a716-446655440000",
             "repo": "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
             "filename": "mistral-7b-instruct-v0.1.Q8_0.gguf",
             "status": "pending",
             "error": null,
             "created_at": "2024-11-10T04:52:06.786Z",
             "updated_at": "2024-11-10T04:52:06.786Z"
         })),
        (status = 200, description = "Existing download request found", body = DownloadRequest,
         example = json!({
             "id": "550e8400-e29b-41d4-a716-446655440000",
             "repo": "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
             "filename": "mistral-7b-instruct-v0.1.Q8_0.gguf",
             "status": "pending",
             "error": null,
             "created_at": "2024-11-10T04:52:06.786Z",
             "updated_at": "2024-11-10T04:52:06.786Z"
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_pull_create(
  auth_scope: AuthScope,
  ValidatedJson(payload): ValidatedJson<NewDownloadRequest>,
) -> Result<(StatusCode, Json<DownloadRequest>), ApiError> {
  let repo = Repo::try_from(payload.repo.clone())?;

  // Check if the file is already downloaded (no auth required)
  if let Ok(true) = auth_scope
    .hub()
    .local_file_exists(&repo, &payload.filename, None)
  {
    return Err(ModelRouteError::FileAlreadyExists {
      repo: repo.to_string(),
      filename: payload.filename.clone(),
      snapshot: "main".to_string(),
    })?;
  }

  // Check for existing pending download request
  let existing = auth_scope
    .downloads()
    .find_by_repo_filename(&payload.repo, &payload.filename)
    .await?;

  if let Some(existing_request) = existing {
    return Ok((StatusCode::OK, Json(existing_request.into())));
  }

  let download_request = auth_scope.downloads().create(&payload).await?;

  let app_service = auth_scope.app_service().clone();
  let request_id = download_request.id.clone();
  let tenant_id = download_request.tenant_id.clone();

  spawn(async move {
    let result = execute_pull_by_repo_file(
      app_service.as_ref(),
      repo,
      payload.filename,
      None,
      Some(Progress::Database(DatabaseProgress::new(
        app_service.db_service().clone(),
        tenant_id.clone(),
        request_id.clone(),
      ))),
    )
    .await;
    update_download_status(app_service, tenant_id, request_id, result).await;
  });

  Ok((StatusCode::CREATED, Json(download_request.into())))
}

/// Get the status of a specific download request
#[utoipa::path(
    get,
    path = ENDPOINT_MODEL_PULL.to_owned() + "/{id}",
    tag = API_TAG_MODELS,
    operation_id = "getDownloadStatus",
    summary = "Get Download Request Status",
    description = "Retrieves the current status and progress information for a specific model download request. Includes download progress, error details, and completion status.",
    params(
        ("id" = String, Path,
         description = "Unique identifier of the download request (UUID format)",
         example = "550e8400-e29b-41d4-a716-446655440000")
    ),
    responses(
        (status = 200, description = "Download request found", body = DownloadRequest,
         example = json!({
             "id": "550e8400-e29b-41d4-a716-446655440000",
             "repo": "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
             "filename": "mistral-7b-instruct-v0.1.Q8_0.gguf",
             "status": "completed",
             "error": null,
             "created_at": "2024-11-10T04:52:06.786Z",
             "updated_at": "2024-01-20T12:00:10Z"
         })),
        (status = 404, description = "Download request not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "item '550e8400-e29b-41d4-a716-446655440000' of type 'download_requests' not found in db",
                 "type": "not_found_error",
                 "code": "db_error-item_not_found"
             }
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_pull_show(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<DownloadRequest>, ApiError> {
  let download_request = auth_scope.downloads().get(&id).await?;
  Ok(Json(download_request.into()))
}

async fn update_download_status(
  app_service: Arc<dyn AppService>,
  tenant_id: String,
  request_id: String,
  result: Result<(), ModelRouteError>,
) {
  let (status, error) = match result {
    Ok(_) => (DownloadStatus::Completed, None),
    Err(e) => {
      let api_error: ApiError = e.into();
      (DownloadStatus::Error, Some(api_error.to_string()))
    }
  };

  if let Err(e) = app_service
    .download_service()
    .update_status(&tenant_id, &request_id, status, error)
    .await
  {
    tracing::error!(
      request_id = %request_id,
      tenant_id = %tenant_id,
      error = %e,
      "Failed to update download request status"
    );
  }
}

async fn execute_pull_by_repo_file(
  service: &dyn AppService,
  repo: Repo,
  filename: String,
  snapshot: Option<String>,
  progress: Option<Progress>,
) -> Result<(), ModelRouteError> {
  let model_file_exists =
    service
      .hub_service()
      .local_file_exists(&repo, &filename, snapshot.clone())?;
  if model_file_exists {
    debug!("repo: '{repo}', filename: '{filename}' already exists in $HF_HOME");
    return Ok(());
  } else {
    service
      .hub_service()
      .download(&repo, &filename, snapshot.clone(), progress)
      .await?;
    debug!("repo: '{repo}', filename: '{filename}' downloaded into $HF_HOME");
  }
  Ok(())
}

#[cfg(test)]
#[path = "test_pull.rs"]
mod test_pull;
