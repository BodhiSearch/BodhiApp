use crate::{auth_initiate_handler, RedirectResponse};
use anyhow_trace::anyhow_trace;
use auth_middleware::{optional_auth_middleware, test_utils::RequestAuthContextExt, AuthContext};
use axum::body::to_bytes;
use axum::{
  http::{status::StatusCode, Request},
  middleware::from_fn_with_state,
  routing::post,
  Router,
};
use objs::test_utils::temp_bodhi_home;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use server_core::{test_utils::RequestTestExt, DefaultRouterState, MockSharedContext, RouterState};
use services::{
  test_utils::{expired_token, token, AppServiceStubBuilder, SettingServiceStub},
  AppService, DefaultSessionService, SessionService, BODHI_AUTH_REALM, BODHI_AUTH_URL,
};
use services::{BODHI_HOST, BODHI_PORT, BODHI_SCHEME};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use time::OffsetDateTime;
use tower::ServiceExt;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};
use url::Url;

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let callback_url = "http://localhost:3000/ui/auth/callback";
  let login_url = "http://test-id.getbodhi.app/realms/test-realm/protocol/openid-connect/auth";

  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "localhost".to_string()),
    (BODHI_PORT.to_string(), "3000".to_string()),
    (
      BODHI_AUTH_URL.to_string(),
      "http://test-id.getbodhi.app".to_string(),
    ),
    (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
  ]));
  let dbfile = temp_bodhi_home.path().join("test.db");
  let mut builder = AppServiceStubBuilder::default();
  builder
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await;
  builder
    .with_app_instance(services::AppInstance {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
      scope: "scope_test_client_id".to_string(),
      status: services::AppStatus::Ready,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    })
    .await;
  let app_service = builder.build().await?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let resp = router
    .oneshot(
      Request::post("/auth/initiate")
        .json(json! {{}})?
        .with_auth_context(AuthContext::Anonymous),
    )
    .await?;

  let status = resp.status();
  assert_eq!(status, StatusCode::CREATED);
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
  assert!(body.location.starts_with(login_url));

  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
  assert_eq!(
    Some("code"),
    query_params.get("response_type").map(|s| s.as_str())
  );
  assert_eq!(
    Some("test_client_id"),
    query_params.get("client_id").map(|s| s.as_str())
  );
  assert_eq!(
    Some(callback_url),
    query_params.get("redirect_uri").map(|s| s.as_str())
  );
  assert!(query_params.contains_key("state"));
  assert!(query_params.contains_key("code_challenge"));
  assert_eq!(
    Some("S256"),
    query_params
      .get("code_challenge_method")
      .map(|s| s.as_str())
  );
  assert_eq!(
    Some("openid email profile roles"),
    query_params.get("scope").map(|s| s.as_str())
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler_loopback_host_detection(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  // Configure with default 0.0.0.0 host (loopback)
  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
    (BODHI_PORT.to_string(), "1135".to_string()),
    (
      BODHI_AUTH_URL.to_string(),
      "http://test-id.getbodhi.app".to_string(),
    ),
    (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
  ]));

  let dbfile = temp_bodhi_home.path().join("test.db");
  let mut builder = AppServiceStubBuilder::default();
  builder
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await;
  builder
    .with_app_instance(services::AppInstance {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
      scope: "scope_test_client_id".to_string(),
      status: services::AppStatus::Ready,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    })
    .await;
  let app_service = builder.build().await?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  // Request with localhost:1135 Host header
  let resp = router
    .oneshot(
      Request::post("/auth/initiate")
        .header("Host", "localhost:1135")
        .json(json! {{}})?
        .with_auth_context(AuthContext::Anonymous),
    )
    .await?;

  assert_eq!(StatusCode::CREATED, resp.status());
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;

  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

  // Should use localhost from Host header instead of configured 0.0.0.0
  assert_eq!(
    Some("http://localhost:1135/ui/auth/callback"),
    query_params.get("redirect_uri").map(|s| s.as_str())
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler_network_host_usage(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  // Configure with default settings (no explicit public host)
  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (BODHI_SCHEME.to_string(), "http".to_string()),
    (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
    (BODHI_PORT.to_string(), "1135".to_string()),
    (
      BODHI_AUTH_URL.to_string(),
      "http://test-id.getbodhi.app".to_string(),
    ),
    (BODHI_AUTH_REALM.to_string(), "test-realm".to_string()),
  ]));

  let dbfile = temp_bodhi_home.path().join("test.db");
  let mut builder = AppServiceStubBuilder::default();
  builder
    .setting_service(Arc::new(setting_service))
    .build_session_service(dbfile)
    .await;
  builder
    .with_app_instance(services::AppInstance {
      client_id: "test_client_id".to_string(),
      client_secret: "test_client_secret".to_string(),
      scope: "scope_test_client_id".to_string(),
      status: services::AppStatus::Ready,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    })
    .await;
  let app_service = builder.build().await?;
  let app_service = Arc::new(app_service);
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  // Request with network host header
  let resp = router
    .oneshot(
      Request::post("/auth/initiate")
        .header("Host", "192.168.1.100:1135")
        .json(json! {{}})?
        .with_auth_context(AuthContext::Anonymous),
    )
    .await?;

  assert_eq!(StatusCode::CREATED, resp.status());
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;

  let url = Url::parse(&body.location)?;
  let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

  // Should now use the request host for network installation support
  assert_eq!(
    Some("http://192.168.1.100:1135/ui/auth/callback"),
    query_params.get("redirect_uri").map(|s| s.as_str())
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler_logged_in_redirects_to_home(
  temp_bodhi_home: TempDir,
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let (status, body) = auth_initiate_handler_with_token_response(temp_bodhi_home, token).await?;
  assert_eq!(status, StatusCode::OK);
  assert!(
    body
      .location
      .starts_with("http://frontend.localhost:3000/ui/chat"),
    "{} does not start with http://frontend.localhost:3000/ui/chat",
    body.location
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_initiate_handler_for_expired_token_redirects_to_login(
  temp_bodhi_home: TempDir,
  expired_token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = expired_token;
  let (status, body) = auth_initiate_handler_with_token_response(temp_bodhi_home, token).await?;
  assert_eq!(status, StatusCode::CREATED);
  assert!(body
    .location
    .starts_with("http://id.localhost/realms/test-realm/protocol/openid-connect/auth"));
  Ok(())
}

async fn auth_initiate_handler_with_token_response(
  temp_bodhi_home: TempDir,
  token: String,
) -> anyhow::Result<(StatusCode, RedirectResponse)> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = DefaultSessionService::build_session_service(dbfile).await;
  let record = set_token_in_session(&session_service, &token).await?;
  let mut builder = AppServiceStubBuilder::default();
  builder
    .with_temp_home_as(temp_bodhi_home)
    .setting_service(Arc::new(
      SettingServiceStub::default()
        .append_settings(HashMap::from([
          (BODHI_SCHEME.to_string(), "http".to_string()),
          (BODHI_HOST.to_string(), "frontend.localhost".to_string()),
          (BODHI_PORT.to_string(), "3000".to_string()),
        ]))
        .await,
    ))
    .with_default_session_service(Arc::new(session_service));
  builder.with_db_service().await;
  builder
    .with_app_instance(services::AppInstance::test_default())
    .await;
  let app_service = builder.build().await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    app_service.clone(),
  ));
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware))
    .with_state(state)
    .layer(app_service.session_service().session_layer());
  let resp = router
    .oneshot(
      Request::post("/auth/initiate")
        .header("Cookie", format!("bodhiapp_session_id={}", record.id))
        .header("Sec-Fetch-Site", "same-origin")
        .json(json! {{}})?,
    )
    .await?;
  let status = resp.status();
  let body_bytes = to_bytes(resp.into_body(), usize::MAX).await?;
  let body: RedirectResponse = serde_json::from_slice(&body_bytes)?;
  Ok((status, body))
}

async fn set_token_in_session(
  session_service: &DefaultSessionService,
  token: &str,
) -> Result<Record, anyhow::Error> {
  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
      "access_token".to_string() => Value::String(token.to_string()),
    },
    expiry_date: OffsetDateTime::now_utc() + time::Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;
  Ok(record)
}
