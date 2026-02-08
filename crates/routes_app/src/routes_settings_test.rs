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
use objs::{test_utils::temp_dir, SettingInfo, SettingMetadata, SettingSource};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::{
  test_utils::{bodhi_home_setting, AppServiceStubBuilder, EnvWrapperStub},
  DefaultSettingService, SettingService, BODHI_EXEC_VARIANT, BODHI_HOST, BODHI_LOG_LEVEL,
  BODHI_PORT,
};
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

fn test_setting_service(
  temp_dir: &TempDir,
  envs: HashMap<String, String>,
  settings: HashMap<String, serde_yaml::Value>,
) -> Result<DefaultSettingService, anyhow::Error> {
  let settings_yaml = temp_dir.path().join("settings.yaml");
  let setting_service = DefaultSettingService::new_with_defaults(
    Arc::new(EnvWrapperStub::new(envs)),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    settings_yaml,
  );
  for (key, value) in settings {
    setting_service.set_setting_value(&key, &value);
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
  )?;

  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
    source: SettingSource::SettingsFile,
    metadata: SettingMetadata::Number { min: 1, max: 65535 },
  };
  assert_eq!(
    Some(&log_level),
    settings.iter().find(|k| k.key == BODHI_LOG_LEVEL)
  );
  assert_eq!(
    Some(&host),
    settings.iter().find(|k| k.key == BODHI_HOST)
  );
  assert_eq!(
    Some(&port),
    settings.iter().find(|k| k.key == BODHI_PORT)
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_routes_setting_update_success(temp_dir: TempDir) -> anyhow::Result<()> {
  let setting_service = test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
  let setting_service = test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
#[ignore = "enable when supporting editing other settings"]
#[tokio::test]
#[anyhow_trace]
async fn test_routes_setting_update_invalid_value(temp_dir: TempDir) -> anyhow::Result<()> {
  let setting_service = test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
  let setting_service = test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
  )?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
  let setting_service = test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
  )?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
  let setting_service = test_setting_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
  let app_service = AppServiceStubBuilder::default()
    .setting_service(Arc::new(setting_service))
    .build()?;
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
