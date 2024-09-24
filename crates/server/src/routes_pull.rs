use crate::{HttpError, HttpErrorBuilder, RouterState};
use axum::{
  extract::{rejection::JsonRejection, Path, State},
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::WithRejection;
use chrono::Utc;
use commands::{PullCommand, PullCommandError};
use hyper::StatusCode;
use objs::Repo;
use serde::{Deserialize, Serialize};
use services::{
  db::{DownloadRequest, DownloadStatus},
  AppService,
};
use std::sync::Arc;
use tokio::spawn;
use validator::Validate;

pub fn pull_router() -> Router<Arc<dyn RouterState>> {
  Router::new()
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

#[derive(Debug, thiserror::Error)]
pub enum PullError {
  #[error("invalid request: {0}")]
  JsonRejection(#[from] JsonRejection),
  #[error("failed to pull: {0}")]
  CommandError(String),
  #[error("invalid pull request")]
  InvalidRequest,
}

impl From<PullError> for HttpError {
  fn from(err: PullError) -> Self {
    let (r#type, code, msg, status) = match err {
      PullError::JsonRejection(msg) => (
        "invalid_request_error",
        "invalid_value",
        msg.to_string(),
        StatusCode::BAD_REQUEST,
      ),
      PullError::CommandError(msg) => ("pull_error", "command_error", msg, StatusCode::BAD_REQUEST),
      PullError::InvalidRequest => (
        "invalid_request_error",
        "invalid_request",
        "Invalid pull request".to_string(),
        StatusCode::BAD_REQUEST,
      ),
    };
    HttpErrorBuilder::default()
      .status_code(status)
      .r#type(r#type)
      .code(code)
      .message(&msg)
      .build()
      .unwrap()
  }
}

impl IntoResponse for PullError {
  fn into_response(self) -> Response {
    let err = HttpError::from(self);
    (err.status_code, Json(err.body)).into_response()
  }
}

async fn pull_by_repo_file_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<PullRepoFileRequest>, PullError>,
) -> Result<(StatusCode, Json<DownloadRequest>), PullError> {
  let repo = Repo::try_from(payload.repo.clone()).map_err(|_| PullError::InvalidRequest)?;

  // Check if the file is already downloaded
  if let Ok(true) =
    state
      .app_service()
      .hub_service()
      .local_file_exists(&repo, &payload.filename, None)
  {
    return Err(PullError::CommandError(
      "File is already downloaded".to_string(),
    ));
  }

  // Check for existing pending download request
  let pending_downloads = state
    .app_service()
    .db_service()
    .list_pending_downloads()
    .await
    .map_err(|e| PullError::CommandError(e.to_string()))?;

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
    .await
    .map_err(|e| PullError::CommandError(e.to_string()))?;

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
) -> Result<(StatusCode, Json<DownloadRequest>), PullError> {
  let model = match state.app_service().data_service().find_remote_model(&alias) {
    Ok(Some(model)) => model,
    Ok(None) => {
      return Err(PullError::CommandError(format!(
        "Remote model not found: {alias}"
      )))
    }
    Err(err) => {
      return Err(PullError::CommandError(format!(
        "Error fetching remote model: {err}"
      )))
    }
  };

  // Check if the file is already downloaded
  if let Ok(true) =
    state
      .app_service()
      .hub_service()
      .local_file_exists(&model.repo, &model.filename, None)
  {
    return Err(PullError::CommandError(
      "File is already downloaded".to_string(),
    ));
  }

  // Check for existing pending download request
  let pending_downloads = state
    .app_service()
    .db_service()
    .list_pending_downloads()
    .await
    .map_err(|e| PullError::CommandError(e.to_string()))?;

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
    .await
    .map_err(|e| PullError::CommandError(e.to_string()))?;

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
) -> Result<Json<DownloadRequest>, PullError> {
  let download_request = state
    .app_service()
    .db_service()
    .get_download_request(&id)
    .await
    .map_err(|e| PullError::CommandError(e.to_string()))?
    .ok_or_else(|| PullError::CommandError("Download request not found".to_string()))?;

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

  download_request.status = match result {
    Ok(_) => DownloadStatus::Completed,
    Err(e) => DownloadStatus::Error(e.to_string()),
  };
  download_request.updated_at = Utc::now();

  app_service
    .db_service()
    .update_download_request(&download_request)
    .await
    .expect("Failed to update download request");
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{test_utils::ResponseTestExt, ErrorBody, MockRouterState};
  use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
  };
  use mockall::predicate::eq;
  use objs::{test_utils::temp_home, HubFile, TOKENIZER_CONFIG_JSON};
  use rstest::rstest;
  use services::{
    db::DbService,
    test_utils::{db_service_in, db_service_with_events, AppServiceStub, AppServiceStubBuilder},
    HubService, MockHubService,
  };
  use std::{sync::Arc, time::Duration};
  use tempfile::TempDir;
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

  async fn test_app_service(
    mock_hub_service: Arc<dyn HubService>,
    db_service: Arc<dyn DbService>,
  ) -> AppServiceStub {
    let temp_home = Arc::new(tempfile::tempdir().unwrap());
    AppServiceStubBuilder::default()
      .temp_home(temp_home)
      .hub_service(mock_hub_service)
      .with_data_service()
      .db_service(db_service)
      .build()
      .unwrap()
  }

  fn test_router(service: Arc<dyn AppService>) -> Router {
    let mut router_state = MockRouterState::new();
    router_state
      .expect_app_service()
      .returning(move || service.clone());
    Router::new()
      .route("/modelfiles/pull", post(pull_by_repo_file_handler))
      .route("/modelfiles/pull/:alias", post(pull_by_alias_handler))
      .route(
        "/modelfiles/pull/status/:id",
        get(get_download_status_handler),
      )
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[tokio::test]
  async fn test_pull_by_repo_file_success(temp_home: TempDir) -> anyhow::Result<()> {
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_local_file_exists()
      .with(
        eq(Repo::testalias()),
        eq("testalias.Q8_0.gguf".to_string()),
        eq(None),
      )
      .returning(|_, _, _| Ok(false));
    mock_hub_service
      .expect_download()
      .with(
        eq(Repo::testalias()),
        eq("testalias.Q8_0.gguf".to_string()),
        eq(None),
      )
      .returning(|_, _, _| Ok(HubFile::testalias()));
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(Repo::llama3()),
        eq("testalias.Q8_0.gguf".to_string()),
        eq(None),
      )
      .once()
      .returning(|_, _, _| Ok(None));
    let mock_hub_service = Arc::new(mock_hub_service);
    let db_service = db_service_with_events(&temp_home).await;
    let mut rx = db_service.subscribe();
    let db_service = Arc::new(db_service);
    let app_service = test_app_service(mock_hub_service, db_service.clone()).await;
    let router = test_router(Arc::new(app_service));
    let payload = serde_json::json!({
        "repo": "MyFactory/testalias-gguf",
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

    assert_eq!(response.status(), StatusCode::CREATED);
    let download_request = response.json::<DownloadRequest>().await?;
    assert_eq!(download_request.repo, Repo::testalias().to_string());
    assert_eq!(download_request.filename, "testalias.Q8_0.gguf");
    assert_eq!(download_request.status, DownloadStatus::Pending);

    // Wait for the update_download_request event
    let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(100));

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
  #[tokio::test]
  async fn test_pull_by_repo_file_already_downloaded(temp_home: TempDir) -> anyhow::Result<()> {
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_local_file_exists()
      .with(
        eq(Repo::testalias()),
        eq("testalias.Q8_0.gguf".to_string()),
        eq(None),
      )
      .returning(|_, _, _| Ok(true));
    let db_service = Arc::new(db_service_in(&temp_home).await);
    let app_service = test_app_service(Arc::new(mock_hub_service), db_service).await;
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
    let error_body = response.json::<ErrorBody>().await?;
    assert_eq!(error_body.code, Some("command_error".to_string()));
    assert_eq!(error_body.message, "File is already downloaded");
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_pull_by_repo_file_existing_pending_download(
    temp_home: TempDir,
  ) -> anyhow::Result<()> {
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_local_file_exists()
      .returning(|_, _, _| Ok(false));
    let db_service = db_service_in(&temp_home).await;
    let pending_request = DownloadRequest::new_pending(
      Repo::testalias().to_string(),
      "testalias.Q8_0.gguf".to_string(),
    );
    db_service.create_download_request(&pending_request).await?;
    let db_service = Arc::new(db_service);
    let app_service = test_app_service(Arc::new(mock_hub_service), db_service.clone()).await;
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

    assert_eq!(response.status(), StatusCode::OK);
    let download_request = response.json::<DownloadRequest>().await?;
    assert_eq!(download_request.id, pending_request.id);
    assert_eq!(download_request.repo, Repo::testalias().to_string());
    assert_eq!(download_request.filename, "testalias.Q8_0.gguf");
    assert_eq!(download_request.status, DownloadStatus::Pending);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_pull_by_alias_success(temp_home: TempDir) -> anyhow::Result<()> {
    let mut mock_hub_service = MockHubService::new();
    mock_hub_service
      .expect_local_file_exists()
      .with(
        eq(Repo::testalias()),
        eq("testalias.Q8_0.gguf".to_string()),
        eq(None),
      )
      .returning(|_, _, _| Ok(false));
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(Repo::testalias()),
        eq("testalias.Q8_0.gguf".to_string()),
        eq(None),
      )
      .returning(|_, _, _| Ok(None));
    mock_hub_service
      .expect_find_local_file()
      .with(
        eq(Repo::llama3()),
        eq(TOKENIZER_CONFIG_JSON.to_string()),
        eq(None),
      )
      .returning(|_, _, _| Ok(Some(HubFile::llama3_tokenizer())));
    mock_hub_service
      .expect_local_file_exists()
      .with(
        eq(Repo::llama3()),
        eq(TOKENIZER_CONFIG_JSON.to_string()),
        eq(None),
      )
      .returning(|_, _, _| Ok(true));
    mock_hub_service
      .expect_download()
      .with(
        eq(Repo::testalias()),
        eq("testalias.Q8_0.gguf".to_string()),
        eq(None),
      )
      .returning(|_, _, _| Ok(HubFile::testalias()));
    let db_service = db_service_with_events(&temp_home).await;
    let mut rx = db_service.subscribe();
    let db_service = Arc::new(db_service);
    let app_service = test_app_service(Arc::new(mock_hub_service), db_service.clone()).await;
    let router = test_router(Arc::new(app_service));
    let response = router
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/modelfiles/pull/testalias:instruct")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::CREATED);
    let download_request = response.json::<DownloadRequest>().await?;
    assert_eq!(download_request.repo, Repo::testalias().to_string());
    assert_eq!(download_request.filename, "testalias.Q8_0.gguf");
    assert_eq!(download_request.status, DownloadStatus::Pending);

    let event_received = wait_for_event!(rx, "update_download_request", Duration::from_millis(100));
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
  #[tokio::test]
  async fn test_pull_by_alias_not_found(temp_home: TempDir) -> anyhow::Result<()> {
    let mock_hub_service = MockHubService::new();
    let db_service = db_service_in(&temp_home).await;
    let app_service = test_app_service(Arc::new(mock_hub_service), Arc::new(db_service)).await;
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
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error_body = response.json::<ErrorBody>().await?;
    assert_eq!(error_body.code, Some("command_error".to_string()));
    assert_eq!(
      error_body.message,
      "Remote model not found: non_existent:alias"
    );

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_download_status_success(temp_home: TempDir) -> anyhow::Result<()> {
    let mock_hub_service = MockHubService::new();
    let db_service = db_service_in(&temp_home).await;
    let db_service = Arc::new(db_service);
    let app_service = test_app_service(Arc::new(mock_hub_service), db_service.clone()).await;
    let router = test_router(Arc::new(app_service));

    // Create a test download request
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
  #[tokio::test]
  async fn test_get_download_status_not_found(temp_home: TempDir) -> anyhow::Result<()> {
    let mock_hub_service = MockHubService::new();
    let db_service = db_service_in(&temp_home).await;
    let app_service = test_app_service(Arc::new(mock_hub_service), Arc::new(db_service)).await;
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error_body = response.json::<ErrorBody>().await?;
    assert_eq!(error_body.code, Some("command_error".to_string()));
    assert_eq!(error_body.message, "Download request not found");

    Ok(())
  }
}
