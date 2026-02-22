use crate::{logout_handler, RedirectResponse};
use anyhow_trace::anyhow_trace;
use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use axum_test::TestServer;
use objs::test_utils::temp_bodhi_home;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use server_core::{DefaultRouterState, MockSharedContext};
use services::{
  test_utils::{AppServiceStubBuilder, SessionTestExt},
  AppService, SqliteSessionService,
};
use std::sync::Arc;
use tempfile::TempDir;
use tower_sessions::Session;

pub async fn create_test_session_handler(session: Session) -> impl IntoResponse {
  session.insert("test", "test").await.unwrap();
  (StatusCode::CREATED, Json(json!({})))
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_logout_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile.clone()).await);
  let app_service: Arc<dyn AppService> = Arc::new(
    AppServiceStubBuilder::default()
      .with_sqlite_session_service(session_service.clone())
      .build()
      .await?,
  );

  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));

  let router = Router::new()
    .route("/app/logout", post(logout_handler))
    .route("/test/session/new", post(create_test_session_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let mut client = TestServer::new(router)?;
  client.save_cookies();

  let resp = client.post("/test/session/new").await;
  resp.assert_status(StatusCode::CREATED);
  let cookie = resp.cookie("bodhiapp_session_id");
  let session_id = cookie.value_trimmed();

  let record = session_service.get_session_record(session_id).await;
  assert!(record.is_some());

  let resp = client.post("/app/logout").await;
  resp.assert_status(StatusCode::OK);
  let body: RedirectResponse = resp.json();
  assert_eq!("http://localhost:1135/ui/login", body.location);
  let record = session_service.get_session_record(session_id).await;
  assert!(record.is_none());
  Ok(())
}
