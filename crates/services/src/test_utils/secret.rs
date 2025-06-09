use crate::{
  asref_impl, AppRegInfo, AppRegInfoBuilder, AppStatus, KeyringError, KeyringStore, SecretService,
  SecretServiceError, SecretServiceExt,
};
use std::{collections::HashMap, sync::Mutex};

type Result<T> = std::result::Result<T, SecretServiceError>;

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
    self.set_app_status(status).unwrap();
    self
  }

  pub fn with_app_status_ready(self) -> Self {
    self.with_app_status(&AppStatus::Ready)
  }

  pub fn with_app_status_setup(self) -> Self {
    self.with_app_status(&AppStatus::Setup)
  }



  pub fn with_app_reg_info(self, app_reg_info: &AppRegInfo) -> Self {
    self.set_app_reg_info(app_reg_info).unwrap();
    self
  }

  pub fn with_app_reg_info_default(self) -> Self {
    self
      .set_app_reg_info(&AppRegInfoBuilder::test_default().build().unwrap())
      .unwrap();
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

  #[cfg(debug_assertions)]
  fn dump(&self) -> Result<String> {
    let store = self.store.lock().unwrap();
    Ok(serde_yaml::to_string(&(*store))?)
  }
}

impl Default for SecretServiceStub {
  fn default() -> Self {
    Self::new().with_app_status_ready()
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
  fn set_password(&self, key: &str, value: &str) -> std::result::Result<(), KeyringError> {
    let mut store = self.store.lock().unwrap();
    store.insert(key.to_string(), value.to_string());
    Ok(())
  }

  fn get_password(&self, key: &str) -> std::result::Result<Option<String>, KeyringError> {
    let store = self.store.lock().unwrap();
    Ok(store.get(key).map(|v| v.to_string()))
  }

  fn delete_password(&self, key: &str) -> std::result::Result<(), KeyringError> {
    let mut store = self.store.lock().unwrap();
    store.remove(key);
    Ok(())
  }
}
