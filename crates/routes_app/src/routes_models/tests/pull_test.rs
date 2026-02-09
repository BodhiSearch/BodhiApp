use crate::{
  create_pull_request_handler, get_download_status_handler, list_downloads_handler,
  wait_for_event, PaginatedDownloadResponse,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{get, post},
  Router,
};
use mockall::predicate::{always, eq};
use objs::{HubFile, Repo};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::{
  db::{DownloadRequest, DownloadStatus, ModelRepository},
  test_utils::{
    app_service_stub_builder, test_db_service, test_hf_service, AppServiceStubBuilder,
    TestDbService, TestHfService,
  },
  AppService,
};
use std::{sync::Arc, time::Duration};
use tower::ServiceExt;

fn test_router(service: Arc<dyn AppService>) -> Router {
  let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::new()), service);
  Router::new()
    .route("/modelfiles/pull", post(create_pull_request_handler))
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
#[anyhow_trace]
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
    .oneshot(Request::post("/modelfiles/pull").json(&payload)?)
    .await?;

  assert_eq!(StatusCode::CREATED, response.status());
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
#[anyhow_trace]
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
    .oneshot(Request::post("/modelfiles/pull").json(&payload)?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let error_body = response.json::<Value>().await?;
  assert_eq!(
    "pull_error-file_already_exists",
    error_body["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
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
    .oneshot(Request::post("/modelfiles/pull").json(&payload)?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());
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
#[anyhow_trace]
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
      Request::get(&format!("/modelfiles/pull/status/{}", test_request.id)).body(Body::empty())?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let download_request = response.json::<DownloadRequest>().await?;
  assert_eq!(download_request.id, test_request.id);
  assert_eq!(download_request.status, DownloadStatus::Pending);

  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
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
    .oneshot(Request::get("/modelfiles/pull/status/non_existent_id").body(Body::empty())?)
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    "db_error-item_not_found",
    response["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
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
    .oneshot(Request::get("/modelfiles/pull/downloads?page=1&page_size=10").body(Body::empty())?)
    .await?;

  assert_eq!(StatusCode::OK, response.status());

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

// Auth tier tests (merged from tests/routes_models_pull_auth_test.rs)

#[anyhow_trace]
#[rstest]
#[case::list_downloads("GET", "/bodhi/v1/modelfiles/pull")]
#[case::create_pull_request("POST", "/bodhi/v1/modelfiles/pull")]
#[case::get_download_status("GET", "/bodhi/v1/modelfiles/pull/some-id")]
#[tokio::test]
async fn test_pull_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_pull_endpoints_reject_insufficient_role(
  #[values("resource_user")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/modelfiles/pull"),
    ("POST", "/bodhi/v1/modelfiles/pull"),
    ("GET", "/bodhi/v1/modelfiles/pull/some-id")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "{role} should be forbidden from {method} {path}"
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_pull_endpoints_allow_power_user_and_above(
  #[values("resource_power_user", "resource_manager", "resource_admin")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/modelfiles/pull"),
    ("GET", "/bodhi/v1/modelfiles/pull/some-id")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  // Both GET endpoints return 200/404 from real services (proves auth passed)
  assert!(
    response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
    "Expected 200 or 404, got {}",
    response.status()
  );
  Ok(())
}
