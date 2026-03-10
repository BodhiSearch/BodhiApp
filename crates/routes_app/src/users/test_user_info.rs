use crate::test_utils::RequestAuthContextExt;
use crate::{users_info, TokenInfo, UserInfoEnvelope, UserResponse};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{status::StatusCode, Request},
  routing::get,
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use server_core::test_utils::ResponseTestExt;
use services::AuthContext;
use services::{
  test_utils::{token, AppServiceStubBuilder, TEST_TENANT_ID},
  AppService,
};
use services::{AppRole, ResourceRole, TokenScope, UserInfo, UserScope};
use std::sync::Arc;
use tower::ServiceExt;

fn test_router(app_service: Arc<dyn AppService>) -> Router {
  let session_layer = app_service.session_service().session_layer();
  Router::new()
    .route("/app/user", get(users_info))
    .layer(session_layer)
    .with_state(app_service)
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_anonymous() -> anyhow::Result<()> {
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = test_router(app_service);

  let response = router
    .oneshot(
      Request::get("/app/user")
        .body(Body::empty())?
        .with_auth_context(AuthContext::Anonymous {
          client_id: None,
          tenant_id: None,
          deployment: services::DeploymentMode::Standalone,
        }),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;
  assert_eq!(
    UserInfoEnvelope {
      user: UserResponse::LoggedOut,
      has_dashboard_session: false,
    },
    response_json
  );
  Ok(())
}

#[rstest]
#[case::resource_user(ResourceRole::User)]
#[case::resource_power_user(ResourceRole::PowerUser)]
#[case::resource_manager(ResourceRole::Manager)]
#[case::resource_admin(ResourceRole::Admin)]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_session_token_with_role(
  token: (String, String),
  #[case] role: ResourceRole,
) -> anyhow::Result<()> {
  let (token, _) = token;
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = test_router(app_service);

  // Extract claims before moving token into AuthContext
  let claims = services::extract_claims::<services::Claims>(&token)?;

  let auth_context =
    AuthContext::test_session_with_token(&claims.sub, "testuser@email.com", role, &token);
  let response = router
    .oneshot(
      Request::get("/app/user")
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;

  assert_eq!(
    UserInfoEnvelope {
      user: UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: "testuser@email.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        role: Some(AppRole::Session(role)),
      }),
      has_dashboard_session: false,
    },
    response_json
  );
  Ok(())
}

#[rstest]
#[case::scope_token_user(TokenScope::User)]
#[case::scope_token_power_user(TokenScope::PowerUser)]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_api_token_with_token_scope(
  #[case] token_scope: TokenScope,
) -> anyhow::Result<()> {
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = test_router(app_service);

  let auth_context = AuthContext::test_api_token("test-user-id", token_scope);
  let response = router
    .oneshot(
      Request::get("/app/user")
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;

  // API tokens should return TokenInfo, not UserInfo
  assert_eq!(
    UserInfoEnvelope {
      user: UserResponse::Token(TokenInfo { role: token_scope }),
      has_dashboard_session: false,
    },
    response_json
  );
  Ok(())
}

#[rstest]
#[case::scope_user_user(UserScope::User)]
#[case::scope_user_power_user(UserScope::PowerUser)]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_bearer_token_with_user_scope(
  token: (String, String),
  #[case] user_scope: UserScope,
) -> anyhow::Result<()> {
  let (token, _) = token;
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = test_router(app_service);

  // Extract claims before moving token into AuthContext
  let claims = services::extract_claims::<services::Claims>(&token)?;

  let auth_context = AuthContext::ExternalApp {
    client_id: "test-client-id".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: claims.sub.clone(),
    role: Some(user_scope),
    token: token.clone(),
    external_app_token: token.clone(),
    app_client_id: "test-azp".to_string(),
    access_request_id: None,
  };
  let response = router
    .oneshot(
      Request::get("/app/user")
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;

  assert_eq!(
    UserInfoEnvelope {
      user: UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: "testuser@email.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        role: Some(AppRole::ExchangedToken(user_scope)),
      }),
      has_dashboard_session: false,
    },
    response_json
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_session_without_role(
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = test_router(app_service);

  // Extract claims before moving token into AuthContext
  let claims = services::extract_claims::<services::Claims>(&token)?;

  let auth_context = AuthContext::Session {
    client_id: "test-client-id".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: claims.sub.clone(),
    username: "testuser@email.com".to_string(),
    role: None,
    token: token.clone(),
  };
  let response = router
    .oneshot(
      Request::get("/app/user")
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;

  assert_eq!(
    UserInfoEnvelope {
      user: UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: "testuser@email.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        role: None,
      }),
      has_dashboard_session: false,
    },
    response_json
  );
  Ok(())
}

// Auth tier: Optional - these endpoints work for both authenticated and unauthenticated users

#[anyhow_trace]
#[rstest]
#[case::get_user_info("GET", "/bodhi/v1/user")]
#[case::request_access("POST", "/bodhi/v1/user/request-access")]
#[case::request_status("GET", "/bodhi/v1/user/request-status")]
#[tokio::test]
async fn test_optional_auth_endpoints_accept_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  use tower::ServiceExt;
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  // These endpoints should not return 401/403 for unauthenticated users
  assert_ne!(StatusCode::UNAUTHORIZED, response.status());
  assert_ne!(StatusCode::FORBIDDEN, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_user_info_allows_authenticated(
  #[values(
    "resource_user",
    "resource_power_user",
    "resource_manager",
    "resource_admin"
  )]
  role: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  use tower::ServiceExt;
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let response = router
    .oneshot(session_request("GET", "/bodhi/v1/user", &cookie))
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_external_app_without_scope(
  token: (String, String),
) -> anyhow::Result<()> {
  let (token, _) = token;
  let app_service: Arc<dyn AppService> = Arc::new(AppServiceStubBuilder::default().build().await?);
  let router = test_router(app_service);

  // Extract claims before moving token into AuthContext
  let claims = services::extract_claims::<services::Claims>(&token)?;

  let auth_context = AuthContext::ExternalApp {
    client_id: "test-client-id".to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: claims.sub.clone(),
    role: None,
    token: token.clone(),
    external_app_token: token.clone(),
    app_client_id: "test-azp".to_string(),
    access_request_id: None,
  };

  let response = router
    .oneshot(
      Request::get("/app/user")
        .body(Body::empty())?
        .with_auth_context(auth_context),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;

  assert_eq!(
    UserInfoEnvelope {
      user: UserResponse::LoggedIn(UserInfo {
        user_id: claims.sub,
        username: "testuser@email.com".to_string(),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        role: None,
      }),
      has_dashboard_session: false,
    },
    response_json
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_user_info_handler_with_dashboard_session() -> anyhow::Result<()> {
  use crate::tenants::DASHBOARD_ACCESS_TOKEN_KEY;
  use crate::test_utils::{build_test_router, session_request};
  use services::test_utils::TEST_CLIENT_ID;
  use std::collections::HashMap;
  use time::OffsetDateTime;
  use tower_sessions::{session::Id, session::Record, SessionStore};

  let (router, app_service, _temp) = build_test_router().await?;

  // Build JWT claims with roles
  let mut claims = services::test_utils::access_token_claims();
  claims["resource_access"][TEST_CLIENT_ID]["roles"] = serde_json::json!(["resource_admin"]);
  let (token, _public_key) = services::test_utils::build_token(claims)?;

  // Create a session record with access_token, active_client_id, AND dashboard token
  let session_id = Id::default();
  let mut data = HashMap::new();
  let access_key = format!("{}:access_token", TEST_CLIENT_ID);
  data.insert(access_key, serde_json::Value::String(token));
  data.insert(
    "active_client_id".to_string(),
    serde_json::Value::String(TEST_CLIENT_ID.to_string()),
  );
  data.insert(
    DASHBOARD_ACCESS_TOKEN_KEY.to_string(),
    serde_json::Value::String("dashboard-token-value".to_string()),
  );

  let record = Record {
    id: session_id,
    data,
    expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
  };

  let session_svc = app_service.session_service();
  let store = session_svc.get_session_store();
  store.save(&record).await?;
  let cookie = format!("bodhiapp_session_id={}", session_id);

  let response = router
    .oneshot(session_request("GET", "/bodhi/v1/user", &cookie))
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let response_json = response.json::<UserInfoEnvelope>().await?;

  // Verify has_dashboard_session is true
  assert_eq!(true, response_json.has_dashboard_session);
  // Verify we got a logged_in user
  match &response_json.user {
    UserResponse::LoggedIn(info) => {
      assert_eq!("testuser@email.com", info.username);
    }
    other => panic!("Expected LoggedIn, got {:?}", other),
  }
  Ok(())
}
