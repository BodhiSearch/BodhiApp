use crate::ENDPOINT_SETTINGS;
use axum::{
  extract::{Path, State},
  Json,
};
use objs::{ApiError, AppError, ErrorType, OpenAIApiError, SettingInfo, API_TAG_SETTINGS};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::{BODHI_EXEC_VARIANT, BODHI_HOME, BODHI_KEEP_ALIVE_SECS};
use std::sync::Arc;
use utoipa::ToSchema;

const EDIT_SETTINGS_ALLOWED: &[&str] = &[BODHI_EXEC_VARIANT, BODHI_KEEP_ALIVE_SECS];

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingsError {
  #[error("Setting '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("BODHI_HOME can only be changed via environment variable.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  BodhiHome,

  #[error("Updating setting '{0}' is not supported yet.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Unsupported(String),
}

/// Request to update a setting value
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "value": "debug"
}))]
pub struct UpdateSettingRequest {
  /// New value for the setting (type depends on setting metadata)
  #[schema(example = "debug")]
  pub value: serde_json::Value,
}

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
        (status = 403, description = "Forbidden - Admin session authentication required", body = OpenAIApiError),
    ),
    security(
        ("session_auth" = ["resource_admin"])
    )
)]
pub async fn list_settings_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<Vec<SettingInfo>>, ApiError> {
  let app_service = state.app_service();
  let settings = app_service.setting_service().list();
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
    description = "Updates the value of a specific application setting. The new value is validated against the setting's constraints and persisted to the settings file.\n\n**Security Note:** Admin session authentication required for system configuration changes.",
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
             "source": "settings_file",
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
                 "code": "settings_error-invalid_setting"
             }
         })),
        (status = 403, description = "Forbidden - Admin session authentication required", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["resource_admin"])
    )
)]
pub async fn update_setting_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(key): Path<String>,
  Json(payload): Json<UpdateSettingRequest>,
) -> Result<Json<SettingInfo>, ApiError> {
  let setting_service = state.app_service().setting_service();

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
        (status = 404, description = "Setting not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Setting not found: INVALID_KEY",
                 "type": "not_found_error",
                 "code": "settings_error-not_found"
             }
         })),
        (status = 403, description = "Forbidden - Admin session authentication required", body = OpenAIApiError)
    ),
    security(
        ("session_auth" = ["resource_admin"])
    )
)]
pub async fn delete_setting_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(key): Path<String>,
) -> Result<Json<SettingInfo>, ApiError> {
  let setting_service = state.app_service().setting_service();

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
#[path = "test_settings.rs"]
mod test_settings;
