use crate::asref_impl;
use aes_gcm::{
  aead::{Aead, KeyInit},
  Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use objs::{impl_error_from, AppError, ErrorType, IoError, SerdeYamlError};
use pbkdf2::pbkdf2_hmac;
use rand::{rng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::Path;
use std::{collections::HashMap, fs::OpenOptions, path::PathBuf};

const SALT_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const PBKDF2_ITERATIONS: u32 = 1000;

#[derive(Serialize, Deserialize)]
struct EncryptedData {
  salt: String,
  nonce: String,
  data: String,
}

struct DecryptedData {
  salt: Vec<u8>,
  nonce: Vec<u8>,
  secrets: HashMap<String, String>,
}

impl Default for DecryptedData {
  fn default() -> Self {
    let mut salt = vec![0u8; SALT_SIZE];
    let mut nonce = vec![0u8; NONCE_SIZE];
    rng().fill_bytes(&mut salt);
    rng().fill_bytes(&mut nonce);
    Self {
      salt,
      nonce,
      secrets: HashMap::new(),
    }
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SecretServiceError {
  #[error("key_mismatch")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  KeyMismatch,
  #[error("key_not_found")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  KeyNotFound,
  #[error(transparent)]
  SerdeYamlError(#[from] SerdeYamlError),
  #[error(transparent)]
  IoError(#[from] IoError),
  #[error("encryption_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  EncryptionError(String),
  #[error("decryption_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DecryptionError(String),
  #[error("invalid_format")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidFormat(String),
}

impl_error_from!(
  ::serde_yaml::Error,
  SecretServiceError::SerdeYamlError,
  ::objs::SerdeYamlError
);

impl_error_from!(
  ::std::io::Error,
  SecretServiceError::IoError,
  ::objs::IoError
);

type Result<T> = std::result::Result<T, SecretServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait SecretService: Send + Sync + std::fmt::Debug {
  fn set_secret_string(&self, key: &str, value: &str) -> Result<()>;

  fn get_secret_string(&self, key: &str) -> Result<Option<String>>;

  fn delete_secret(&self, key: &str) -> Result<()>;

  #[cfg(debug_assertions)]
  fn dump(&self) -> Result<String>;
}

asref_impl!(SecretService, DefaultSecretService);

#[derive(Debug)]
pub struct DefaultSecretService {
  encryption_key: Vec<u8>,
  secrets_path: PathBuf,
}

impl DefaultSecretService {
  pub fn new(encryption_key: impl AsRef<[u8]>, secrets_path: &Path) -> Result<Self> {
    let service = Self {
      secrets_path: secrets_path.to_path_buf(),
      encryption_key: encryption_key.as_ref().to_vec(),
    };
    service.validate()?;
    Ok(service)
  }

  fn derive_key(&self, salt: &[u8]) -> Result<Vec<u8>> {
    let mut key = vec![0u8; 32];
    pbkdf2_hmac::<Sha256>(&self.encryption_key, salt, PBKDF2_ITERATIONS, &mut key);
    Ok(key)
  }

  fn read_secrets(&self) -> Result<DecryptedData> {
    if !self.secrets_path.exists() {
      return Ok(DecryptedData::default());
    }

    let mut file = OpenOptions::new().read(true).open(&self.secrets_path)?;
    fs2::FileExt::lock_shared(&file)?;

    let result = (|| {
      let mut content = String::new();
      use std::io::Read;
      file.read_to_string(&mut content)?;

      if content.trim().is_empty() {
        return Ok(DecryptedData::default());
      }

      let encrypted: EncryptedData = serde_yaml::from_str(&content)?;

      let salt = BASE64
        .decode(&encrypted.salt)
        .map_err(|_| SecretServiceError::InvalidFormat("Invalid salt format".into()))?;
      let nonce = BASE64
        .decode(&encrypted.nonce)
        .map_err(|_| SecretServiceError::InvalidFormat("Invalid nonce format".into()))?;
      let encrypted_data = BASE64
        .decode(&encrypted.data)
        .map_err(|_| SecretServiceError::InvalidFormat("Invalid data format".into()))?;

      let key = self.derive_key(&salt)?;
      let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
      let nonce_slice = Nonce::from_slice(&nonce);

      let decrypted_data = cipher
        .decrypt(nonce_slice, encrypted_data.as_ref())
        .map_err(|_| SecretServiceError::DecryptionError("Failed to decrypt data".into()))?;

      let yaml_str = String::from_utf8(decrypted_data).map_err(|_| {
        SecretServiceError::DecryptionError("Invalid UTF-8 in decrypted data".into())
      })?;

      let secrets: HashMap<String, String> = serde_yaml::from_str(&yaml_str)
        .map_err(|e| SecretServiceError::InvalidFormat(e.to_string()))?;

      Ok(DecryptedData {
        salt,
        nonce,
        secrets,
      })
    })();

    fs2::FileExt::unlock(&file)?;
    result
  }

  fn write_secrets(&self, data: &DecryptedData) -> Result<()> {
    let file = OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(&self.secrets_path)?;

    fs2::FileExt::lock_exclusive(&file)?;

    let result = (|| {
      let key = self.derive_key(&data.salt)?;
      let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
      let nonce_slice = Nonce::from_slice(&data.nonce);

      let yaml_str = serde_yaml::to_string(&data.secrets)
        .map_err(|e| SecretServiceError::SerdeYamlError(e.into()))?;

      let encrypted_data = cipher
        .encrypt(nonce_slice, yaml_str.as_bytes())
        .map_err(|_| SecretServiceError::EncryptionError("Failed to encrypt data".into()))?;

      let encrypted = EncryptedData {
        salt: BASE64.encode(&data.salt),
        nonce: BASE64.encode(&data.nonce),
        data: BASE64.encode(encrypted_data),
      };

      serde_yaml::to_writer(&file, &encrypted)
        .map_err(|e| SecretServiceError::SerdeYamlError(e.into()))
    })();

    fs2::FileExt::unlock(&file)?;
    result
  }

  fn validate(&self) -> Result<()> {
    if !self.secrets_path.exists() {
      return Ok(());
    }
    self.read_secrets()?;
    Ok(())
  }
}

impl SecretService for DefaultSecretService {
  fn set_secret_string(&self, key: &str, value: &str) -> Result<()> {
    let mut data = self.read_secrets()?;
    data.secrets.insert(key.to_string(), value.to_string());
    self.write_secrets(&data)
  }

  fn get_secret_string(&self, key: &str) -> Result<Option<String>> {
    let data = self.read_secrets()?;
    Ok(data.secrets.get(key).cloned())
  }

  fn delete_secret(&self, key: &str) -> Result<()> {
    let mut data = self.read_secrets()?;
    data.secrets.remove(key);
    self.write_secrets(&data)
  }

  #[cfg(debug_assertions)]
  fn dump(&self) -> Result<String> {
    let data = self.read_secrets()?;
    Ok(serde_yaml::to_string(&data.secrets)?)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    generate_random_key, get_secret, set_secret, DefaultSecretService, SecretService,
    SecretServiceError,
  };
  use objs::{
    test_utils::{assert_error_message, setup_l10n, temp_dir},
    AppError, FluentLocalizationService,
  };
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use std::sync::Arc;
  use tempfile::TempDir;

  #[rstest]
  #[case(&SecretServiceError::KeyMismatch, "passed encryption key and encryption key stored on platform do not match")]
  #[case(&SecretServiceError::KeyNotFound, "encryption key not found on platform secure storage")]
  #[case(&SecretServiceError::EncryptionError("invalid format".to_string()), "invalid format")]
  fn test_secret_service_error_messages(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }

  fn secret_service(temp_dir: &TempDir) -> DefaultSecretService {
    DefaultSecretService::new(
      generate_random_key(),
      temp_dir.path().join("secrets.yaml").as_ref(),
    )
    .unwrap()
  }

  #[rstest]
  fn test_secret_service_with_serialized_object(temp_dir: TempDir) -> anyhow::Result<()> {
    let service = DefaultSecretService::new(
      generate_random_key(),
      temp_dir.path().join("secrets.yaml").as_ref(),
    )
    .unwrap();

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

    set_secret(&service, "test_object", &test_object)?;
    let retrieved_object: Option<TestObject> = get_secret(&service, "test_object").unwrap();
    assert_eq!(retrieved_object, Some(test_object));

    service.delete_secret("test_object").unwrap();
    let deleted_object: Option<TestObject> = get_secret(&service, "test_object").unwrap();
    assert_eq!(deleted_object, None);
    Ok(())
  }

  #[rstest]
  fn test_secret_service_with_wrong_key(temp_dir: TempDir) -> anyhow::Result<()> {
    let secrets_path = temp_dir.path().join("secrets.yaml");
    let service = DefaultSecretService::new(generate_random_key(), secrets_path.as_ref())?;
    service.set_secret_string("test-key", "test-value")?;
    assert_eq!(
      service.get_secret_string("test-key")?,
      Some("test-value".to_string())
    );
    drop(service);

    let service = DefaultSecretService::new(generate_random_key(), secrets_path.as_ref());

    assert!(matches!(
      service.unwrap_err(),
      SecretServiceError::DecryptionError(_)
    ));

    Ok(())
  }

  #[rstest]
  fn test_secret_service_with_same_key(temp_dir: TempDir) -> anyhow::Result<()> {
    let secrets_path = temp_dir.path().join("secrets.yaml");
    let encryption_key = generate_random_key();
    let service = DefaultSecretService::new(encryption_key.clone(), secrets_path.as_ref())?;

    // Store and verify a secret
    service.set_secret_string("test-key", "test-value")?;
    assert_eq!(
      service.get_secret_string("test-key")?,
      Some("test-value".to_string())
    );

    drop(service);

    // Create new service instance without key
    let service2 = DefaultSecretService::new(encryption_key, secrets_path.as_ref())?;

    // Verify it can read the secret
    let value = service2.get_secret_string("test-key")?;
    assert_eq!(value, Some("test-value".to_string()));

    Ok(())
  }

  #[rstest]
  fn test_secret_service_delete(temp_dir: TempDir) -> anyhow::Result<()> {
    // Store a secret
    let service = secret_service(&temp_dir);
    service.set_secret_string("test-key", "test-value")?;

    // Verify it exists
    assert_eq!(
      service.get_secret_string("test-key")?,
      Some("test-value".to_string())
    );

    // Delete it
    service.delete_secret("test-key")?;

    // Verify it's gone
    assert_eq!(service.get_secret_string("test-key")?, None);

    Ok(())
  }
}
