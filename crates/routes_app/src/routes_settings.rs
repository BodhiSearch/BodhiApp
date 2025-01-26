use crate::ENDPOINT_SETTINGS;
use axum::{
  extract::{Path, State},
  Json,
};
use objs::{ApiError, AppError, ErrorType, OpenAIApiError, SettingInfo};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{BODHI_EXEC_VARIANT, BODHI_HOME, BODHI_KEEP_ALIVE_SECS};
use std::sync::Arc;
use utoipa::ToSchema;

const EDIT_SETTINGS_ALLOWED: &[&str] = &[BODHI_EXEC_VARIANT, BODHI_KEEP_ALIVE_SECS];

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingsError {
  #[error("not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("bodhi_home")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  BodhiHome,

  #[error("unsupported")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Unsupported(String),
}

/// Request to update a setting value
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "value": "debug"
}))]
pub struct UpdateSettingRequest {
  pub value: serde_json::Value,
}

/// List all application settings
#[utoipa::path(
    get,
    path = ENDPOINT_SETTINGS,
    tag = "settings",
    operation_id = "listSettings",
    responses(
        (status = 200, description = "List of application settings", body = Vec<SettingInfo>,
         example = json!([
             {
                 "key": "BODHI_LOG_LEVEL",
                 "current_value": "info",
                 "default_value": "warn",
                 "source": "environment",
                 "metadata": {
                     "type": "option",
                     "options": ["error", "warn", "info", "debug", "trace"]
                 }
             },
             {
                 "key": "BODHI_PORT",
                 "current_value": 1135,
                 "default_value": 1135,
                 "source": "default",
                 "metadata": {
                     "type": "number",
                     "min": 1025,
                     "max": 65535
                 }
             }
         ])),
        (status = 401, description = "Unauthorized - User is not an admin", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Only administrators can view settings",
                 "type": "unauthorized_error",
                 "code": "settings_error-unauthorized"
             }
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn list_settings_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<Vec<SettingInfo>>, ApiError> {
  let app_service = state.app_service();
  let settings = app_service.env_service().list();
  Ok(Json(settings))
}

/// Update a specific setting
#[utoipa::path(
    put,
    path = ENDPOINT_SETTINGS.to_owned() + "/{key}",
    tag = "settings",
    operation_id = "updateSetting",
    params(
        ("key" = String, Path, description = "Setting key to update")
    ),
    request_body(content = inline(UpdateSettingRequest), description = "New setting value",
        example = json!({
            "value": "debug"
        })
    ),
    responses(
        (status = 200, description = "Setting updated successfully", body = SettingInfo,
         example = json!({
             "key": "BODHI_LOG_LEVEL",
             "current_value": "debug",
             "default_value": "warn",
             "source": "settings_file",
             "metadata": {
                 "type": "option",
                 "options": ["error", "warn", "info", "debug", "trace"]
             }
         })),
        (status = 400, description = "Invalid setting or value", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Invalid value for setting: value out of range",
                 "type": "invalid_request_error",
                 "code": "settings_error-validation_error"
             }
         })),
        (status = 404, description = "Setting not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Setting not found: INVALID_KEY",
                 "type": "not_found_error", 
                 "code": "settings_error-invalid_setting"
             }
         }))
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn update_setting_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(key): Path<String>,
  Json(payload): Json<UpdateSettingRequest>,
) -> Result<Json<SettingInfo>, ApiError> {
  let setting_service = state.app_service().env_service().setting_service();

  // Validate setting exists
  if BODHI_HOME == key {
    return Err(SettingsError::BodhiHome)?;
  }
  let settings = setting_service.list();
  let setting = settings
    .iter()
    .find(|s| s.key == key)
    .ok_or_else(|| SettingsError::NotFound(key.clone()))?;

  if !EDIT_SETTINGS_ALLOWED.contains(&key.as_str()) {
    return Err(SettingsError::Unsupported(key))?;
  }
  // Validate new value against metadata
  let value = setting.metadata.convert(payload.value)?;
  setting_service.set_setting_value(&key, &value);

  // Get updated setting
  let settings = setting_service.list();
  let updated = settings
    .into_iter()
    .find(|s| s.key == key)
    .ok_or_else(|| SettingsError::NotFound(key.clone()))?;

  Ok(Json(updated))
}

/// Reset a setting to its default value
#[utoipa::path(
    delete,
    path = ENDPOINT_SETTINGS.to_owned() + "/{key}",
    tag = "settings", 
    operation_id = "deleteSetting",
    params(
        ("key" = String, Path, description = "Setting key to reset")
    ),
    responses(
        (status = 200, description = "Setting reset to default successfully", body = SettingInfo,
         example = json!({
             "key": "BODHI_LOG_LEVEL",
             "current_value": "warn",
             "default_value": "warn", 
             "source": "default",
             "metadata": {
                 "type": "option",
                 "options": ["error", "warn", "info", "debug", "trace"]
             }
         })),
        (status = 404, description = "Setting not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Setting not found: INVALID_KEY",
                 "type": "not_found_error",
                 "code": "settings_error-not_found"
             }
         }))
    ),
    security(
        ("session_auth" = [])
    )
)]
pub async fn delete_setting_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(key): Path<String>,
) -> Result<Json<SettingInfo>, ApiError> {
  let setting_service = state.app_service().env_service().setting_service();

  if BODHI_HOME == key {
    return Err(SettingsError::BodhiHome)?;
  }
  // Validate setting exists
  let settings = setting_service.list();
  let _ = settings
    .iter()
    .find(|s| s.key == key)
    .ok_or_else(|| SettingsError::NotFound(key.clone()))?;

  if !EDIT_SETTINGS_ALLOWED.contains(&key.as_str()) {
    return Err(SettingsError::Unsupported(key))?;
  }
  // Delete setting (reset to default)
  setting_service.delete_setting(&key)?;

  // Get updated setting info
  let settings = setting_service.list();
  let updated = settings
    .into_iter()
    .find(|s| s.key == key)
    .ok_or_else(|| SettingsError::NotFound(key.clone()))?;

  Ok(Json(updated))
}

#[cfg(test)]
mod tests {
  use super::{delete_setting_handler, list_settings_handler, update_setting_handler};
  use crate::ENDPOINT_SETTINGS;
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{delete, get, put},
    Router,
  };
  use objs::{test_utils::temp_dir, AppType, EnvType, SettingInfo, SettingMetadata, SettingSource};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::json;
  use server_core::{
    test_utils::{RequestTestExt, ResponseTestExt},
    DefaultRouterState, MockSharedContext,
  };
  use services::{
    test_utils::{bodhi_home_setting, AppServiceStubBuilder, EnvWrapperStub},
    DefaultEnvService, DefaultSettingService, SettingService, BODHI_EXEC_VARIANT, BODHI_HOST,
    BODHI_LOG_LEVEL, BODHI_PORT,
  };
  use std::{collections::HashMap, str::FromStr, sync::Arc};
  use tempfile::TempDir;
  use tower::ServiceExt;

  async fn app(app_service: Arc<dyn services::AppService>) -> Router {
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), app_service);
    Router::new()
      .route(ENDPOINT_SETTINGS, get(list_settings_handler))
      .route("/v1/bodhi/settings/:key", put(update_setting_handler))
      .route("/v1/bodhi/settings/:key", delete(delete_setting_handler))
      .with_state(Arc::new(router_state))
  }

  fn test_env_service(
    temp_dir: &TempDir,
    envs: HashMap<String, String>,
    settings: HashMap<String, serde_yaml::Value>,
  ) -> Result<DefaultEnvService, anyhow::Error> {
    let settings_yaml = temp_dir.path().join("settings.yaml");
    let setting_service = DefaultSettingService::new_with_defaults(
      Arc::new(EnvWrapperStub::new(envs)),
      bodhi_home_setting(&temp_dir.path(), SettingSource::Environment),
      vec![],
      settings_yaml,
    )?;
    for (key, value) in settings {
      setting_service.set_setting_value(&key, &value);
    }
    let env_service = DefaultEnvService::new(
      EnvType::Development,
      AppType::Native,
      "http://auth.url".to_string(),
      "test-realm".to_string(),
      Arc::new(setting_service),
    )?;
    Ok(env_service)
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_settings_get(temp_dir: TempDir) -> anyhow::Result<()> {
    // GIVEN app with auth disabled
    let env_service = test_env_service(
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
      .env_service(Arc::new(env_service))
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
      default_value: serde_yaml::Value::String("localhost".to_string()),
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
      &log_level,
      settings.iter().find(|k| k.key == BODHI_LOG_LEVEL).unwrap()
    );
    assert_eq!(
      &host,
      settings.iter().find(|k| k.key == BODHI_HOST).unwrap()
    );
    assert_eq!(
      &port,
      settings.iter().find(|k| k.key == BODHI_PORT).unwrap()
    );
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_setting_success(temp_dir: TempDir) -> anyhow::Result<()> {
    let env_service = test_env_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(env_service))
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

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_setting_invalid_key(temp_dir: TempDir) -> anyhow::Result<()> {
    let env_service = test_env_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(env_service))
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
      "settings_error-not_found",
      error["error"]["code"].as_str().unwrap()
    );

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[ignore = "enable when supporting editing other settings"]
  #[awt]
  #[tokio::test]
  async fn test_update_setting_invalid_value(temp_dir: TempDir) -> anyhow::Result<()> {
    let env_service = test_env_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(env_service))
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
      json! {{
        "message": "cannot parse \u{2068}\"not a number\"\u{2069} as \u{2068}Number\u{2069}",
        "type": "invalid_request_error",
        "code": "settings_metadata_error-invalid_value_type"
      }},
      error["error"]
    );
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[ignore = "enable when supporting editing other settings"]
  #[awt]
  #[tokio::test]
  async fn test_update_setting_invalid_value_out_of_range(temp_dir: TempDir) -> anyhow::Result<()> {
    let env_service = test_env_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(env_service))
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
      json! {{
        "message": "passed value is not a valid value: \u{2068}1000\u{2069}",
        "type": "invalid_request_error",
        "code": "settings_metadata_error-invalid_value"
      }},
      error["error"]
    );
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_delete_setting_success(temp_dir: TempDir) -> anyhow::Result<()> {
    // GIVEN an app with a custom setting value
    let env_service = test_env_service(
      &temp_dir,
      maplit::hashmap! {},
      maplit::hashmap! {
        BODHI_EXEC_VARIANT.to_string() => serde_yaml::Value::String("metal".to_string()),
      },
    )?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(env_service))
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

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_delete_setting_invalid_key(temp_dir: TempDir) -> anyhow::Result<()> {
    // GIVEN an app
    let env_service = test_env_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(env_service))
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
      "settings_error-not_found",
      error["error"]["code"].as_str().unwrap()
    );

    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_delete_setting_with_env_override(temp_dir: TempDir) -> anyhow::Result<()> {
    // GIVEN an app with both env and file settings
    let env_service = test_env_service(
      &temp_dir,
      maplit::hashmap! {
        BODHI_EXEC_VARIANT.to_string() => "aarch64-unknown-linux-gnu/cpu/llama-server".to_string(),
      },
      maplit::hashmap! {
        BODHI_EXEC_VARIANT.to_string() => serde_yaml::Value::String("aarch64-unknown-linux-gnu/cuda/llama-server".to_string()),
      },
    )?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(env_service))
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

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_delete_setting_no_override(temp_dir: TempDir) -> anyhow::Result<()> {
    // GIVEN an app with no custom settings
    let env_service = test_env_service(&temp_dir, maplit::hashmap! {}, maplit::hashmap! {})?;
    let app_service = AppServiceStubBuilder::default()
      .env_service(Arc::new(env_service))
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
}
