use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use keyring::Entry;
use objs::{AppError, ErrorType};
use rand::{rng, RngCore};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum KeyringError {
  #[error("Keyring access failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  KeyringError(#[from] keyring::Error),
  #[error("Keyring data decode failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  DecodeError(#[from] base64::DecodeError),
}

type Result<T> = std::result::Result<T, KeyringError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait KeyringStore: Send + Sync + std::fmt::Debug {
  fn get_or_generate(&self, key: &str) -> Result<Vec<u8>> {
    Ok(match self.get_password(key)? {
      Some(stored_key) => BASE64.decode(&stored_key)?,
      None => {
        let generated = generate_random_key();
        self.set_password(key, &BASE64.encode(&generated))?;
        generated
      }
    })
  }

  fn set_password(&self, key: &str, value: &str) -> Result<()>;
  fn get_password(&self, key: &str) -> Result<Option<String>>;
  fn delete_password(&self, key: &str) -> Result<()>;
}

pub fn hash_key(key: &str) -> Vec<u8> {
  let mut hasher = Sha256::new();
  hasher.update(key.as_bytes());
  hasher.finalize().to_vec()
}

#[derive(Debug)]
pub struct SystemKeyringStore {
  service_name: String,
}

impl SystemKeyringStore {
  pub fn new(service_name: &str) -> Self {
    Self {
      service_name: service_name.to_string(),
    }
  }

  fn entry(&self, key: &str) -> Result<Entry> {
    Ok(Entry::new(&self.service_name, key)?)
  }
}

impl KeyringStore for SystemKeyringStore {
  fn set_password(&self, key: &str, value: &str) -> Result<()> {
    Ok(self.entry(key)?.set_password(value)?)
  }

  fn get_password(&self, key: &str) -> Result<Option<String>> {
    match self.entry(key)?.get_password() {
      Ok(value) => Ok(Some(value)),
      Err(keyring::Error::NoEntry) => Ok(None),
      Err(err) => Err(err.into()),
    }
  }

  fn delete_password(&self, key: &str) -> Result<()> {
    match self.entry(key)?.delete_credential() {
      Ok(()) => Ok(()),
      Err(keyring::Error::NoEntry) => Ok(()),
      Err(err) => Err(err.into()),
    }
  }
}

pub fn generate_random_key() -> Vec<u8> {
  let mut generated = vec![0u8; 32];
  rng().fill_bytes(&mut generated);
  generated
}

#[cfg(test)]
mod tests {
  use crate::KeyringError;
  use objs::AppError;

  #[test]
  fn test_keyring_error_types() {
    let error = KeyringError::KeyringError(keyring::Error::NoEntry);
    assert_eq!(
      error.error_type(),
      objs::ErrorType::InternalServer.to_string()
    );
    assert_eq!(error.code(), "keyring_error-keyring_error");
  }
}
