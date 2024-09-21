use crate::{asref_impl, CacheService, MokaCacheService};
use derive_new::new;
use keyring::Entry;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub const KEY_APP_STATUS: &str = "app_status";
pub const APP_STATUS_READY: &str = "ready";
pub const APP_STATUS_SETUP: &str = "setup";
pub const KEY_APP_AUTHZ: &str = "app_authz";
pub const APP_AUTHZ_TRUE: &str = "true";
pub const APP_AUTHZ_FALSE: &str = "false";
pub const KEY_RESOURCE_TOKEN: &str = "X-Resource-Token";
pub const KEY_APP_REG_INFO: &str = "app_reg_info";

#[derive(Debug, Clone, thiserror::Error)]
pub enum SecretServiceError {
  #[error("{0}")]
  KeyringError(String),
  #[error("Secret not found")]
  SecretNotFound,
  #[error("{0}")]
  SerdeJsonError(String),
}

impl From<serde_json::Error> for SecretServiceError {
  fn from(err: serde_json::Error) -> Self {
    SecretServiceError::SerdeJsonError(format!("{:?}", err))
  }
}

impl From<keyring::Error> for SecretServiceError {
  fn from(err: keyring::Error) -> Self {
    SecretServiceError::KeyringError(format!("{:?}", err))
  }
}

pub type Result<T> = std::result::Result<T, SecretServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait SecretService: Send + Sync + std::fmt::Debug {
  // TODO: make it async so can have async mutex
  fn set_secret_string(&self, key: &str, value: &str) -> Result<()>;

  fn get_secret_string(&self, key: &str) -> Result<Option<String>>;

  // TODO: make it async so can have async mutex
  fn delete_secret(&self, key: &str) -> Result<()>;
}

pub fn set_secret<S, T>(slf: S, key: &str, value: T) -> Result<()>
where
  T: serde::Serialize,
  S: AsRef<dyn SecretService>,
{
  let value_str = serde_json::to_string(&value)?;
  slf.as_ref().set_secret_string(key, &value_str)
}

pub fn get_secret<S, T>(slf: S, key: &str) -> Result<Option<T>>
where
  T: DeserializeOwned,
  S: AsRef<dyn SecretService>,
{
  match slf.as_ref().get_secret_string(key)? {
    Some(value) => {
      let result = serde_json::from_str::<T>(&value)?;
      Ok(Some(result))
    }
    None => Ok(None),
  }
}

asref_impl!(SecretService, KeyringSecretService);

#[derive(Debug, new)]
pub struct KeyringSecretService {
  service_name: String,
  cache: Arc<dyn CacheService>,
}

impl KeyringSecretService {
  pub fn with_service_name(service_name: String) -> Self {
    let cache = Arc::new(MokaCacheService::default());
    Self {
      service_name,
      cache,
    }
  }

  fn entry(&self, key: &str) -> Result<Entry> {
    let result = Entry::new(&self.service_name, key)?;
    Ok(result)
  }
}

impl SecretService for KeyringSecretService {
  fn set_secret_string(&self, key: &str, value: &str) -> Result<()> {
    self.entry(key)?.set_password(value)?;
    self.cache.set(key, value);
    Ok(())
  }

  fn get_secret_string(&self, key: &str) -> Result<Option<String>> {
    if let Some(cached_value) = self.cache.get(key) {
      return Ok(Some(cached_value));
    }

    match self.entry(key)?.get_password() {
      Ok(value) => {
        self.cache.set(key, &value);
        Ok(Some(value))
      }
      Err(keyring::Error::NoEntry) => Ok(None),
      Err(e) => Err(SecretServiceError::KeyringError(e.to_string())),
    }
  }

  fn delete_secret(&self, key: &str) -> Result<()> {
    self.entry(key)?.delete_credential()?;
    self.cache.remove(key);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    get_secret, set_secret, CacheService, KeyringSecretService, MokaCacheService, SecretService,
  };
  use serde::{Deserialize, Serialize};
  use std::sync::Arc;

  #[test]
  fn test_secret_service_with_cache() {
    let cache = Arc::new(MokaCacheService::default());
    let service = KeyringSecretService::with_cache("bodhi_test".to_string(), cache.clone());
    service.set_secret_string("test_key", "test_value").unwrap();
    let value = service.get_secret_string("test_key").unwrap();
    assert_eq!(value, Some("test_value".to_string()));
    assert_eq!(cache.get("test_key"), Some("test_value".to_string()));
    let cached_value = service.get_secret_string("test_key").unwrap();
    assert_eq!(cached_value, Some("test_value".to_string()));
    service.delete_secret("test_key").unwrap();
    assert!(matches!(service.get_secret_string("test_key"), Ok(None)));
    assert_eq!(cache.as_ref().get("test_key"), None);
  }

  #[test]
  fn test_secret_service_with_serialized_object() -> anyhow::Result<()> {
    let cache = Arc::new(MokaCacheService::default());
    let mut service = Arc::new(KeyringSecretService::with_cache(
      "bodhi_test".to_string(),
      cache,
    ));
    // Create a test struct
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestObject {
      name: String,
      age: u32,
    }

    let test_object = TestObject {
      name: "Alice".to_string(),
      age: 30,
    };

    set_secret(&mut service, "test_object", &test_object)?;
    let retrieved_object: Option<TestObject> = get_secret(&service, "test_object").unwrap();

    assert_eq!(retrieved_object, Some(test_object));

    service.delete_secret("test_object").unwrap();
    let deleted_object: Option<TestObject> = get_secret(&service, "test_object").unwrap();

    assert_eq!(deleted_object, None);
    Ok(())
  }
}
