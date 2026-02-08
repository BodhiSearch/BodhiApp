use crate::{
  create_pull_request_handler, get_download_status_handler, list_downloads_handler,
  pull_by_alias_handler, PaginatedDownloadResponse,
};
use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  routing::{get, post},
  Router,
};
use mockall::predicate::{always, eq};
use objs::{HubFile, Repo};
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
    .route("/modelfiles/pull", post(create_pull_request_handler))
    .route("/modelfiles/pull/{alias}", post(pull_by_alias_handler))
    .route(
      "/modelfiles/pull/status/{id}",
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
    .inner_mock
    .expect_download()
    .with(
      eq(Repo::testalias()),
      eq(Repo::testalias_model_q4()),
      eq(None),
      always(),
    )
    .times(1)
    .return_once(|_, _, _, _| Ok(HubFile::testalias()));
  let mut rx = db_service.subscribe();
  let db_service = Arc::new(db_service);
  let app_service = app_service_stub_builder
    .db_service(db_service.clone())
    .hub_service(Arc::new(test_hf_service))
    .build()?;
  let router = test_router(Arc::new(app_service));
  let payload = serde_json::json!({
      "repo": "MyFactory/testalias-gguf",
      "filename": Repo::testalias_model_q4()
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
  assert_eq!(download_request.filename, Repo::testalias_model_q4());
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
        "message": "File 'testalias.Q8_0.gguf' already exists in 'MyFactory/testalias-gguf'.",
        "code": "pull_error-file_already_exists",
        "type": "invalid_request_error",
        "param": {
          "filename": "testalias.Q8_0.gguf",
          "repo": "MyFactory/testalias-gguf",
          "snapshot": "main"
        }
      }
    }}
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_pull_by_repo_file_existing_pending_download(
  test_hf_service: TestHfService,
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
) -> anyhow::Result<()> {
  let pending_request = DownloadRequest::new_pending(
    &Repo::testalias().to_string(),
    &Repo::testalias_model_q4(),
    db_service.now(),
  );
  db_service.create_download_request(&pending_request).await?;
  let db_service = Arc::new(db_service);
  let app_service = app_service_stub_builder
    .db_service(db_service.clone())
    .hub_service(Arc::new(test_hf_service))
    .build()?;

  let router = test_router(Arc::new(app_service));

  let payload = serde_json::json!({
      "repo": Repo::testalias().to_string(),
      "filename": Repo::testalias_model_q4()
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
  assert_eq!(download_request.filename, Repo::testalias_model_q4());
  assert_eq!(download_request.status, DownloadStatus::Pending);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_pull_by_alias_success(
  mut test_hf_service: TestHfService,
  #[future]
  #[from(test_db_service)]
  db_service: TestDbService,
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
) -> anyhow::Result<()> {
  test_hf_service
    .inner_mock
    .expect_download()
    .with(
      eq(Repo::testalias()),
      eq(Repo::testalias_model_q4()),
      eq(None),
      always(),
    )
    .times(1)
    .return_once(|_, _, _, _| Ok(HubFile::testalias()));
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
  assert_eq!(download_request.filename, Repo::testalias_model_q4());
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
        "message": "Remote model 'non_existent:alias' not found. Check the alias name and try again.",
        "type": "not_found_error",
        "code": "remote_model_not_found_error",
        "param": {
          "alias": "non_existent:alias"
        }
      }
    }}
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_get_download_status_success(
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
  let test_request = DownloadRequest::new_pending("test/repo", "test.gguf", db_service.now());
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
        "message": "Item 'non_existent_id' of type 'download_requests' not found.",
        "type": "not_found_error",
        "code": "item_not_found",
        "param": {
          "id": "non_existent_id",
          "item_type": "download_requests"
        }
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
  let download1 = DownloadRequest::new_pending("test/repo1", "file1.gguf", db_service.now());
  let mut download2 = DownloadRequest::new_pending("test/repo2", "file2.gguf", db_service.now());
  let mut download3 = DownloadRequest::new_pending("test/repo3", "file3.gguf", db_service.now());

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

  let body = response.json::<PaginatedDownloadResponse>().await?;
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
