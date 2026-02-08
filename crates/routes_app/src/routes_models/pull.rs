use crate::{
  NewDownloadRequest, PaginatedDownloadResponse, PaginationSortParams, PullError,
  ENDPOINT_MODEL_PULL,
};
use axum::http::StatusCode;
use axum::{
  extract::{Path, Query, State},
  Json,
};
use axum_extra::extract::WithRejection;
use chrono::Utc;
use commands::PullCommand;
use objs::{ApiError, OpenAIApiError, Repo, API_TAG_MODELS};
use server_core::RouterState;
use services::db::ItemNotFound;
use services::RemoteModelNotFoundError;
use services::{
  db::{DownloadRequest, DownloadStatus},
  AppService, DatabaseProgress, Progress,
};
use std::sync::Arc;
use tokio::spawn;

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
pub async fn list_downloads_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(query): Query<PaginationSortParams>,
) -> Result<Json<PaginatedDownloadResponse>, ApiError> {
  let downloads = state
    .app_service()
    .db_service()
    .list_download_requests(query.page, query.page_size)
    .await?;

  let paginated = PaginatedDownloadResponse {
    data: downloads.0,
    total: downloads.1 as usize,
    page: query.page,
    page_size: query.page_size,
  };
  Ok(Json(paginated))
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
pub async fn create_pull_request_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<NewDownloadRequest>, ApiError>,
) -> Result<(StatusCode, Json<DownloadRequest>), ApiError> {
  let repo = Repo::try_from(payload.repo.clone())?;

  // Check if the file is already downloaded
  if let Ok(true) =
    state
      .app_service()
      .hub_service()
      .local_file_exists(&repo, &payload.filename, None)
  {
    return Err(PullError::FileAlreadyExists {
      repo: repo.to_string(),
      filename: payload.filename.clone(),
      snapshot: "main".to_string(),
    })?;
  }

  // Check for existing pending download request
  let pending_downloads = state
    .app_service()
    .db_service()
    .find_download_request_by_repo_filename(&payload.repo, &payload.filename)
    .await?
    .into_iter()
    .find(|r| r.repo == payload.repo && r.filename == payload.filename);

  if let Some(existing_request) = pending_downloads {
    return Ok((StatusCode::OK, Json(existing_request)));
  }

  let download_request = DownloadRequest::new_pending(
    repo.to_string().as_str(),
    payload.filename.as_str(),
    state.app_service().time_service().utc_now(),
  );

  state
    .app_service()
    .db_service()
    .create_download_request(&download_request)
    .await?;

  let app_service = state.app_service().clone();
  let request_id = download_request.id.clone();

  spawn(async move {
    let command = PullCommand::ByRepoFile {
      repo,
      filename: payload.filename,
      snapshot: None,
    };
    let result = command
      .execute(
        app_service.clone(),
        Some(Progress::Database(DatabaseProgress::new(
          app_service.db_service().clone(),
          request_id.clone(),
        ))),
      )
      .await;
    update_download_status(app_service, request_id, result).await;
  });

  Ok((StatusCode::CREATED, Json(download_request)))
}

/// Start a model file download using a predefined alias
#[utoipa::path(
    post,
    path = ENDPOINT_MODEL_PULL.to_owned() + "/{alias}",
    tag = API_TAG_MODELS,
    operation_id = "pullModelByAlias",
    summary = "Download Model by Alias",
    description = "Initiates a model download using a predefined alias that maps to specific repository and file combinations. This provides a convenient way to download popular models without specifying exact repository details.",
    params(
        ("alias" = String, Path,
         description = "Predefined model alias. Available aliases include popular models like llama2:chat, mistral:instruct, phi3:mini, etc. Use the /models endpoint to see all available aliases.",
         example = "llama2:chat")
    ),
    responses(
        (status = 201, description = "Download request created", body = DownloadRequest,
         example = json!({
             "id": "550e8400-e29b-41d4-a716-446655440000",
             "repo": "TheBloke/Llama-2-7B-Chat-GGUF",
             "filename": "llama-2-7b-chat.Q8_0.gguf",
             "status": "pending",
             "error": null,
             "created_at": "2024-11-10T04:52:06.786Z",
             "updated_at": "2024-11-10T04:52:06.786Z"
         })),
        (status = 200, description = "Existing download request found", body = DownloadRequest,
         example = json!({
             "id": "550e8400-e29b-41d4-a716-446655440000",
             "repo": "TheBloke/Llama-2-7B-Chat-GGUF",
             "filename": "llama-2-7b-chat.Q8_0.gguf",
             "status": "pending",
             "error": null,
             "created_at": "2024-11-10T04:52:06.786Z",
             "updated_at": "2024-11-10T04:52:06.786Z"
         })),
        (status = 404, description = "Alias not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "remote model alias 'invalid:model' not found, check your alias and try again",
                 "type": "not_found_error",
                 "code": "remote_model_not_found_error"
             }
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn pull_by_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(alias): Path<String>,
) -> Result<(StatusCode, Json<DownloadRequest>), ApiError> {
  let model = state
    .app_service()
    .data_service()
    .find_remote_model(&alias)?
    .ok_or(RemoteModelNotFoundError::new(alias.clone()))?;

  // Check if the file is already downloaded
  if let Ok(true) =
    state
      .app_service()
      .hub_service()
      .local_file_exists(&model.repo, &model.filename, None)
  {
    return Err(PullError::FileAlreadyExists {
      repo: model.repo.to_string(),
      filename: model.filename.clone(),
      snapshot: "main".to_string(),
    })?;
  }

  // Check for existing pending download request
  let pending_downloads = state
    .app_service()
    .db_service()
    .find_download_request_by_repo_filename(&model.repo.to_string(), &model.filename)
    .await?
    .into_iter()
    .find(|r| r.status == DownloadStatus::Pending);

  if let Some(existing_request) = pending_downloads {
    return Ok((StatusCode::OK, Json(existing_request)));
  }

  let download_request = DownloadRequest::new_pending(
    model.repo.to_string().as_str(),
    model.filename.as_str(),
    state.app_service().time_service().utc_now(),
  );
  state
    .app_service()
    .db_service()
    .create_download_request(&download_request)
    .await?;

  let app_service = state.app_service().clone();
  let request_id = download_request.id.clone();

  spawn(async move {
    let command = PullCommand::ByAlias { alias };
    let result = command
      .execute(
        app_service.clone(),
        Some(Progress::Database(DatabaseProgress::new(
          app_service.db_service().clone(),
          request_id.clone(),
        ))),
      )
      .await;
    update_download_status(app_service, request_id, result).await;
  });

  Ok((StatusCode::CREATED, Json(download_request)))
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
                 "code": "item_not_found"
             }
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn get_download_status_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<DownloadRequest>, ApiError> {
  let download_request = state
    .app_service()
    .db_service()
    .get_download_request(&id)
    .await?
    .ok_or_else(|| ItemNotFound::new(id, "download_requests".to_string()))?;

  Ok(Json(download_request))
}

async fn update_download_status(
  app_service: Arc<dyn AppService>,
  request_id: String,
  result: Result<(), commands::PullCommandError>,
) {
  let mut download_request = app_service
    .db_service()
    .get_download_request(&request_id)
    .await
    .expect("Failed to get download request")
    .expect("Download request not found");

  let (status, error) = match result {
    Ok(_) => (DownloadStatus::Completed, None),
    Err(e) => {
      let api_error: ApiError = e.into();
      (DownloadStatus::Error, Some(api_error.to_string()))
    }
  };
  download_request.status = status;
  download_request.error = error;
  download_request.updated_at = Utc::now();

  app_service
    .db_service()
    .update_download_request(&download_request)
    .await
    .expect("Failed to update download request");
}
