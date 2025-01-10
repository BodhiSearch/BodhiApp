use axum::http::StatusCode;
use axum::{
  extract::{Path, Query, State},
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::WithRejection;
use chrono::Utc;
use commands::{PullCommand, PullCommandError};
use objs::{ApiError, AppError, ErrorType, ObjValidationError, Repo};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::db::ItemNotFound;
use services::RemoteModelNotFoundError;
use services::{
  db::{DownloadRequest, DownloadStatus},
  AppService,
};
use std::sync::Arc;
use tokio::spawn;
use validator::Validate;

pub fn pull_router() -> Router<Arc<dyn RouterState>> {
  Router::new()
    .route("/modelfiles/pull/downloads", get(list_downloads_handler))
    .route("/modelfiles/pull", post(pull_by_repo_file_handler))
    .route("/modelfiles/pull/:alias", post(pull_by_alias_handler))
    .route(
      "/modelfiles/pull/status/:id",
      get(get_download_status_handler),
    )
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PullRepoFileRequest {
  repo: String,
  filename: String,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum PullError {
  #[error("file_already_exists")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  FileAlreadyExists {
    repo: String,
    filename: String,
    snapshot: String,
  },
  #[error(transparent)]
  PullCommand(#[from] PullCommandError),
  #[error(transparent)]
  ObjValidation(#[from] ObjValidationError),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDownloadsQuery {
  #[serde(default = "default_page")]
  pub page: u32,
  #[serde(default = "default_page_size")]
  pub page_size: u32,
}

fn default_page() -> u32 {
  1
}

fn default_page_size() -> u32 {
  30
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDownloadsResponse {
  pub data: Vec<DownloadRequest>,
  pub total: u32,
  pub page: u32,
  pub page_size: u32,
}

async fn list_downloads_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(query): Query<ListDownloadsQuery>,
) -> Result<Json<ListDownloadsResponse>, ApiError> {
  let downloads = state
    .app_service()
    .db_service()
    .list_all_downloads()
    .await?;

  // Calculate pagination
  let total = downloads.len() as u32;
  let start = ((query.page - 1) * query.page_size) as usize;
  let end = (start + query.page_size as usize).min(downloads.len());
  let paged_downloads = downloads[start..end].to_vec();

  Ok(Json(ListDownloadsResponse {
    data: paged_downloads,
    total,
    page: query.page,
    page_size: query.page_size,
  }))
}

async fn pull_by_repo_file_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<PullRepoFileRequest>, ApiError>,
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
    .list_all_downloads()
    .await?
    .into_iter()
    .filter(|r| r.status == DownloadStatus::Pending)
    .collect::<Vec<_>>();

  if let Some(existing_request) = pending_downloads
    .into_iter()
    .find(|r| r.repo == payload.repo && r.filename == payload.filename)
  {
    return Ok((StatusCode::OK, Json(existing_request)));
  }

  let download_request = DownloadRequest::new_pending(repo.to_string(), payload.filename.clone());

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
    let result = command.execute(app_service.clone());
    update_download_status(app_service, request_id, result).await;
  });

  Ok((StatusCode::CREATED, Json(download_request)))
}

async fn pull_by_alias_handler(
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
    .list_all_downloads()
    .await?
    .into_iter()
    .filter(|r| r.status == DownloadStatus::Pending)
    .collect::<Vec<_>>();

  if let Some(existing_request) = pending_downloads
    .into_iter()
    .find(|r| r.repo == model.repo.to_string() && r.filename == model.filename)
  {
    return Ok((StatusCode::OK, Json(existing_request)));
  }

  let download_request =
    DownloadRequest::new_pending(model.repo.to_string(), model.filename.clone());
  state
    .app_service()
    .db_service()
    .create_download_request(&download_request)
    .await?;

  let app_service = state.app_service().clone();
  let request_id = download_request.id.clone();

  spawn(async move {
    let command = PullCommand::ByAlias { alias };
    let result = command.execute(app_service.clone());
    update_download_status(app_service, request_id, result).await;
  });

  Ok((StatusCode::CREATED, Json(download_request)))
}

async fn get_download_status_handler(
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
  result: Result<(), PullCommandError>,
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

#[cfg(test)]
mod tests {
  use crate::ListDownloadsResponse;

  use super::{
    get_download_status_handler, list_downloads_handler, pull_by_alias_handler,
    pull_by_repo_file_handler,
  };
  use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    routing::{get, post},
    Router,
  };
  use mockall::predicate::eq;
  use objs::{
    test_utils::setup_l10n, FluentLocalizationService, HubFile, Repo, TOKENIZER_CONFIG_JSON,
  };
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
  use services::{
    db::{DbService, DownloadRequest, DownloadStatus},
    test_utils::{
      app_service_stub_builder, test_db_service, test_hf_service, AppServiceStubBuilder,
      TestDbService, TestHfService,
    },
    AppService,
  };
  use std::{sync::Arc, time::Duration};
  use tower::ServiceExt;

  macro_rules! wait_for_event {
    ($rx:expr, $event_name:expr, $timeout:expr) => {{
      loop {
        tokio::select! {
            event = $rx.recv() => {
                match event {
                    Ok(e) if e == $event_name => break true,
                    _ => continue
                }
            }
            _ = tokio::time::sleep($timeout) => break false
        }
      }
    }};
  }

  fn test_router(service: Arc<dyn AppService>) -> Router {
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::new()), service);
    Router::new()
      .route("/modelfiles/pull", post(pull_by_repo_file_handler))
      .route("/modelfiles/pull/:alias", post(pull_by_alias_handler))
      .route(
        "/modelfiles/pull/status/:id",
        get(get_download_status_handler),
      )
      .route("/modelfiles/pull/downloads", get(list_downloads_handler))
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_pull_by_repo_file_success(
    mut test_hf_service: TestHfService,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    test_hf_service
      .expect_download()
      .with(eq(Repo::testalias()), eq(Repo::testalias_q4()), eq(None))
      .returning(|_, _, _| Ok(HubFile::testalias()));
    let mut rx = db_service.subscribe();
    let db_service = Arc::new(db_service);
    let app_service = app_service_stub_builder
      .db_service(db_service.clone())
      .hub_service(Arc::new(test_hf_service))
      .build()?;
    let router = test_router(Arc::new(app_service));
    let payload = serde_json::json!({
        "repo": "MyFactory/testalias-gguf",
        "filename": Repo::testalias_q4()
    });

    let response = router
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/modelfiles/pull")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::CREATED);
    let download_request = response.json::<DownloadRequest>().await?;
    assert_eq!(download_request.repo, Repo::testalias().to_string());
    assert_eq!(download_request.filename, Repo::testalias_q4());
    assert_eq!(download_request.status, DownloadStatus::Pending);

    // Wait for the update_download_request event
    let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(500));

    assert!(
      event_received,
      "Timed out waiting for update_download_request event"
    );

    let final_status = db_service
      .get_download_request(&download_request.id)
      .await?
      .unwrap();
    assert_eq!(final_status.status, DownloadStatus::Completed);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_pull_by_repo_file_already_downloaded(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    test_hf_service: TestHfService,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(db_service);
    let app_service = app_service_stub_builder
      .db_service(db_service.clone())
      .hub_service(Arc::new(test_hf_service))
      .build()?;
    let router = test_router(Arc::new(app_service));
    let payload = serde_json::json!({
        "repo": Repo::testalias().to_string(),
        "filename": "testalias.Q8_0.gguf"
    });

    let response = router
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/modelfiles/pull")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error_body = response.json::<Value>().await?;
    assert_eq!(
      error_body,
      json! {{
        "error": {
          "message": "file \u{2068}testalias.Q8_0.gguf\u{2069} already exists in repo \u{2068}MyFactory/testalias-gguf\u{2069} with snapshot \u{2068}main\u{2069}",
          "code": "pull_error-file_already_exists",
          "type": "invalid_request_error"
        }
      }}
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_pull_by_repo_file_existing_pending_download(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    test_hf_service: TestHfService,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    let pending_request =
      DownloadRequest::new_pending(Repo::testalias().to_string(), Repo::testalias_q4());
    db_service.create_download_request(&pending_request).await?;
    let db_service = Arc::new(db_service);
    let app_service = app_service_stub_builder
      .db_service(db_service.clone())
      .hub_service(Arc::new(test_hf_service))
      .build()?;

    let router = test_router(Arc::new(app_service));

    let payload = serde_json::json!({
        "repo": Repo::testalias().to_string(),
        "filename": Repo::testalias_q4()
    });

    let response = router
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/modelfiles/pull")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let download_request = response.json::<DownloadRequest>().await?;
    assert_eq!(download_request.id, pending_request.id);
    assert_eq!(download_request.repo, Repo::testalias().to_string());
    assert_eq!(download_request.filename, Repo::testalias_q4());
    assert_eq!(download_request.status, DownloadStatus::Pending);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_pull_by_alias_success(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    mut test_hf_service: TestHfService,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    test_hf_service
      .expect_download()
      .with(eq(Repo::testalias()), eq(Repo::testalias_q4()), eq(None))
      .returning(|_, _, _| Ok(HubFile::testalias()));
    test_hf_service
      .expect_download()
      .with(
        eq(Repo::llama3_tokenizer()),
        eq(TOKENIZER_CONFIG_JSON),
        eq(None),
      )
      .return_once(|_, _, _| Ok(HubFile::llama3_tokenizer()));
    let mut rx = db_service.subscribe();
    let db_service = Arc::new(db_service);
    let app_service = app_service_stub_builder
      .db_service(db_service.clone())
      .hub_service(Arc::new(test_hf_service))
      .build()?;
    let router = test_router(Arc::new(app_service));
    let response = router
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/modelfiles/pull/testalias:q4_instruct")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::CREATED);
    let download_request = response.json::<DownloadRequest>().await?;
    assert_eq!(download_request.repo, Repo::testalias().to_string());
    assert_eq!(download_request.filename, Repo::testalias_q4());
    assert_eq!(download_request.status, DownloadStatus::Pending);

    let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(500));
    assert!(
      event_received,
      "Timed out waiting for update_download_request event"
    );

    let final_status = db_service
      .get_download_request(&download_request.id)
      .await?
      .unwrap();
    assert_eq!(final_status.status, DownloadStatus::Completed);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_pull_by_alias_not_found(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    test_hf_service: TestHfService,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    let app_service = app_service_stub_builder
      .db_service(Arc::new(db_service))
      .hub_service(Arc::new(test_hf_service))
      .build()?;
    let router = test_router(Arc::new(app_service));
    let response = router
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/modelfiles/pull/non_existent:alias")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let response = response.json::<Value>().await?;
    assert_eq!(
      response,
      json! {{
        "error": {
          "message": "remote model alias '\u{2068}non_existent:alias\u{2069}' not found, check your alias and try again",
          "type": "not_found_error",
          "code": "remote_model_not_found_error"
        }
      }}
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_download_status_success(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    test_hf_service: TestHfService,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(db_service);
    let app_service = app_service_stub_builder
      .db_service(db_service.clone())
      .hub_service(Arc::new(test_hf_service))
      .build()?;
    let router = test_router(Arc::new(app_service));
    let test_request =
      DownloadRequest::new_pending("test/repo".to_string(), "test.gguf".to_string());
    db_service.create_download_request(&test_request).await?;

    let response = router
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .uri(format!("/modelfiles/pull/status/{}", test_request.id))
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let download_request = response.json::<DownloadRequest>().await?;
    assert_eq!(download_request.id, test_request.id);
    assert_eq!(download_request.status, DownloadStatus::Pending);

    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_download_status_not_found(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    test_hf_service: TestHfService,
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    let app_service = app_service_stub_builder
      .db_service(Arc::new(db_service))
      .hub_service(Arc::new(test_hf_service))
      .build()?;

    let router = test_router(Arc::new(app_service));
    let response = router
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .uri("/modelfiles/pull/status/non_existent_id")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let response = response.json::<Value>().await?;
    assert_eq!(
      response,
      json! {{
        "error": {
          "message": "item '\u{2068}non_existent_id\u{2069}' of type '\u{2068}download_requests\u{2069}' not found in db",
          "type": "not_found_error",
          "code": "item_not_found"
        }
      }}
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_downloads(
    #[future]
    #[from(test_db_service)]
    db_service: TestDbService,
    #[future] mut app_service_stub_builder: AppServiceStubBuilder,
  ) -> anyhow::Result<()> {
    // Create test downloads with different statuses
    let download1 =
      DownloadRequest::new_pending("test/repo1".to_string(), "file1.gguf".to_string());
    let mut download2 =
      DownloadRequest::new_pending("test/repo2".to_string(), "file2.gguf".to_string());
    let mut download3 =
      DownloadRequest::new_pending("test/repo3".to_string(), "file3.gguf".to_string());

    let db_service = Arc::new(db_service);
    db_service.create_download_request(&download1).await?;
    db_service.create_download_request(&download2).await?;
    db_service.create_download_request(&download3).await?;

    // Update status of download2 to completed and download3 to error
    download2.status = DownloadStatus::Completed;
    download3.status = DownloadStatus::Error;
    download3.error = Some("test error".to_string());
    db_service.update_download_request(&download2).await?;
    db_service.update_download_request(&download3).await?;

    let app_service = app_service_stub_builder.db_service(db_service).build()?;

    let router = test_router(Arc::new(app_service));

    let response = router
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .uri("/modelfiles/pull/downloads?page=1&page_size=10")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.json::<ListDownloadsResponse>().await?;
    assert_eq!(body.data.len(), 3);
    assert_eq!(body.total, 3);
    assert_eq!(body.page, 1);
    assert_eq!(body.page_size, 10);

    // Verify download details - should be sorted by updated_at DESC
    let downloads = body.data;
    assert_eq!(downloads[2].repo, "test/repo3");
    assert_eq!(downloads[2].filename, "file3.gguf");
    assert_eq!(downloads[2].status, DownloadStatus::Error);
    assert_eq!(downloads[2].error, Some("test error".to_string()));

    assert_eq!(downloads[1].repo, "test/repo2");
    assert_eq!(downloads[1].filename, "file2.gguf");
    assert_eq!(downloads[1].status, DownloadStatus::Completed);
    assert_eq!(downloads[1].error, None);

    assert_eq!(downloads[0].repo, "test/repo1");
    assert_eq!(downloads[0].filename, "file1.gguf");
    assert_eq!(downloads[0].status, DownloadStatus::Pending);
    assert_eq!(downloads[0].error, None);

    Ok(())
  }
}
