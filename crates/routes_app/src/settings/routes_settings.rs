use crate::settings::error::SettingsRouteError;
use crate::shared::AuthScope;
use crate::{BodhiErrorResponse, ValidatedJson};
use crate::{API_TAG_SETTINGS, ENDPOINT_SETTINGS};
use axum::{extract::Path, Json};
use services::SettingInfo;
use services::{UpdateSettingRequest, BODHI_HOME, EDIT_SETTINGS_ALLOWED, LLM_SETTINGS};

/// List all application settings
///
/// **Security Note:** Admin session authentication required. Settings management
/// is restricted to interactive sessions to prevent unauthorized configuration changes.
#[utoipa::path(
    get,
    path = ENDPOINT_SETTINGS,
    tag = API_TAG_SETTINGS,
    operation_id = "listSettings",
    summary = "List Application Settings",
    description = "Retrieves all configurable application settings with their current values, default values, sources, and metadata including validation constraints.\n\n**Security Note:** Admin session authentication required for system configuration access.",
    responses(
        (status = 200, description = "Application settings retrieved successfully", body = Vec<SettingInfo>,
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
        (status = 403, description = "Forbidden - Admin session authentication required", body = BodhiErrorResponse),
    ),
    security(
        ("session_auth" = ["resource_admin"])
    )
)]
pub async fn settings_index(
  auth_scope: AuthScope,
) -> Result<Json<Vec<SettingInfo>>, BodhiErrorResponse> {
  let settings = auth_scope.settings().list().await;
  Ok(Json(settings))
}

/// Update a specific setting
///
/// **Security Note:** Admin session authentication required. Settings modification
/// is restricted to interactive sessions to prevent unauthorized system reconfiguration.
#[utoipa::path(
    put,
    path = ENDPOINT_SETTINGS.to_owned() + "/{key}",
    tag = API_TAG_SETTINGS,
    operation_id = "updateSetting",
    summary = "Update Application Setting",
    description = "Updates the value of a specific application setting. The new value is validated against the setting's constraints and persisted to the application database.\n\n**Security Note:** Admin session authentication required for system configuration changes.",
    params(
        ("key" = String, Path,
         description = "Setting key identifier (e.g., BODHI_LOG_LEVEL, BODHI_PORT)",
         example = "BODHI_LOG_LEVEL")
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
             "source": "database",
             "metadata": {
                 "type": "option",
                 "options": ["error", "warn", "info", "debug", "trace"]
             }
         })),
        (status = 404, description = "Setting not found", body = BodhiErrorResponse,
         example = json!({
             "error": {
                 "message": "Setting not found: INVALID_KEY",
                 "type": "not_found_error",
                 "code": "settings_route_error-not_found"
             }
         })),
        (status = 403, description = "Forbidden - Admin session authentication required", body = BodhiErrorResponse)
    ),
    security(
        ("session_auth" = ["resource_admin"])
    )
)]
pub async fn settings_update(
  auth_scope: AuthScope,
  Path(key): Path<String>,
  ValidatedJson(request): ValidatedJson<UpdateSettingRequest>,
) -> Result<Json<SettingInfo>, BodhiErrorResponse> {
  let settings = auth_scope.settings();

  if BODHI_HOME == key {
    return Err(SettingsRouteError::BodhiHome)?;
  }
  let (current_value, _source) = settings.get_setting_value_with_source(&key).await;
  if current_value.is_none() {
    return Err(SettingsRouteError::NotFound(key))?;
  }
  if !EDIT_SETTINGS_ALLOWED.contains(&key.as_str()) {
    return Err(SettingsRouteError::Unsupported(key))?;
  }
  if settings.is_multi_tenant().await && LLM_SETTINGS.contains(&key.as_str()) {
    return Err(SettingsRouteError::Unsupported(key))?;
  }
  let metadata = settings.get_setting_metadata(&key).await;
  let value = metadata.convert(request.value)?;
  settings.set_setting_value(&key, &value).await?;

  let (updated_value, source) = settings.get_setting_value_with_source(&key).await;
  let default_value = settings.get_default_value(&key).await;
  let metadata = settings.get_setting_metadata(&key).await;
  Ok(Json(SettingInfo {
    key,
    current_value: updated_value.unwrap_or(serde_yaml::Value::Null),
    default_value: default_value.unwrap_or(serde_yaml::Value::Null),
    source,
    metadata,
  }))
}

/// Reset a setting to its default value
///
/// **Security Note:** Admin session authentication required for system configuration.
#[utoipa::path(
    delete,
    path = ENDPOINT_SETTINGS.to_owned() + "/{key}",
    tag = API_TAG_SETTINGS,
    operation_id = "deleteSetting",
    summary = "Reset Setting to Default",
    description = "Resets a specific application setting to its default value by removing any custom overrides. Some critical settings like BODHI_HOME cannot be reset.\n\n**Security Note:** Admin session authentication required.",
    params(
        ("key" = String, Path,
         description = "Setting key identifier to reset to default value",
         example = "BODHI_LOG_LEVEL")
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
        (status = 404, description = "Setting not found", body = BodhiErrorResponse,
         example = json!({
             "error": {
                 "message": "Setting not found: INVALID_KEY",
                 "type": "not_found_error",
                 "code": "settings_route_error-not_found"
             }
         })),
        (status = 403, description = "Forbidden - Admin session authentication required", body = BodhiErrorResponse)
    ),
    security(
        ("session_auth" = ["resource_admin"])
    )
)]
pub async fn settings_destroy(
  auth_scope: AuthScope,
  Path(key): Path<String>,
) -> Result<Json<SettingInfo>, BodhiErrorResponse> {
  let settings = auth_scope.settings();

  if BODHI_HOME == key {
    return Err(SettingsRouteError::BodhiHome)?;
  }
  let (current_value, _source) = settings.get_setting_value_with_source(&key).await;
  if current_value.is_none() {
    return Err(SettingsRouteError::NotFound(key))?;
  }
  if !EDIT_SETTINGS_ALLOWED.contains(&key.as_str()) {
    return Err(SettingsRouteError::Unsupported(key))?;
  }
  if settings.is_multi_tenant().await && LLM_SETTINGS.contains(&key.as_str()) {
    return Err(SettingsRouteError::Unsupported(key))?;
  }
  settings.delete_setting(&key).await?;

  let (updated_value, source) = settings.get_setting_value_with_source(&key).await;
  let default_value = settings.get_default_value(&key).await;
  let metadata = settings.get_setting_metadata(&key).await;
  Ok(Json(SettingInfo {
    key,
    current_value: updated_value.unwrap_or(serde_yaml::Value::Null),
    default_value: default_value.unwrap_or(serde_yaml::Value::Null),
    source,
    metadata,
  }))
}
