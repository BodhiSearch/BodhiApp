use keyring::Entry;
use thiserror::Error;

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
  fn set_secret(&self, key: &str, value: &str) -> Result<()>;
  fn get_secret(&self, key: &str) -> Result<String>;
  fn delete_secret(&self, key: &str) -> Result<()>;
}

#[derive(Debug)]
pub struct KeyringSecretService {
  service_name: String,
}

impl KeyringSecretService {
  pub fn new(service_name: String) -> Self {
    Self { service_name }
  }

  fn entry(&self, key: &str) -> Result<Entry> {
    let result = Entry::new(&self.service_name, key)?;
    Ok(result)
  }
}

impl SecretService for KeyringSecretService {
  fn set_secret(&self, key: &str, value: &str) -> Result<()> {
    self.entry(key)?.set_password(value)?;
    Ok(())
  }

  fn get_secret(&self, key: &str) -> Result<String> {
    self.entry(key)?.get_password().map_err(|e| match e {
      keyring::Error::NoEntry => SecretServiceError::SecretNotFound,
      _ => SecretServiceError::KeyringError(e),
    })
  }

  fn delete_secret(&self, key: &str) -> Result<()> {
    self.entry(key)?.delete_credential()?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_secret_service() {
    let service = KeyringSecretService::new("bodhi_test".to_string());
    service.set_secret("test_key", "test_value").unwrap();
    let value = service.get_secret("test_key").unwrap();
    assert_eq!(value, "test_value");

    service.delete_secret("test_key").unwrap();

    assert!(matches!(
      service.get_secret("test_key"),
      Err(SecretServiceError::SecretNotFound)
    ));
  }
}
