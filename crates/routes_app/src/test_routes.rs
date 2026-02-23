use super::apply_ui_router;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::get,
  Router,
};
use mockall::predicate::*;
use objs::EnvType;
use rstest::{fixture, rstest};
use server_core::test_utils::ResponseTestExt;
use services::{
  test_utils::SettingServiceStub, SettingService, BODHI_DEV_PROXY_UI, BODHI_ENV_TYPE,
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

// Helper to create a stub router that returns a specific path
fn create_stub_router(path: &'static str) -> Router {
  let result = path.to_string();
  Router::new().route(path, get(|| async { result }))
}

// Helper to make a test request to the router
async fn test_request(router: Router, path: &str) -> (StatusCode, String) {
  let response = router
    .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
    .await
    .unwrap();

  let status = response.status();
  let body = response.text().await.unwrap();
  (status, body)
}

#[fixture]
fn base_router() -> Router {
  create_stub_router("/api")
}

#[fixture]
fn static_router() -> Router {
  create_stub_router("/static")
}

#[fixture]
fn proxy_router() -> Router {
  create_stub_router("/proxy")
}

struct EnvConfig {
  is_production: bool,
  proxy_ui: Option<String>,
}

fn test_setting_service(config: EnvConfig) -> Arc<dyn SettingService> {
  let env_type = if config.is_production {
    EnvType::Production
  } else {
    EnvType::Development
  };
  let mut envs = HashMap::new();
  if let Some(proxy_ui) = config.proxy_ui {
    envs.insert(BODHI_DEV_PROXY_UI.to_string(), proxy_ui);
  }
  let setting_service = SettingServiceStub::with_envs_settings(
    envs,
    HashMap::from([(BODHI_ENV_TYPE.to_string(), env_type.to_string())]),
  );
  Arc::new(setting_service)
}

#[rstest]
#[case::production_with_static(
  EnvConfig {
    is_production: true,
    proxy_ui: None
  },
  Some(static_router()),
  vec![
    ("/api", true),
    ("/static", true),
    ("/proxy", false),
  ]
)]
#[case::production_without_static(
  EnvConfig {
    is_production: true,
    proxy_ui: None
  },
  None,
  vec![
    ("/api", true),
    ("/static", false),
    ("/proxy", false),
  ]
)]
#[case::dev_with_proxy(
  EnvConfig {
    is_production: false,
    proxy_ui: Some("true".to_string())
  },
  Some(static_router()),
  vec![
    ("/api", true),
    ("/static", false),
    ("/proxy", true),
  ]
)]
#[case::dev_with_static(
  EnvConfig {
    is_production: false,
    proxy_ui: Some("false".to_string())
  },
  Some(static_router()),
  vec![
    ("/api", true),
    ("/static", true),
    ("/proxy", false),
  ]
)]
#[case::dev_without_static(
  EnvConfig {
    is_production: false,
    proxy_ui: Some("false".to_string())
  },
  None,
  vec![
    ("/api", true),
    ("/static", false),
    ("/proxy", false),
  ]
)]
#[tokio::test]
async fn test_ui_router_scenarios(
  #[case] config: EnvConfig,
  #[case] static_router: Option<Router>,
  #[case] test_paths: Vec<(&str, bool)>,
) {
  let setting_service = test_setting_service(config);
  let router = apply_ui_router(
    &setting_service,
    base_router(),
    static_router,
    proxy_router(),
  )
  .await;

  for (path, should_exist) in test_paths {
    let (status, body) = test_request(router.clone(), path).await;

    if should_exist {
      assert_eq!(status, StatusCode::OK, "Path {} should exist", path);
      assert_eq!(body, path);
    } else {
      assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "Path {} should not exist",
        path
      );
    }
  }
}
