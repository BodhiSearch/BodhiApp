use crate::{auth_callback_handler, RedirectResponse};
use anyhow_trace::anyhow_trace;
use auth_middleware::generate_random_string;
use axum::body::to_bytes;
use axum::{
  http::{status::StatusCode, Request},
  response::Response,
  routing::post,
  Router,
};
use mockito::{Matcher, Server};
use oauth2::PkceCodeVerifier;
use objs::test_utils::temp_bodhi_home;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::{test_utils::RequestTestExt, DefaultRouterState, MockSharedContext};
use services::{
  test_utils::{
    test_auth_service, token, AppServiceStub, AppServiceStubBuilder, SettingServiceStub,
  },
  AppService, AppStatus, SqliteSessionService, BODHI_AUTH_URL,
};
use services::{BODHI_HOST, BODHI_PORT, BODHI_SCHEME};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_callback_handler_resource_admin(
  temp_bodhi_home: TempDir,
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let mut server = Server::new_async().await;
  let auth_server_url = server.url();
  let id = Id::default();
  let state = generate_random_string(32); // Use simple random state like the actual implementation
  let app_service =
    setup_app_service_resource_admin(&temp_bodhi_home, &id, &auth_server_url, &state).await?;
  setup_auth_server_mocks_resource_admin(&mut server, &token).await;
  let result = execute_auth_callback(&id, app_service.clone(), &state).await?;
  assert_login_callback_result_resource_admin(result, app_service).await?;
  Ok(())
}

async fn setup_app_service_resource_admin(
  temp_bodhi_home: &TempDir,
  id: &Id,
  auth_server_url: &str,
  state: &str,
) -> anyhow::Result<Arc<AppServiceStub>> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = SqliteSessionService::build_session_service(dbfile).await;
  let mut record = Record {
    id: *id,
    data: maplit::hashmap! {
      "oauth_state".to_string() => Value::String(state.to_string()),
      "pkce_verifier".to_string() => Value::String("test_pkce_verifier".to_string()),
      "callback_url".to_string() => Value::String("http://frontend.localhost:3000/ui/auth/callback".to_string()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service.session_store.create(&mut record).await?;
  let session_service = Arc::new(session_service);
  let auth_service = Arc::new(test_auth_service(auth_server_url));
  let setting_service = Arc::new(
    SettingServiceStub::default()
      .append_settings(HashMap::from([
        (BODHI_SCHEME.to_string(), "http".to_string()),
        (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
        (BODHI_PORT.to_string(), "3000".to_string()),
        (BODHI_AUTH_URL.to_string(), auth_server_url.to_string()),
      ]))
      .await,
  );
  let mut builder = AppServiceStubBuilder::default();
  builder
    .auth_service(auth_service)
    .setting_service(setting_service)
    .with_sqlite_session_service(session_service);
  builder
    .with_app_instance(services::AppInstance {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
      scope: "scope_test_client_id".to_string(),
      status: AppStatus::ResourceAdmin,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    })
    .await;
  let app_service = builder.build().await?;
  Ok(Arc::new(app_service))
}

async fn setup_auth_server_mocks_resource_admin(server: &mut Server, token: &str) {
  // Mock token endpoint for code exchange
  let code_verifier = PkceCodeVerifier::new("test_pkce_verifier".to_string());
  let code_secret = code_verifier.secret();
  server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "authorization_code".into()),
      Matcher::UrlEncoded("code".into(), "test_code".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
      Matcher::UrlEncoded(
        "redirect_uri".into(),
        "http://frontend.localhost:3000/ui/auth/callback".into(),
      ),
      Matcher::UrlEncoded("code_verifier".into(), code_secret.into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
        "access_token": token,
        "refresh_token": "initial_refresh_token",
        "token_type": "Bearer",
        "expires_in": 300,
      })
      .to_string(),
    )
    .create_async()
    .await;

  // Mock token endpoint for client credentials
  server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "client_credentials".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
        "access_token": "client_access_token",
        "token_type": "Bearer",
        "expires_in": 300,
      })
      .to_string(),
    )
    .create_async()
    .await;

  // Mock make-resource-admin endpoint
  server
    .mock(
      "POST",
      "/realms/test-realm/bodhi/resources/make-resource-admin",
    )
    .match_header("Authorization", "Bearer client_access_token")
    .match_body(Matcher::Regex(r#"\{"user_id":"[^"]+"\}"#.to_string()))
    .with_status(200)
    .with_body("{}")
    .create_async()
    .await;

  // Mock token refresh endpoint
  server
    .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
    .match_body(Matcher::AllOf(vec![
      Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
      Matcher::UrlEncoded("refresh_token".into(), "initial_refresh_token".into()),
      Matcher::UrlEncoded("client_id".into(), "test_client_id".into()),
      Matcher::UrlEncoded("client_secret".into(), "test_client_secret".into()),
    ]))
    .with_status(200)
    .with_body(
      json!({
        "access_token": "new_access_token",
        "refresh_token": "new_refresh_token",
        "token_type": "Bearer",
        "expires_in": 300,
      })
      .to_string(),
    )
    .create_async()
    .await;
}

async fn execute_auth_callback(
  id: &Id,
  app_service: Arc<AppServiceStub>,
  request_state: &str,
) -> Result<Response, anyhow::Error> {
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router: Router = Router::new()
    .route("/auth/callback", post(auth_callback_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);
  let request = Request::post("/auth/callback")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .json(json! {{
      "code": "test_code",
      "state": request_state,
    }})?;
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(response)
}

async fn assert_login_callback_result_resource_admin(
  response: Response,
  app_service: Arc<AppServiceStub>,
) -> anyhow::Result<()> {
  let body_bytes = to_bytes(response.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
  assert_eq!("http://frontend.localhost:3000/ui/chat", body.location);
  let app_instance_service = app_service.app_instance_service();
  let updated_status = app_instance_service.get_status().await?;
  assert_eq!(AppStatus::Ready, updated_status);
  Ok(())
}
