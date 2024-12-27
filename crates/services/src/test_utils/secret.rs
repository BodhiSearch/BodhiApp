use crate::{
  asref_impl, secret_service::Result, AppRegInfo, AppStatus, KeyringStore, SecretService,
  SecretServiceExt,
};
use std::{collections::HashMap, sync::Mutex};

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

  pub fn with_app_status(self, status: &AppStatus) -> Self {
    self.set_app_status(&status).unwrap();
    self
  }

  pub fn with_app_status_ready(self) -> Self {
    self.with_app_status(&AppStatus::Ready)
  }

  pub fn with_app_status_setup(self) -> Self {
    self.with_app_status(&AppStatus::Setup)
  }

  pub fn with_authz(self, authz: bool) -> Self {
    self.set_authz(authz).unwrap();
    self
  }

  pub fn with_authz_disabled(self) -> Self {
    self.with_authz(false)
  }

  pub fn with_authz_enabled(self) -> Self {
    self.with_authz(true)
  }

  pub fn with_app_reg_info(self, app_reg_info: &AppRegInfo) -> Self {
    self.set_app_reg_info(app_reg_info).unwrap();
    self
  }

  pub fn with(&mut self, key: String, value: String) -> &mut Self {
    self.store.lock().unwrap().insert(key, value);
    self
  }
}

asref_impl!(SecretService, SecretServiceStub);

impl SecretService for SecretServiceStub {
  fn set_secret_string(&self, key: &str, value: &str) -> Result<()> {
    let mut store = self.store.lock().unwrap();
    store.insert(key.to_string(), value.to_string());
    Ok(())
  }

  fn get_secret_string(&self, key: &str) -> Result<Option<String>> {
    let value = self.store.lock().unwrap().get(key).map(|v| v.to_string());
    Ok(value)
  }

  fn delete_secret(&self, key: &str) -> Result<()> {
    let mut store = self.store.lock().unwrap();
    store.remove(key);
    Ok(())
  }
}

impl Default for SecretServiceStub {
  fn default() -> Self {
    Self::new().with_app_status_ready().with_authz_enabled()
  }
}

#[derive(Debug)]
pub struct KeyringStoreStub {
  store: Mutex<HashMap<String, String>>,
}

impl KeyringStoreStub {
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

impl Default for KeyringStoreStub {
  fn default() -> Self {
    Self::new()
  }
}

impl KeyringStore for KeyringStoreStub {
  fn set_password(&self, key: &str, value: &str) -> Result<()> {
    let mut store = self.store.lock().unwrap();
    store.insert(key.to_string(), value.to_string());
    Ok(())
  }

  fn get_password(&self, key: &str) -> Result<Option<String>> {
    let store = self.store.lock().unwrap();
    Ok(store.get(key).map(|v| v.to_string()))
  }

  fn delete_password(&self, key: &str) -> Result<()> {
    let mut store = self.store.lock().unwrap();
    store.remove(key);
    Ok(())
  }
}
