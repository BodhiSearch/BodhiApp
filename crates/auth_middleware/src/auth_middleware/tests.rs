use crate::{auth_middleware, optional_auth_middleware, AuthContext};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  extract::Extension,
  http::{HeaderMap, Request, StatusCode},
  middleware::from_fn_with_state,
  response::{IntoResponse, Response},
  routing::get,
  Json, Router,
};
use base64::{engine::general_purpose, Engine};
use mockall::predicate::eq;
use objs::{test_utils::temp_bodhi_home, ReqwestError, TokenScope};
use pretty_assertions::assert_eq;
use rand::RngCore;
use rstest::rstest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext, RouterState,
};
use services::{
  db::{ApiToken, TokenStatus},
  test_utils::{
    access_token_claims, build_token, expired_token, AppServiceStubBuilder, TEST_CLIENT_ID,
    TEST_CLIENT_SECRET,
  },
  AppInstance, AppService, AppStatus, AuthServiceError, DefaultSessionService, MockAuthService,
  SessionService, BODHI_HOST, BODHI_PORT, BODHI_SCHEME,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestResponse {
  token: Option<String>,
  role: Option<String>,
  scope: Option<String>,
  user_id: Option<String>,
  is_authenticated: bool,
  authorization_header: Option<String>,
  path: String,
}

async fn test_handler_teapot(
  auth_context: Option<Extension<AuthContext>>,
  headers: HeaderMap,
  path: &str,
) -> Response {
  let authorization_header = headers
    .get("Authorization")
    .map(|v| v.to_str().unwrap().to_string());
  let (token, role, scope, user_id, is_authenticated) = if let Some(Extension(ctx)) = &auth_context
  {
    let token = ctx.token().map(|s| s.to_string());
    let role = match ctx {
      AuthContext::Session {
        role: Some(role), ..
      } => Some(role.to_string()),
      _ => None,
    };
    let scope = match ctx {
      AuthContext::ApiToken { role, .. } => Some(role.to_string()),
      _ => None,
    };
    let user_id = ctx.user_id().map(|s| s.to_string());
    let is_authenticated = ctx.is_authenticated();
    (token, role, scope, user_id, is_authenticated)
  } else {
    (None, None, None, None, false)
  };
  (
    StatusCode::IM_A_TEAPOT,
    Json(TestResponse {
      token,
      role,
      scope,
      user_id,
      is_authenticated,
      authorization_header,
      path: path.to_string(),
    }),
  )
    .into_response()
}

fn router_with_auth() -> Router<Arc<dyn RouterState>> {
  Router::new().route(
    "/with_auth",
    get(
      |auth_context: Option<Extension<AuthContext>>, headers: HeaderMap| async move {
        test_handler_teapot(auth_context, headers, "/with_auth").await
      },
    ),
  )
}

fn router_with_optional_auth() -> Router<Arc<dyn RouterState>> {
  Router::new().route(
    "/with_optional_auth",
    get(
      |auth_context: Option<Extension<AuthContext>>, headers: HeaderMap| async move {
        test_handler_teapot(auth_context, headers, "/with_optional_auth").await
      },
    ),
  )
}

fn test_router(state: Arc<dyn RouterState>) -> Router {
  Router::new()
    .merge(router_with_auth().route_layer(from_fn_with_state(state.clone(), auth_middleware)))
    .merge(
      router_with_optional_auth()
        .route_layer(from_fn_with_state(state.clone(), optional_auth_middleware)),
    )
    .layer(state.app_service().session_service().session_layer())
    .with_state(state)
}

async fn assert_optional_auth_passthrough(router: &Router) -> anyhow::Result<()> {
  let response = router
    .clone()
    .oneshot(Request::get("/with_optional_auth").body(Body::empty())?)
    .await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
  let response_json = response.json::<TestResponse>().await?;
  assert_eq!(false, response_json.is_authenticated);
  assert_eq!(None, response_json.token);
  Ok(())
}

#[rstest]
#[case::authz_setup(AppStatus::Setup)]
#[case::authz_missing(AppStatus::Setup)]
#[tokio::test]
#[anyhow_trace]
async fn test_auth_middleware_returns_app_status_invalid_for_app_status_setup_or_missing(
  #[case] status: AppStatus,
) -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_with_status(status))
    .await
    .with_session_service()
    .await
    .with_db_service()
    .await
    .with_settings(maplit::hashmap! {
      BODHI_SCHEME => "https",
      BODHI_HOST => "bodhi.app",
      BODHI_PORT => "443",
    })
    .await
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    app_service.clone(),
  ));
  let req = Request::get("/with_auth").body(Body::empty())?;
  let router = test_router(state);
  let response = router.clone().oneshot(req).await?;
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
  let body: Value = response.json().await?;
  assert_eq!(
    json! {{
      "error": {
        "code": "auth_error-app_status_invalid",
        "type": "invalid_app_state",
        "message": "Application is not ready. Current status: setup.",
        "param": {
          "var_0": "setup"
        }
      }
    }},
    body
  );
  assert_optional_auth_passthrough(&router).await?;
  Ok(())
}

#[rstest]
#[case::with_auth_role_admin("/with_auth", &["resource_admin"], "resource_admin")]
#[case::with_auth_role_user("/with_auth", &["resource_user"], "resource_user")]
#[case::with_auth_role_manager("/with_auth", &["resource_manager"], "resource_manager")]
#[case::with_auth_role_power_user("/with_auth", &["resource_power_user"], "resource_power_user")]
#[case::with_auth_role_user_admin("/with_auth", &["resource_user", "resource_admin"], "resource_admin")]
#[case::with_auth_role_user_manager("/with_auth", &["resource_user", "resource_manager"], "resource_manager")]
#[case::with_optional_auth_role_admin("/with_optional_auth", &["resource_admin"], "resource_admin")]
#[case::with_optional_auth_role_user("/with_optional_auth", &["resource_user"], "resource_user")]
#[case::with_optional_auth_role_manager("/with_optional_auth", &["resource_manager"], "resource_manager")]
#[case::with_optional_auth_role_power_user("/with_optional_auth", &["resource_power_user"], "resource_power_user")]
#[case::with_optional_auth_role_user_admin("/with_optional_auth", &["resource_user", "resource_admin"], "resource_admin")]
#[case::with_optional_auth_role_user_manager("/with_optional_auth", &["resource_user", "resource_manager"], "resource_manager")]
#[tokio::test]
async fn test_auth_middleware_with_valid_session_token(
  temp_bodhi_home: TempDir,
  #[case] path: &str,
  #[case] roles: &[&str],
  #[case] expected_role: &str,
) -> anyhow::Result<()> {
  let mut claims = access_token_claims();
  claims["resource_access"][TEST_CLIENT_ID]["roles"] =
    Value::Array(roles.iter().map(|r| Value::String(r.to_string())).collect());
  let (token, _) = build_token(claims)?;
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
      "access_token".to_string() => Value::String(token.clone()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .session_service(session_service.clone())
    .with_db_service()
    .await
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    app_service.clone(),
  ));

  let req = Request::get(path)
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;
  let router = test_router(state);

  let response = router.oneshot(req).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
  let body = response.json::<TestResponse>().await?;
  assert_eq!(path, body.path);
  assert_eq!(Some(token), body.token);
  assert_eq!(Some(expected_role.to_string()), body.role);
  assert_eq!(None, body.scope);
  assert_eq!(true, body.is_authenticated);
  Ok(())
}

#[rstest]
#[case::with_auth_role_user("/with_auth", &["resource_user"], "resource_user")]
#[case::with_auth_role_manager("/with_auth", &["resource_manager"], "resource_manager")]
#[case::with_auth_role_power_user("/with_auth", &["resource_power_user"], "resource_power_user")]
#[case::with_auth_role_admin("/with_auth", &["resource_admin"], "resource_admin")]
#[case::with_auth_role_user_admin("/with_auth", &["resource_user", "resource_admin"], "resource_admin")]
#[case::with_auth_role_user_manager("/with_auth", &["resource_user", "resource_manager"], "resource_manager")]
#[case::with_optional_auth_role_user("/with_optional_auth", &["resource_user"], "resource_user")]
#[case::with_optional_auth_role_manager("/with_optional_auth", &["resource_manager"], "resource_manager")]
#[case::with_optional_auth_role_power_user("/with_optional_auth", &["resource_power_user"], "resource_power_user")]
#[case::with_optional_auth_role_admin("/with_optional_auth", &["resource_admin"], "resource_admin")]
#[case::with_optional_auth_role_user_admin("/with_optional_auth", &["resource_user", "resource_admin"], "resource_admin")]
#[case::with_optional_auth_role_user_manager("/with_optional_auth", &["resource_user", "resource_manager"], "resource_manager")]
#[tokio::test]
async fn test_auth_middleware_with_expired_session_token(
  expired_token: (String, String),
  temp_bodhi_home: TempDir,
  #[case] path: &str,
  #[case] roles: &[&str],
  #[case] expected_role: &str,
) -> anyhow::Result<()> {
  let (expired_token, _) = expired_token;
  let mut access_token_claims = access_token_claims();
  access_token_claims["resource_access"][TEST_CLIENT_ID]["roles"] =
    Value::Array(roles.iter().map(|r| Value::String(r.to_string())).collect());
  let (exchanged_token, _) = build_token(access_token_claims)?;
  let exchanged_token_cl = exchanged_token.clone();
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
        "access_token".to_string() => Value::String(expired_token.clone()),
        "refresh_token".to_string() => Value::String("valid_refresh_token".to_string()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;
  // Create mock auth service
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_refresh_token()
    .with(
      eq(TEST_CLIENT_ID),
      eq(TEST_CLIENT_SECRET),
      eq("valid_refresh_token".to_string()),
    )
    .times(1)
    .return_once(|_, _, _| Ok((exchanged_token_cl, Some("new_refresh_token".to_string()))));

  // Setup app service with mocks
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .time_service(Arc::new(services::db::DefaultTimeService))
    .auth_service(Arc::new(mock_auth_service))
    .session_service(session_service.clone())
    .with_db_service()
    .await
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    app_service.clone(),
  ));

  let req = Request::get(path)
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;
  let router = test_router(state);

  let response = router.clone().oneshot(req).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
  let body = response.json::<TestResponse>().await?;
  assert_eq!(path, body.path);
  assert_eq!(Some(exchanged_token.clone()), body.token);
  assert_eq!(Some(expected_role.to_string()), body.role);
  assert_eq!(true, body.is_authenticated);

  // Verify that the session was updated with the new tokens
  let updated_record = session_service
    .get_session_store()
    .load(&id)
    .await?
    .unwrap();
  assert_eq!(
    exchanged_token,
    updated_record
      .data
      .get("access_token")
      .unwrap()
      .as_str()
      .unwrap()
  );
  assert_eq!(
    "new_refresh_token",
    updated_record
      .data
      .get("refresh_token")
      .unwrap()
      .as_str()
      .unwrap()
  );

  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_auth_middleware_token_refresh_persists_to_session(
  expired_token: (String, String),
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let (expired_token, _) = expired_token;
  let mut access_token_claims = access_token_claims();
  access_token_claims["resource_access"][TEST_CLIENT_ID]["roles"] =
    Value::Array(vec![Value::String("resource_admin".to_string())]);
  let (new_access_token, _) = build_token(access_token_claims)?;
  let new_access_token_cl = new_access_token.clone();

  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
        "access_token".to_string() => Value::String(expired_token.clone()),
        "refresh_token".to_string() => Value::String("valid_refresh_token".to_string()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  // Create mock auth service that succeeds refresh
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_refresh_token()
    .with(
      eq(TEST_CLIENT_ID),
      eq(TEST_CLIENT_SECRET),
      eq("valid_refresh_token".to_string()),
    )
    .times(1)
    .return_once(|_, _, _| Ok((new_access_token_cl, Some("new_refresh_token".to_string()))));

  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .time_service(Arc::new(services::db::DefaultTimeService))
    .auth_service(Arc::new(mock_auth_service))
    .session_service(session_service.clone())
    .with_db_service()
    .await
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    app_service.clone(),
  ));

  // First request should trigger refresh
  let req1 = Request::get("/with_auth")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;
  let router = test_router(state.clone());

  let response1 = router.clone().oneshot(req1).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response1.status());

  // Verify session was updated with new tokens
  let updated_record = session_service
    .get_session_store()
    .load(&id)
    .await?
    .unwrap();
  assert_eq!(
    new_access_token,
    updated_record
      .data
      .get("access_token")
      .unwrap()
      .as_str()
      .unwrap()
  );
  assert_eq!(
    "new_refresh_token",
    updated_record
      .data
      .get("refresh_token")
      .unwrap()
      .as_str()
      .unwrap()
  );

  // Second request should use the refreshed token without another refresh
  let req2 = Request::get("/with_auth")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;

  let response2 = router.oneshot(req2).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response2.status());

  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_auth_middleware_with_expired_session_token_and_failed_refresh(
  expired_token: (String, String),
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let (expired_token, _) = expired_token;
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = Arc::new(DefaultSessionService::build_session_service(dbfile).await);

  // Create mock auth service that fails to refresh the token
  let mut mock_auth_service = MockAuthService::default();
  mock_auth_service
    .expect_refresh_token()
    .with(
      eq("test_client_id"),
      eq("test_client_secret"),
      eq("refresh_token"),
    )
    .times(1)
    .return_once(|_, _, _| {
      Err(AuthServiceError::Reqwest(ReqwestError::new(
        "Failed to refresh token".to_string(),
      )))
    });

  let instance = AppInstance {
    client_id: "test_client_id".to_string(),
    client_secret: "test_client_secret".to_string(),
    status: AppStatus::Ready,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(instance)
    .await
    .time_service(Arc::new(services::db::DefaultTimeService))
    .auth_service(Arc::new(mock_auth_service))
    .session_service(session_service.clone())
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    app_service.clone(),
  ));

  let id = Id::default();
  let mut record = Record {
    id,
    data: maplit::hashmap! {
        "access_token".to_string() => Value::String(expired_token.clone()),
        "refresh_token".to_string() => Value::String("refresh_token".to_string()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  let req = Request::get("/with_auth")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "same-origin")
    .body(Body::empty())?;
  let router = test_router(state);

  let response = router.clone().oneshot(req).await?;
  assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
  let actual: Value = response.json().await?;
  assert_eq!(
    json! {{
      "error": {
        "message": "Network error: Failed to refresh token.",
        "type": "internal_server_error",
        "code": "reqwest_error",
        "param": {
          "error": "Failed to refresh token"
        }
      }
    }},
    actual
  );
  // Verify that the session was not updated
  let updated_record = session_service
    .get_session_store()
    .load(&id)
    .await?
    .unwrap();
  assert_eq!(
    expired_token,
    updated_record
      .data
      .get("access_token")
      .unwrap()
      .as_str()
      .unwrap()
  );
  assert_eq!(
    "refresh_token",
    updated_record
      .data
      .get("refresh_token")
      .unwrap()
      .as_str()
      .unwrap()
  );

  assert_optional_auth_passthrough(&router).await?;
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_auth_middleware_returns_invalid_access_when_no_token_in_session(
  temp_bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .session_service(session_service.clone())
    .with_db_service()
    .await
    .build()
    .await?;
  let app_service = Arc::new(app_service);
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    app_service.clone(),
  ));

  let req = Request::get("/with_auth").body(Body::empty())?;
  let router = test_router(state);
  let response = router.oneshot(req).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  let body: Value = response.json().await?;
  assert_eq!(
    json! {{
      "error": {
        "code": "auth_error-invalid_access",
        "type": "authentication_error",
        "message": "Access denied."
      }
    }},
    body
  );
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_auth_middleware_removes_internal_token_headers() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .with_session_service()
    .await
    .with_db_service()
    .await
    .build()
    .await?;
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));
  let router = test_router(state);
  let req = Request::get("/with_optional_auth")
    .header("X-BodhiApp-Token", "user-sent-token")
    .header("X-BodhiApp-Role", "user-sent-role")
    .header("X-BodhiApp-Scope", "user-sent-scope")
    .header("X-BodhiApp-Username", "user-sent-username")
    .header("X-BodhiApp-User-Id", "user-sent-userid")
    .json(json! {{}})?;
  let response = router.clone().oneshot(req).await?;
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
  let actual: TestResponse = response.json().await?;
  assert_eq!("/with_optional_auth", actual.path);
  assert_eq!(false, actual.is_authenticated);
  assert_eq!(None, actual.token);
  assert_eq!(None, actual.role);
  assert_eq!(None, actual.scope);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_session_ignored_when_cross_site(temp_bodhi_home: TempDir) -> anyhow::Result<()> {
  let dbfile = temp_bodhi_home.path().join("test.db");
  let session_service = Arc::new(DefaultSessionService::build_session_service(dbfile).await);
  // create dummy session record
  let id = tower_sessions::session::Id::default();
  let mut record = tower_sessions::session::Record {
    id,
    data: maplit::hashmap! {
      "access_token".to_string() => serde_json::Value::String("dummy".to_string()),
    },
    expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
  };
  session_service
    .get_session_store()
    .create(&mut record)
    .await?;

  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .session_service(session_service.clone())
    .with_db_service()
    .await
    .build()
    .await?;
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));
  let router = test_router(state);
  // cross-site header
  let req = Request::get("/with_auth")
    .header("Cookie", format!("bodhiapp_session_id={}", id))
    .header("Sec-Fetch-Site", "cross-site")
    .body(Body::empty())?;
  let response = router.oneshot(req).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

#[rstest]
#[case::user("scope_token_user")]
#[case::power_user("scope_token_power_user")]
#[tokio::test]
async fn test_auth_middleware_bodhiapp_token_scope_variations(
  #[case] scope_str: &str,
) -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .with_session_service()
    .await
    .with_db_service()
    .await
    .build()
    .await?;
  let db_service = app_service.db_service.as_ref().unwrap();

  // Generate test token
  let mut random_bytes = [0u8; 32];
  rand::rng().fill_bytes(&mut random_bytes);
  let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
  let token_str = format!("bodhiapp_{}", random_string);
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Hash the token
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // Create ApiToken in database with specified scope
  let now = app_service.time_service().utc_now();
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user-id".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: scope_str.to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  db_service.create_api_token(&mut api_token).await?;

  // Create test router
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));
  let router = test_router(state);

  // Make request with bearer token
  let req = Request::get("/with_auth")
    .header("Authorization", format!("Bearer {}", token_str))
    .body(Body::empty())?;

  let response = router.oneshot(req).await?;

  // Assert response is successful
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());

  // Assert AuthContext is set correctly
  let actual: TestResponse = response.json().await?;
  assert_eq!("/with_auth", actual.path);
  assert_eq!(Some(token_str.clone()), actual.token);
  assert_eq!(None, actual.role);
  let expected_scope: TokenScope = scope_str.parse().unwrap();
  assert_eq!(Some(expected_scope.to_string()), actual.scope);
  assert_eq!(true, actual.is_authenticated);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_auth_middleware_bodhiapp_token_success() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .with_session_service()
    .await
    .with_db_service()
    .await
    .build()
    .await?;
  let db_service = app_service.db_service.as_ref().unwrap();

  // Generate test token
  let mut random_bytes = [0u8; 32];
  rand::rng().fill_bytes(&mut random_bytes);
  let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
  let token_str = format!("bodhiapp_{}", random_string);
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Hash the token
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // Create ApiToken in database
  let now = app_service.time_service().utc_now();
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user-id".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  db_service.create_api_token(&mut api_token).await?;

  // Create test router
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));
  let router = test_router(state);

  // Make request with bearer token
  let req = Request::get("/with_auth")
    .header("Authorization", format!("Bearer {}", token_str))
    .body(Body::empty())?;

  let response = router.oneshot(req).await?;

  // Assert response is successful
  assert_eq!(StatusCode::IM_A_TEAPOT, response.status());

  // Assert AuthContext is set correctly
  let actual: TestResponse = response.json().await?;
  assert_eq!("/with_auth", actual.path);
  assert_eq!(Some(token_str), actual.token);
  assert_eq!(None, actual.role);
  assert_eq!(Some("scope_token_user".to_string()), actual.scope);
  assert_eq!(true, actual.is_authenticated);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_auth_middleware_bodhiapp_token_inactive() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .with_session_service()
    .await
    .with_db_service()
    .await
    .build()
    .await?;
  let db_service = app_service.db_service.as_ref().unwrap();

  // Generate test token
  let mut random_bytes = [0u8; 32];
  rand::rng().fill_bytes(&mut random_bytes);
  let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
  let token_str = format!("bodhiapp_{}", random_string);
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Hash the token
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // Create ApiToken in database with Inactive status
  let now = app_service.time_service().utc_now();
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user-id".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Inactive,
    created_at: now,
    updated_at: now,
  };
  db_service.create_api_token(&mut api_token).await?;

  // Create test router
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));
  let router = test_router(state);

  // Make request with bearer token
  let req = Request::get("/with_auth")
    .header("Authorization", format!("Bearer {}", token_str))
    .body(Body::empty())?;

  let response = router.oneshot(req).await?;

  // Assert request returns 401 Unauthorized
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  let body: Value = response.json().await?;
  assert_eq!(
    json! {{
      "error": {
        "code": "auth_error-token_inactive",
        "type": "authentication_error",
        "message": "API token is inactive."
      }
    }},
    body
  );
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_auth_middleware_bodhiapp_token_invalid_hash() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .with_session_service()
    .await
    .with_db_service()
    .await
    .build()
    .await?;
  let db_service = app_service.db_service.as_ref().unwrap();

  // Generate test token
  let mut random_bytes = [0u8; 32];
  rand::rng().fill_bytes(&mut random_bytes);
  let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
  let token_str = format!("bodhiapp_{}", random_string);
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Hash a DIFFERENT token for storage
  let different_token = format!(
    "bodhiapp_{}",
    general_purpose::URL_SAFE_NO_PAD.encode([1u8; 32])
  );
  let mut hasher = Sha256::new();
  hasher.update(different_token.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // Create ApiToken in database with different hash
  let now = app_service.time_service().utc_now();
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user-id".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: now,
    updated_at: now,
  };
  db_service.create_api_token(&mut api_token).await?;

  // Create test router
  let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::new()),
    Arc::new(app_service),
  ));
  let router = test_router(state);

  // Make request with bearer token (different from stored hash)
  let req = Request::get("/with_auth")
    .header("Authorization", format!("Bearer {}", token_str))
    .body(Body::empty())?;

  let response = router.oneshot(req).await?;

  // Assert request returns 401 Unauthorized
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  let body: Value = response.json().await?;
  assert_eq!(
    json! {{
      "error": {
        "code": "token_error-invalid_token",
        "type": "authentication_error",
        "message": "Invalid token: Invalid token.",
        "param": {
          "var_0": "Invalid token"
        }
      }
    }},
    body
  );
  Ok(())
}
