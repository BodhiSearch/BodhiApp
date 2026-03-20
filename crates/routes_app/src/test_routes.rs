use super::apply_ui_router;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::get,
  Router,
};
use include_dir::{include_dir, Dir};
use rstest::rstest;
use server_core::test_utils::ResponseTestExt;
use services::EnvType;
use services::{
  test_utils::SettingServiceStub, SettingService, BODHI_DEV_PROXY_UI, BODHI_ENV_TYPE,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
  net::TcpListener,
  sync::oneshot::{channel, Sender},
};
use tower::ServiceExt;

static TEST_UI_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/test_spa_assets");

async fn test_request(router: Router, path: &str) -> (StatusCode, String) {
  let response = router
    .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
    .await
    .unwrap();
  let status = response.status();
  let body = response.text().await.unwrap();
  (status, body)
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
  EnvConfig { is_production: true, proxy_ui: None },
  true,
  vec![
    ("/", StatusCode::TEMPORARY_REDIRECT),
    ("/ui", StatusCode::OK),
    ("/ui/", StatusCode::OK),
    ("/ui/style.css", StatusCode::OK),
    ("/api", StatusCode::OK),
  ]
)]
#[case::production_without_static(
  EnvConfig { is_production: true, proxy_ui: None },
  false,
  vec![
    ("/", StatusCode::TEMPORARY_REDIRECT),
    ("/ui/", StatusCode::NOT_FOUND),
    ("/api", StatusCode::OK),
  ]
)]
#[case::dev_with_static(
  EnvConfig { is_production: false, proxy_ui: Some("false".to_string()) },
  true,
  vec![
    ("/", StatusCode::TEMPORARY_REDIRECT),
    ("/ui", StatusCode::OK),
    ("/ui/", StatusCode::OK),
    ("/ui/style.css", StatusCode::OK),
    ("/api", StatusCode::OK),
  ]
)]
#[case::dev_without_static(
  EnvConfig { is_production: false, proxy_ui: Some("false".to_string()) },
  false,
  vec![
    ("/", StatusCode::TEMPORARY_REDIRECT),
    ("/ui/", StatusCode::NOT_FOUND),
    ("/api", StatusCode::OK),
  ]
)]
#[tokio::test]
async fn test_ui_router_static_scenarios(
  #[case] config: EnvConfig,
  #[case] include_static: bool,
  #[case] test_paths: Vec<(&str, StatusCode)>,
) {
  let setting_service = test_setting_service(config);
  let base_router = Router::new().route("/api", get(|| async { "api" }));
  let static_dir = if include_static {
    Some(&TEST_UI_DIR as &'static Dir<'static>)
  } else {
    None
  };
  let router = apply_ui_router(&setting_service, base_router, static_dir).await;

  for (path, expected_status) in test_paths {
    let (status, _body) = test_request(router.clone(), path).await;
    assert_eq!(
      expected_status, status,
      "Path {} expected {:?}",
      path, expected_status
    );
  }
}

/// Test that dev + proxy_ui=true routes through the proxy backend (not static assets)
#[rstest]
#[tokio::test]
async fn test_ui_router_dev_proxy() -> anyhow::Result<()> {
  let setting_service = test_setting_service(EnvConfig {
    is_production: false,
    proxy_ui: Some("true".to_string()),
  });

  // Start a backend that expects /ui-prefixed paths (simulates Next.js dev server with basePath)
  let listener = TcpListener::bind("127.0.0.1:0").await?;
  let addr: SocketAddr = listener.local_addr()?;
  let backend_app = Router::new()
    .route("/ui", get(|| async { "Root no slash" }))
    .route("/ui/", get(|| async { "Root with slash" }))
    .route("/ui/page", get(|| async { "Proxied page" }));
  let (shutdown_tx, shutdown_rx) = channel::<()>();
  tokio::spawn(async move {
    axum::serve(listener, backend_app)
      .with_graceful_shutdown(async move {
        shutdown_rx.await.unwrap();
      })
      .await
      .unwrap();
  });

  // Same structure as apply_ui_router: root redirect + proxy merge
  let base_router = Router::new().route("/api", get(|| async { "api" }));
  let router = base_router
    .route(
      "/",
      get(|| async { axum::response::Redirect::temporary("/ui/") }),
    )
    .merge(crate::build_ui_proxy_router(format!("http://{addr}")));

  // /ui → proxy → backend /ui
  let (status, body) = test_request(router.clone(), "/ui").await;
  assert_eq!(StatusCode::OK, status);
  assert_eq!("Root no slash", body);

  // /ui/ → proxy → backend /ui/
  let (status, body) = test_request(router.clone(), "/ui/").await;
  assert_eq!(StatusCode::OK, status);
  assert_eq!("Root with slash", body);

  // /ui/page → proxy → backend /ui/page
  let (status, body) = test_request(router.clone(), "/ui/page").await;
  assert_eq!(StatusCode::OK, status);
  assert_eq!("Proxied page", body);

  // /api still works
  let (status, body) = test_request(router.clone(), "/api").await;
  assert_eq!(StatusCode::OK, status);
  assert_eq!("api", body);

  // Root redirects
  let (status, _) = test_request(router, "/").await;
  assert_eq!(StatusCode::TEMPORARY_REDIRECT, status);

  shutdown_tx.send(()).unwrap();
  Ok(())
}
