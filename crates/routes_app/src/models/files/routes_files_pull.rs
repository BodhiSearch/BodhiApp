use crate::models::error::ModelRouteError;
use crate::shared::AuthScope;
use crate::{BodhiErrorResponse, ValidatedJson};
use crate::{PaginationSortParams, API_TAG_MODELS_FILES, ENDPOINT_MODELS_FILES_PULL};
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
    path = ENDPOINT_MODELS_FILES_PULL,
    tag = API_TAG_MODELS_FILES,
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
) -> Result<Json<PaginatedDownloadResponse>, BodhiErrorResponse> {
  let result = auth_scope
    .downloads()
    .list(query.page, query.page_size)
    .await?;
  Ok(Json(result))
}

/// Start a new model file download
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_FILES_PULL,
    tag = API_TAG_MODELS_FILES,
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
) -> Result<(StatusCode, Json<DownloadRequest>), BodhiErrorResponse> {
  let repo = Repo::try_from(payload.repo.clone())?;

  // file-existence check needs no auth scope
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

  let existing = auth_scope
    .downloads()
    .find_by_repo_filename(&payload.repo, &payload.filename)
    .await?;

  if let Some(existing_request) = existing {
    return Ok((StatusCode::OK, Json(existing_request.into())));
  }

  let download_request = auth_scope.downloads().create(&payload).await?;

  spawn_pull(
    auth_scope.app_service().clone(),
    repo,
    payload.filename,
    download_request.id.clone(),
    download_request.tenant_id.clone(),
  );

  Ok((StatusCode::CREATED, Json(download_request.into())))
}

/// Spawns the background pull task (download + status update). Shared by create and retry.
/// On retry, hf-hub resumes from the surviving `.sync.part` partial.
fn spawn_pull(
  app_service: Arc<dyn AppService>,
  repo: Repo,
  filename: String,
  request_id: String,
  tenant_id: String,
) {
  spawn(async move {
    let result = execute_pull_by_repo_file(
      app_service.as_ref(),
      repo,
      filename,
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
}

/// Get the status of a specific download request
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS_FILES_PULL.to_owned() + "/{id}",
    tag = API_TAG_MODELS_FILES,
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
        (status = 404, description = "Download request not found", body = BodhiErrorResponse,
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
) -> Result<Json<DownloadRequest>, BodhiErrorResponse> {
  let download_request = auth_scope.downloads().get(&id).await?;
  Ok(Json(download_request.into()))
}

/// Archive a download request (hide it from the list and the API)
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_FILES_PULL.to_owned() + "/{id}/archive",
    tag = API_TAG_MODELS_FILES,
    operation_id = "archiveDownload",
    summary = "Archive Download Request",
    description = "Archives a completed, failed, or queued download request so it no longer appears in the downloads list or the list API response. Actively-downloading requests cannot be archived.",
    params(
        ("id" = String, Path, description = "Unique identifier of the download request")
    ),
    responses(
        (status = 200, description = "Download request archived", body = DownloadRequest),
        (status = 400, description = "Download is actively downloading and cannot be archived", body = BodhiErrorResponse),
        (status = 404, description = "Download request not found", body = BodhiErrorResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_pull_archive(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<DownloadRequest>, BodhiErrorResponse> {
  let archived = auth_scope.downloads().archive(&id).await?;
  Ok(Json(archived.into()))
}

/// Retry a failed download request
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS_FILES_PULL.to_owned() + "/{id}/retry",
    tag = API_TAG_MODELS_FILES,
    operation_id = "retryDownload",
    summary = "Retry Failed Download Request",
    description = "Resets a failed download request to pending and re-runs it. The download resumes from the partially-downloaded file when present. Only failed requests can be retried.",
    params(
        ("id" = String, Path, description = "Unique identifier of the download request")
    ),
    responses(
        (status = 200, description = "Download request reset and re-started", body = DownloadRequest),
        (status = 400, description = "Download is not in a failed state", body = BodhiErrorResponse),
        (status = 404, description = "Download request not found", body = BodhiErrorResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn models_pull_retry(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<DownloadRequest>, BodhiErrorResponse> {
  let request = auth_scope.downloads().reset_for_retry(&id).await?;
  let repo = Repo::try_from(request.repo.clone())?;

  spawn_pull(
    auth_scope.app_service().clone(),
    repo,
    request.filename.clone(),
    request.id.clone(),
    request.tenant_id.clone(),
  );

  Ok(Json(request.into()))
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
      let api_error: BodhiErrorResponse = e.into();
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
