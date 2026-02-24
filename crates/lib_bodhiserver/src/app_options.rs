use crate::{BootstrapError, DefaultEnvWrapper};
use objs::{AppType, EnvType};
use services::{
  AppRegInfo, AppStatus, EnvWrapper, BODHI_APP_TYPE, BODHI_AUTH_REALM, BODHI_AUTH_URL,
  BODHI_COMMIT_SHA, BODHI_ENV_TYPE, BODHI_VERSION,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration options for setting up application directories and settings.
/// Uses the builder pattern for flexible configuration with sensible defaults.
#[derive(Debug, Clone, derive_new::new)]
pub struct AppOptions {
  /// Environment wrapper for accessing environment variables and system paths
  pub env_wrapper: Arc<dyn EnvWrapper>,
  /// Environment type (Development, Production, etc.)
  pub env_type: EnvType,
  /// Application type (Native, Container, etc.)
  pub app_type: AppType,
  /// Application version string
  pub app_version: String,
  /// Application commit SHA
  pub app_commit_sha: String,
  /// Authentication server URL
  pub auth_url: String,
  /// Authentication realm
  pub auth_realm: String,
  /// App settings (configurable via settings.yaml)
  pub app_settings: HashMap<String, String>,
  /// OAuth client credentials (optional)
  pub app_reg_info: Option<AppRegInfo>,
  /// App initialization status (optional)
  pub app_status: Option<AppStatus>,
}

#[derive(Debug, Clone, derive_new::new)]
pub struct AppStateOption {
  /// OAuth client credentials (optional)
  pub app_reg_info: Option<AppRegInfo>,
  /// App initialization status (optional)
  pub app_status: Option<AppStatus>,
}

impl From<&AppOptions> for AppStateOption {
  fn from(options: &AppOptions) -> Self {
    Self {
      app_reg_info: options.app_reg_info.clone(),
      app_status: options.app_status.clone(),
    }
  }
}

/// Custom builder for AppOptions that handles internal state management
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AppOptionsBuilder {
  // Internal storage for environment variables
  environment_vars: HashMap<String, String>,

  // Core fields that will be set via system settings
  env_type: Option<EnvType>,
  app_type: Option<AppType>,
  app_version: Option<String>,
  app_commit_sha: Option<String>,
  auth_url: Option<String>,
  auth_realm: Option<String>,

  // Configuration fields
  app_settings: HashMap<String, String>,
  app_reg_info: Option<AppRegInfo>,
  app_status: Option<AppStatus>,
}

impl AppOptionsBuilder {
  /// Sets an environment variable
  pub fn set_env(mut self, key: &str, value: &str) -> Self {
    self
      .environment_vars
      .insert(key.to_string(), value.to_string());
    self
  }

  /// Sets an app setting (configurable via settings.yaml)
  pub fn set_app_setting(mut self, key: &str, value: &str) -> Self {
    self.app_settings.insert(key.to_string(), value.to_string());
    self
  }

  /// Sets a system setting (immutable)
  pub fn set_system_setting(self, key: &str, value: &str) -> Result<Self, BootstrapError> {
    // Validate and set system settings
    match key {
      BODHI_ENV_TYPE => {
        let env_type = value.parse::<EnvType>()?;
        Ok(self.env_type(env_type))
      }
      BODHI_APP_TYPE => {
        let app_type = value.parse::<AppType>()?;
        Ok(self.app_type(app_type))
      }
      BODHI_VERSION => Ok(self.app_version(value)),
      BODHI_COMMIT_SHA => Ok(self.app_commit_sha(value)),
      BODHI_AUTH_URL => Ok(self.auth_url(value)),
      BODHI_AUTH_REALM => Ok(self.auth_realm(value)),
      key => Err(BootstrapError::UnknownSystemSetting(key.to_string())),
    }
  }

  pub fn app_type(mut self, app_type: AppType) -> Self {
    self.app_type = Some(app_type);
    self
  }

  pub fn env_type(mut self, env_type: EnvType) -> Self {
    self.env_type = Some(env_type);
    self
  }

  pub fn app_version(mut self, app_version: &str) -> Self {
    self.app_version = Some(app_version.to_string());
    self
  }

  pub fn app_commit_sha(mut self, app_commit_sha: &str) -> Self {
    self.app_commit_sha = Some(app_commit_sha.to_string());
    self
  }

  pub fn auth_url(mut self, auth_url: &str) -> Self {
    self.auth_url = Some(auth_url.to_string());
    self
  }

  pub fn auth_realm(mut self, auth_realm: &str) -> Self {
    self.auth_realm = Some(auth_realm.to_string());
    self
  }

  /// Sets OAuth client credentials
  pub fn set_app_reg_info(mut self, client_id: &str, client_secret: &str) -> Self {
    self.app_reg_info = Some(AppRegInfo {
      client_id: client_id.to_string(),
      client_secret: client_secret.to_string(),
      scope: format!("scope_{}", client_id),
    });
    self
  }

  /// Sets app initialization status
  pub fn set_app_status(mut self, status: AppStatus) -> Self {
    self.app_status = Some(status);
    self
  }

  /// Builds the AppOptions with validation and environment wrapper setup
  pub fn build(self) -> Result<AppOptions, BootstrapError> {
    // Always build environment wrapper, even if no environment variables were set
    let env_wrapper = self.build_env_wrapper_from_vars();

    Ok(AppOptions {
      env_wrapper,
      env_type: self
        .env_type
        .ok_or_else(|| BootstrapError::ValidationError(BODHI_ENV_TYPE.to_string()))?,
      app_type: self
        .app_type
        .ok_or_else(|| BootstrapError::ValidationError(BODHI_APP_TYPE.to_string()))?,
      app_version: self
        .app_version
        .ok_or_else(|| BootstrapError::ValidationError(BODHI_VERSION.to_string()))?,
      app_commit_sha: self
        .app_commit_sha
        .unwrap_or_else(|| crate::BUILD_COMMIT_SHA.to_string()),
      auth_url: self
        .auth_url
        .ok_or_else(|| BootstrapError::ValidationError(BODHI_AUTH_URL.to_string()))?,
      auth_realm: self
        .auth_realm
        .ok_or_else(|| BootstrapError::ValidationError(BODHI_AUTH_REALM.to_string()))?,
      app_settings: self.app_settings,
      app_reg_info: self.app_reg_info,
      app_status: self.app_status,
    })
  }

  /// Builds an environment wrapper with collected environment variables
  fn build_env_wrapper_from_vars(&self) -> Arc<dyn EnvWrapper> {
    let mut env_wrapper = DefaultEnvWrapper::default();
    for (key, value) in &self.environment_vars {
      env_wrapper.set_var(key, value);
    }
    Arc::new(env_wrapper)
  }
}
