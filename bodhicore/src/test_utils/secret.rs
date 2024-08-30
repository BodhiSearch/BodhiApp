use crate::service::{
  AppRegInfo, CacheService, ISecretService, KeyringSecretService, SecretServiceError,
  APP_AUTHZ_FALSE, APP_AUTHZ_TRUE, APP_STATUS_READY, KEY_APP_AUTHZ, KEY_APP_REG_INFO,
  KEY_APP_STATUS,
};
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

impl KeyringSecretService {
  pub fn with_cache(service_name: String, cache: Arc<dyn CacheService>) -> Self {
    Self::new(service_name, cache)
  }
}

#[derive(Debug)]
pub struct SecretServiceStub {
  store: Mutex<HashMap<String, String>>,
}

impl SecretServiceStub {
  pub fn new() -> Self {
    Self {
      store: Mutex::new(HashMap::new()),
    }
  }

  pub fn with_map(map: HashMap<String, String>) -> Self {
    Self {
      store: Mutex::new(map),
    }
  }
}

impl Default for SecretServiceStub {
  fn default() -> Self {
    let mut slf = Self::new();
    slf.with_app_status_ready();
    slf.with_app_authz_enabled();
    slf
  }
}

impl SecretServiceStub {
  pub fn with_app_status_ready(&mut self) -> &mut Self {
    self.with(KEY_APP_STATUS.to_string(), APP_STATUS_READY.to_string());
    self
  }

  pub fn with_app_authz_disabled(&mut self) -> &mut Self {
    self.with(KEY_APP_AUTHZ.to_string(), APP_AUTHZ_FALSE.to_string());
    self
  }

  pub fn with_app_authz_enabled(&mut self) -> &mut Self {
    self.with(KEY_APP_AUTHZ.to_string(), APP_AUTHZ_TRUE.to_string());
    self
  }

  pub fn with_app_reg_info(&mut self, app_reg_info: &AppRegInfo) -> &mut Self {
    let value = serde_json::to_string(app_reg_info).unwrap();
    self.with(KEY_APP_REG_INFO.to_string(), value);
    self
  }

  pub fn with(&mut self, key: String, value: String) -> &mut Self {
    self.store.lock().unwrap().insert(key, value);
    self
  }
}

impl ISecretService for SecretServiceStub {
  fn set_secret_string(&self, key: &str, value: &str) -> Result<(), SecretServiceError> {
    let mut store = self.store.lock().unwrap();
    store.insert(key.to_string(), value.to_string());
    Ok(())
  }

  fn get_secret_string(&self, key: &str) -> Result<Option<String>, SecretServiceError> {
    let value = self.store.lock().unwrap().get(key).map(|v| v.to_string());
    Ok(value)
  }

  fn delete_secret(&self, key: &str) -> Result<(), SecretServiceError> {
    let mut store = self.store.lock().unwrap();
    store.remove(key);
    Ok(())
  }
}
