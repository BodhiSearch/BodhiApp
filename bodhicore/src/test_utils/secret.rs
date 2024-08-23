use crate::service::{
  SecretService, SecretServiceError, APP_AUTHZ_FALSE, APP_AUTHZ_TRUE, APP_STATUS_READY,
  APP_STATUS_SETUP, KEY_APP_AUTHZ, KEY_APP_STATUS, KEY_PUBLIC_KEY,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SecretServiceStub {
  store: HashMap<String, String>,
}

impl SecretServiceStub {
  pub fn new() -> Self {
    Self {
      store: HashMap::new(),
    }
  }
}

impl SecretServiceStub {
  pub fn with_app_status_ready(&mut self) -> &mut Self {
    self
      .store
      .insert(KEY_APP_STATUS.to_string(), APP_STATUS_READY.to_string());
    self
  }

  pub fn with_app_status_setup(&mut self) -> &mut Self {
    self
      .store
      .insert(KEY_APP_STATUS.to_string(), APP_STATUS_SETUP.to_string());
    self
  }

  pub fn with_app_authz_disabled(&mut self) -> &mut Self {
    self
      .store
      .insert(KEY_APP_AUTHZ.to_string(), APP_AUTHZ_FALSE.to_string());
    self
  }

  pub fn with_app_authz_enabled(&mut self) -> &mut Self {
    self
      .store
      .insert(KEY_APP_AUTHZ.to_string(), APP_AUTHZ_TRUE.to_string());
    self
  }

  pub fn with_public_key(&mut self, public_key: String) -> &mut Self {
    self.store.insert(KEY_PUBLIC_KEY.to_string(), public_key);
    self
  }

  pub fn with(&mut self, key: String, value: String) -> &mut Self {
    self.store.insert(key, value);
    self
  }
}

impl SecretService for SecretServiceStub {
  fn set_secret(&mut self, key: &str, value: &str) -> Result<(), SecretServiceError> {
    self.store.insert(key.to_string(), value.to_string());
    Ok(())
  }

  fn get_secret(&self, key: &str) -> Result<Option<String>, SecretServiceError> {
    let value = self.store.get(key).map(|v| v.to_string());
    Ok(value)
  }

  fn delete_secret(&mut self, key: &str) -> Result<(), SecretServiceError> {
    self.store.remove(key);
    Ok(())
  }
}
