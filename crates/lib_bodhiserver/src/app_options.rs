use crate::{BootstrapError, DefaultEnvWrapper};
use services::{AppType, DeploymentMode, EnvType};
use services::{
  EnvWrapper, Tenant, BODHI_APP_TYPE, BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_COMMIT_SHA,
  BODHI_DEPLOYMENT, BODHI_ENV_TYPE, BODHI_VERSION,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration options for setting up application directories and settings.
#[derive(Debug, Clone, derive_new::new)]
pub struct AppOptions {
  pub env_wrapper: Arc<dyn EnvWrapper>,
  pub env_type: EnvType,
  pub app_type: AppType,
  pub app_version: String,
  pub app_commit_sha: String,
  pub auth_url: String,
  pub auth_realm: String,
  pub deployment_mode: DeploymentMode,
  /// App settings (configurable via settings.yaml)
  pub app_settings: HashMap<String, String>,
  pub tenant: Option<Tenant>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AppOptionsBuilder {
  environment_vars: HashMap<String, String>,

  env_type: Option<EnvType>,
  app_type: Option<AppType>,
  app_version: Option<String>,
  app_commit_sha: Option<String>,
  auth_url: Option<String>,
  auth_realm: Option<String>,
  deployment_mode: Option<DeploymentMode>,

  app_settings: HashMap<String, String>,
  tenant: Option<Tenant>,
}

impl AppOptionsBuilder {
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
      BODHI_DEPLOYMENT => {
        let deployment_mode = value.parse::<DeploymentMode>()?;
        Ok(self.deployment_mode(deployment_mode))
      }
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

  pub fn deployment_mode(mut self, deployment_mode: DeploymentMode) -> Self {
    self.deployment_mode = Some(deployment_mode);
    self
  }

  pub fn set_tenant(mut self, instance: Tenant) -> Self {
    self.tenant = Some(instance);
    self
  }

  pub fn build(self) -> Result<AppOptions, BootstrapError> {
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
      deployment_mode: self
        .deployment_mode
        .ok_or_else(|| BootstrapError::ValidationError(BODHI_DEPLOYMENT.to_string()))?,
      app_settings: self.app_settings,
      tenant: self.tenant,
    })
  }

  fn build_env_wrapper_from_vars(&self) -> Arc<dyn EnvWrapper> {
    let mut env_wrapper = DefaultEnvWrapper::default();
    for (key, value) in &self.environment_vars {
      env_wrapper.set_var(key, value);
    }
    Arc::new(env_wrapper)
  }
}
