use crate::service::cache_service::{CacheService, MokaCacheService};
use keyring::Entry;
use std::sync::Arc;
use thiserror::Error;

pub const KEY_APP_STATUS: &str = "app_status";
pub const APP_STATUS_READY: &str = "ready";
pub const APP_STATUS_SETUP: &str = "setup";
pub const KEY_APP_AUTHZ: &str = "app_authz";
pub const APP_AUTHZ_TRUE: &str = "true";
pub const APP_AUTHZ_FALSE: &str = "false";
pub const KEY_PUBLIC_KEY: &str = "public_key";
pub const KEY_ISSUER: &str = "issuer";
pub const KEY_RESOURCE_TOKEN: &str = "X-Resource-Token";

#[derive(Debug, Error)]
pub enum SecretServiceError {
  #[error("secret_service error: {0}")]
  KeyringError(#[from] keyring::Error),
  #[error("Secret not found")]
  SecretNotFound,
}

pub type Result<T> = std::result::Result<T, SecretServiceError>;

#[cfg_attr(test, mockall::automock)]
pub trait SecretService: Send + Sync + std::fmt::Debug {
  fn set_secret(&mut self, key: &str, value: &str) -> Result<()>;

  fn get_secret(&self, key: &str) -> Result<Option<String>>;

  fn delete_secret(&mut self, key: &str) -> Result<()>;
}

#[derive(Debug)]
pub struct KeyringSecretService {
  service_name: String,
  cache: Arc<dyn CacheService>,
}

impl KeyringSecretService {
  pub fn new(service_name: String) -> Self {
    let cache = Arc::new(MokaCacheService::new(None, None));
    Self {
      service_name,
      cache,
    }
  }

  pub fn with_cache(service_name: String, cache: Arc<dyn CacheService>) -> Self {
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
  fn set_secret(&mut self, key: &str, value: &str) -> Result<()> {
    self.entry(key)?.set_password(value)?;
    self.cache.set(key, value);
    Ok(())
  }

  fn get_secret(&self, key: &str) -> Result<Option<String>> {
    if let Some(cached_value) = self.cache.get(key) {
      return Ok(Some(cached_value));
    }

    match self.entry(key)?.get_password() {
      Ok(value) => {
        self.cache.set(key, &value);
        Ok(Some(value))
      }
      Err(keyring::Error::NoEntry) => Ok(None),
      Err(e) => Err(SecretServiceError::KeyringError(e)),
    }
  }

  fn delete_secret(&mut self, key: &str) -> Result<()> {
    self.entry(key)?.delete_credential()?;
    self.cache.remove(key);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::service::cache_service::MokaCacheService;
  use std::time::Duration;

  #[test]
  fn test_secret_service_with_cache() {
    let cache = Arc::new(MokaCacheService::new(
      Some(100),
      Some(Duration::from_secs(60)),
    ));
    let mut service = KeyringSecretService::with_cache("bodhi_test".to_string(), cache.clone());

    // Set and get from keyring
    service.set_secret("test_key", "test_value").unwrap();
    let value = service.get_secret("test_key").unwrap();
    assert_eq!(value, Some("test_value".to_string()));

    // Verify it's in the cache
    assert_eq!(cache.get("test_key"), Some("test_value".to_string()));

    // Get from cache
    let cached_value = service.get_secret("test_key").unwrap();
    assert_eq!(cached_value, Some("test_value".to_string()));

    // Delete and verify it's removed from both keyring and cache
    service.delete_secret("test_key").unwrap();
    assert!(matches!(service.get_secret("test_key"), Ok(None)));
    assert_eq!(cache.as_ref().get("test_key"), None);
  }
}
