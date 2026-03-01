use crate::{
  delete_setting_handler, list_settings_handler, update_setting_handler, ENDPOINT_SETTINGS,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::{delete, get, put},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::test_utils::temp_dir;
use services::{
  test_utils::{bodhi_home_setting, AppServiceStubBuilder, EnvWrapperStub, MockSettingsRepository},
  BootstrapParts, DbSetting, DefaultSettingService, SettingService, BODHI_EXEC_VARIANT, BODHI_HOST,
  BODHI_LOG_LEVEL, BODHI_PORT,
};
use services::{SettingInfo, SettingMetadata, SettingSource};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;

async fn app(app_service: Arc<dyn services::AppService>) -> Router {
  let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), app_service);
  Router::new()
    .route(ENDPOINT_SETTINGS, get(list_settings_handler))
    .route("/v1/bodhi/settings/{key}", put(update_setting_handler))
    .route("/v1/bodhi/settings/{key}", delete(delete_setting_handler))
    .with_state(Arc::new(router_state))
}

fn noop_settings_repo() -> Arc<MockSettingsRepository> {
  let mut mock = MockSettingsRepository::new();
  let store: Arc<std::sync::RwLock<HashMap<String, DbSetting>>> =
    Arc::new(std::sync::RwLock::new(HashMap::new()));
  let store_get = store.clone();
  let store_upsert = store.clone();
  let store_delete = store.clone();
  let store_list = store;
  mock
    .expect_get_setting()
    .returning(move |key| Ok(store_get.read().unwrap().get(key).cloned()));
  mock.expect_upsert_setting().returning(move |setting| {
    store_upsert
      .write()
      .unwrap()
      .insert(setting.key.clone(), setting.clone());
    Ok(setting.clone())
  });
  mock.expect_delete_setting().returning(move |key| {
    store_delete.write().unwrap().remove(key);
    Ok(())
  });
  mock
    .expect_list_settings()
    .returning(move || Ok(store_list.read().unwrap().values().cloned().collect()));
  Arc::new(mock)
}

async fn test_setting_service(
  temp_dir: &TempDir,
  envs: HashMap<String, String>,
  settings: HashMap<String, serde_yaml::Value>,
) -> Result<DefaultSettingService, anyhow::Error> {
  let settings_yaml = temp_dir.path().join("settings.yaml");
  let bodhi_home = temp_dir.path().to_path_buf();
  let bodhi_home_s = bodhi_home_setting(temp_dir.path(), SettingSource::Environment);
  let setting_service = DefaultSettingService::from_parts(
    BootstrapParts {
      env_wrapper: Arc::new(EnvWrapperStub::new(envs)),
      settings_file: settings_yaml,
      system_settings: vec![bodhi_home_s],
      file_defaults: HashMap::new(),
      app_settings: HashMap::new(),
      app_command: services::AppCommand::Default,
      bodhi_home,
    },
    noop_settings_repo(),
  );
  for (key, value) in settings {
    setting_service.set_setting_value(&key, &value).await?;
  }
  Ok(setting_service)
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_routes_settings_list(temp_dir: TempDir) -> anyhow::Result<()> {
  // GIVEN app with auth disabled
  let setting_service = test_setting_service(
    &temp_dir,
    maplit::hashmap! {
      BODHI_LOG_LEVEL.to_string() => "info".to_string(),
      BODHI_HOST.to_string() => "test.host".to_string(),
    },
    maplit::hashmap! {
      BODHI_PORT.to_string() => serde_yaml::Value::Number(serde_yaml::Number::from_str("8080")?),
    },
  )
  .await?;

  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN requesting settings without auth
  let response = app
    .oneshot(
      Request::builder()
        .uri(ENDPOINT_SETTINGS)
        .body(Body::empty())?,
    )
    .await?;

  // THEN returns settings successfully
  assert_eq!(StatusCode::OK, response.status());
  let settings = response.json::<Vec<SettingInfo>>().await?;
  let log_level = SettingInfo {
    key: BODHI_LOG_LEVEL.to_string(),
    current_value: serde_yaml::Value::String("info".to_string()),
    default_value: serde_yaml::Value::String("warn".to_string()),
    source: SettingSource::Environment,
    metadata: SettingMetadata::Option {
      options: vec![
        "error".to_string(),
        "warn".to_string(),
        "info".to_string(),
        "debug".to_string(),
        "trace".to_string(),
      ],
    },
  };
  let host = SettingInfo {
    key: BODHI_HOST.to_string(),
    current_value: serde_yaml::Value::String("test.host".to_string()),
    default_value: serde_yaml::Value::String("0.0.0.0".to_string()),
    source: SettingSource::Environment,
    metadata: SettingMetadata::String,
  };
  let port = SettingInfo {
    key: "BODHI_PORT".to_string(),
    current_value: serde_yaml::Value::Number(serde_yaml::Number::from(8080)),
    default_value: serde_yaml::Value::Number(serde_yaml::Number::from(1135)),
    source: SettingSource::Database,
    metadata: SettingMetadata::Number { min: 1, max: 65535 },
  };
  assert_eq!(
    Some(&log_level),
    settings.iter().find(|k| k.key == BODHI_LOG_LEVEL)
  );
  assert_eq!(Some(&host), settings.iter().find(|k| k.key == BODHI_HOST));
  assert_eq!(Some(&port), settings.iter().find(|k| k.key == BODHI_PORT));
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_routes_setting_update_success(temp_dir: TempDir) -> anyhow::Result<()> {
  let setting_service =
    test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {}).await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN updating the setting
  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/v1/bodhi/settings/BODHI_EXEC_VARIANT")
        .json(json! {{"value": llama_server_proc::DEFAULT_VARIANT.to_string()}})?,
    )
    .await?;

  // THEN it succeeds
  assert_eq!(StatusCode::OK, response.status());

  // AND returns updated setting
  let setting = response.json::<SettingInfo>().await?;
  assert_eq!(BODHI_EXEC_VARIANT, setting.key);
  assert_eq!(
    serde_yaml::Value::String(llama_server_proc::DEFAULT_VARIANT.to_string()),
    setting.current_value
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_routes_setting_update_invalid_key(temp_dir: TempDir) -> anyhow::Result<()> {
  let setting_service =
    test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {}).await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN updating an invalid setting
  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/v1/bodhi/settings/INVALID_KEY")
        .json(json! {{ "value": "any" }})?,
    )
    .await?;

  // THEN it fails with not found
  assert_eq!(StatusCode::NOT_FOUND, response.status());

  let error = response.json::<serde_json::Value>().await?;
  assert_eq!(
    Some("settings_error-not_found"),
    error["error"]["code"].as_str()
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_routes_setting_update_unsupported_key(temp_dir: TempDir) -> anyhow::Result<()> {
  let setting_service =
    test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {}).await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN updating a setting that exists but is not editable
  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/v1/bodhi/settings/BODHI_PORT")
        .json(json! {{ "value": 8080 }})?,
    )
    .await?;

  // THEN it fails with unsupported
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let error = response.json::<serde_json::Value>().await?;
  assert_eq!(
    Some("settings_error-unsupported"),
    error["error"]["code"].as_str()
  );

  Ok(())
}

#[rstest]
#[ignore = "enable when supporting editing other settings"]
#[tokio::test]
#[anyhow_trace]
async fn test_routes_setting_update_invalid_value(temp_dir: TempDir) -> anyhow::Result<()> {
  let setting_service =
    test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {}).await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN updating with invalid value type
  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/v1/bodhi/settings/BODHI_PORT")
        .json(json! {{ "value": "not a number" }})?,
    )
    .await?;

  // THEN it fails validation
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let error = response.json::<serde_json::Value>().await?;
  assert_eq!(
    Some("settings_metadata_error-invalid_value_type"),
    error["error"]["code"].as_str()
  );
  Ok(())
}

#[rstest]
#[ignore = "enable when supporting editing other settings"]
#[tokio::test]
#[anyhow_trace]
async fn test_routes_setting_update_invalid_value_out_of_range(
  temp_dir: TempDir,
) -> anyhow::Result<()> {
  let setting_service =
    test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {}).await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN updating with invalid value type
  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/v1/bodhi/settings/BODHI_PORT")
        .json(json! {{ "value": 1000 }})?,
    )
    .await?;

  // THEN it fails validation
  // assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let error = response.json::<serde_json::Value>().await?;
  assert_eq!(
    Some("settings_metadata_error-invalid_value"),
    error["error"]["code"].as_str()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_setting_success(temp_dir: TempDir) -> anyhow::Result<()> {
  // GIVEN an app with a custom setting value
  let setting_service = test_setting_service(
    &temp_dir,
    maplit::hashmap! {},
    maplit::hashmap! {
      BODHI_EXEC_VARIANT.to_string() => serde_yaml::Value::String("metal".to_string()),
    },
  )
  .await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN deleting the setting
  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri(format!("/v1/bodhi/settings/{BODHI_EXEC_VARIANT}"))
        .body(Body::empty())?,
    )
    .await?;

  // THEN it succeeds
  assert_eq!(StatusCode::OK, response.status());

  // AND returns setting with default value
  let setting = response.json::<SettingInfo>().await?;
  assert_eq!(BODHI_EXEC_VARIANT, setting.key);
  assert_eq!(
    serde_yaml::Value::String(llama_server_proc::DEFAULT_VARIANT.to_string()),
    setting.current_value
  );
  assert_eq!(SettingSource::Default, setting.source);

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_setting_invalid_key(temp_dir: TempDir) -> anyhow::Result<()> {
  // GIVEN an app
  let setting_service =
    test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {}).await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN deleting an invalid setting
  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/v1/bodhi/settings/INVALID_KEY")
        .body(Body::empty())?,
    )
    .await?;

  // THEN it fails with not found
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let error = response.json::<serde_json::Value>().await?;
  assert_eq!(
    Some("settings_error-not_found"),
    error["error"]["code"].as_str()
  );

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_setting_with_env_override(temp_dir: TempDir) -> anyhow::Result<()> {
  // GIVEN an app with both env and file settings
  let setting_service = test_setting_service(
    &temp_dir,
    maplit::hashmap! {
      BODHI_EXEC_VARIANT.to_string() => "aarch64-unknown-linux-gnu/cpu/llama-server".to_string(),
    },
    maplit::hashmap! {
      BODHI_EXEC_VARIANT.to_string() => serde_yaml::Value::String("aarch64-unknown-linux-gnu/cuda/llama-server".to_string()),
    },
  ).await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN deleting the setting
  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/v1/bodhi/settings/BODHI_EXEC_VARIANT")
        .body(Body::empty())?,
    )
    .await?;

  // THEN it succeeds
  assert_eq!(StatusCode::OK, response.status());

  // AND returns setting with env value (not default)
  let setting = response.json::<SettingInfo>().await?;
  assert_eq!(BODHI_EXEC_VARIANT, setting.key);
  assert_eq!(
    serde_yaml::Value::String("aarch64-unknown-linux-gnu/cpu/llama-server".to_string()),
    setting.current_value
  );
  assert_eq!(SettingSource::Environment, setting.source);

  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_delete_setting_no_override(temp_dir: TempDir) -> anyhow::Result<()> {
  // GIVEN an app with no custom settings
  let setting_service =
    test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {}).await?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()
    .await?;
  let app = app(Arc::new(app_service)).await;

  // WHEN deleting a setting that's already at default
  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/v1/bodhi/settings/BODHI_EXEC_VARIANT")
        .body(Body::empty())?,
    )
    .await?;

  // THEN it succeeds
  assert_eq!(StatusCode::OK, response.status());

  // AND returns setting with default value
  let setting = response.json::<SettingInfo>().await?;
  assert_eq!(BODHI_EXEC_VARIANT, setting.key);
  assert_eq!(
    serde_yaml::Value::String(llama_server_proc::DEFAULT_VARIANT.to_string()),
    setting.current_value
  );
  assert_eq!(SettingSource::Default, setting.source);

  Ok(())
}

// Auth tier tests (merged from tests/routes_settings_auth_test.rs)

#[anyhow_trace]
#[rstest]
#[case::list_settings("GET", "/bodhi/v1/settings")]
#[case::update_setting("PUT", "/bodhi/v1/settings/some_key")]
#[case::delete_setting("DELETE", "/bodhi/v1/settings/some_key")]
#[tokio::test]
async fn test_settings_endpoints_reject_unauthenticated(
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
async fn test_settings_endpoints_reject_insufficient_role(
  #[values("resource_user", "resource_power_user", "resource_manager")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/settings"),
    ("PUT", "/bodhi/v1/settings/some_key"),
    ("DELETE", "/bodhi/v1/settings/some_key")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  assert_eq!(
    StatusCode::FORBIDDEN,
    response.status(),
    "{role} should be forbidden from {method} {path}"
  );
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[case::list_settings("GET", "/bodhi/v1/settings")]
#[tokio::test]
async fn test_settings_endpoints_allow_admin(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie =
    create_authenticated_session(app_service.session_service().as_ref(), &["resource_admin"])
      .await?;
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  // GET /bodhi/v1/settings returns 200 with real SettingServiceStub
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
