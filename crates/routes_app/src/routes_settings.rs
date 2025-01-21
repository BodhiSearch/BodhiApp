use crate::ENDPOINT_SETTINGS;
use axum::{extract::State, Json};
use objs::{ApiError, OpenAIApiError, SettingInfo};
use server_core::RouterState;
use std::sync::Arc;

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

#[cfg(test)]
mod tests {
  use super::list_settings_handler;
  use crate::ENDPOINT_SETTINGS;
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
  };
  use objs::{SettingInfo, SettingMetadata, SettingSource};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
  use services::test_utils::{AppServiceStubBuilder, EnvServiceStub};
  use std::sync::Arc;
  use tower::ServiceExt;

  async fn app(app_service: Arc<dyn services::AppService>) -> Router {
    let router_state = DefaultRouterState::new(Arc::new(MockSharedContext::default()), app_service);
    Router::new()
      .route(ENDPOINT_SETTINGS, get(list_settings_handler))
      .with_state(Arc::new(router_state))
  }

  #[anyhow_trace]
  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_settings_get() -> anyhow::Result<()> {
    // GIVEN app with auth disabled
    let env_service = EnvServiceStub::new(maplit::hashmap! {
      "BODHI_LOG_LEVEL".to_string() => "info".to_string(),
      "BODHI_HOST".to_string() => "test.host".to_string(),
    });
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
    let mut settings = response.json::<Vec<SettingInfo>>().await?;
    settings.sort_by(|a, b| a.key.cmp(&b.key));
    let mut expected: Vec<SettingInfo> = vec![
      SettingInfo {
        key: "BODHI_LOG_LEVEL".to_string(),
        current_value: serde_yaml::Value::String("info".to_string()),
        default_value: serde_yaml::Value::Null,
        source: SettingSource::Environment,
        metadata: SettingMetadata::String,
      },
      SettingInfo {
        key: "BODHI_HOST".to_string(),
        current_value: serde_yaml::Value::String("test.host".to_string()),
        default_value: serde_yaml::Value::Null,
        source: SettingSource::Environment,
        metadata: SettingMetadata::String,
      },
    ];
    expected.sort_by(|a, b| a.key.cmp(&b.key));
    assert_eq!(expected, settings);
    Ok(())
  }
}
