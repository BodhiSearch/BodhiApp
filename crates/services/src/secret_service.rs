use crate::{asref_impl, CacheService, MokaCacheService};
use aes_gcm::{
  aead::{Aead, KeyInit},
  Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use fs2::FileExt;
use keyring::Entry;
use objs::{impl_error_from, AppError, ErrorType, IoError, SerdeYamlError};
use pbkdf2::pbkdf2_hmac;
use rand::{rngs::OsRng, RngCore};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs::OpenOptions, path::PathBuf};
use std::{path::Path, sync::Arc};

pub const KEY_APP_STATUS: &str = "app_status";
pub const APP_STATUS_READY: &str = "ready";
pub const APP_STATUS_SETUP: &str = "setup";
pub const KEY_APP_AUTHZ: &str = "app_authz";
pub const APP_AUTHZ_TRUE: &str = "true";
pub const APP_AUTHZ_FALSE: &str = "false";
pub const KEY_RESOURCE_TOKEN: &str = "X-Resource-Token";
pub const KEY_APP_REG_INFO: &str = "app_reg_info";
const SALT_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const PBKDF2_ITERATIONS: u32 = 1000;
const SECRET_KEY: &str = "secret_key";

#[derive(Serialize, Deserialize)]
struct EncryptedData {
  salt: String,
  nonce: String,
  data: String,
}

#[derive(Default, Serialize, Deserialize)]
struct SecretsData {
  secrets: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SecretServiceError {
  #[error("keyring_error")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, args_delegate = false)]
  KeyringError(#[from] keyring::Error),
  #[error("key_mismatch")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  KeyMismatch,
  #[error("key_not_found")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  KeyNotFound,
  #[error("decode_error")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, args_delegate = false)]
  DecodeError(#[from] base64::DecodeError),
  #[error(transparent)]
  SerdeYamlError(#[from] SerdeYamlError),
  #[error(transparent)]
  IoError(#[from] IoError),
  #[error("encryption_error")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  EncryptionError(String),
  #[error("decryption_error")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  DecryptionError(String),
  #[error("invalid_format")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
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

pub(crate) type Result<T> = std::result::Result<T, SecretServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait SecretService: Send + Sync + std::fmt::Debug {
  fn set_secret_string(&self, key: &str, value: &str) -> Result<()>;

  fn get_secret_string(&self, key: &str) -> Result<Option<String>>;

  fn delete_secret(&self, key: &str) -> Result<()>;
}

pub fn set_secret<S, T>(slf: S, key: &str, value: T) -> Result<()>
where
  T: serde::Serialize,
  S: AsRef<dyn SecretService>,
{
  let value_str = serde_yaml::to_string(&value)?;
  slf.as_ref().set_secret_string(key, &value_str)
}

pub fn get_secret<S, T>(slf: S, key: &str) -> Result<Option<T>>
where
  T: DeserializeOwned,
  S: AsRef<dyn SecretService>,
{
  match slf.as_ref().get_secret_string(key)? {
    Some(value) => {
      let result = serde_yaml::from_str::<T>(&value)?;
      Ok(Some(result))
    }
    None => Ok(None),
  }
}

asref_impl!(SecretService, KeyringSecretService);

#[derive(Debug)]
pub struct KeyringSecretService {
  secrets_path: PathBuf,
  cache: Arc<dyn CacheService>,
  keyring: Arc<dyn KeyringStore>,
}

impl KeyringSecretService {
  pub fn new(
    service_name: &str,
    secrets_path: &Path,
    encryption_key: Option<String>,
  ) -> Result<Self> {
    let cache = Arc::new(MokaCacheService::default());
    let keyring = Arc::new(SystemKeyringStore::new(service_name.to_string()));
    Self::new_internal(keyring, encryption_key, secrets_path, cache)
  }

  fn new_internal(
    keyring: Arc<dyn KeyringStore>,
    encryption_key: Option<String>,
    secrets_path: &Path,
    cache: Arc<dyn CacheService>,
  ) -> Result<Self> {
    let service = Self {
      secrets_path: secrets_path.to_path_buf(),
      cache,
      keyring,
    };

    match (encryption_key, service.keyring.get_password(SECRET_KEY)?) {
      (Some(key), Some(stored_key)) => {
        // Hash the provided key for uniform distribution
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hashed_key = hasher.finalize();
        let key_str = BASE64.encode(hashed_key);

        if stored_key != key_str {
          return Err(SecretServiceError::KeyMismatch);
        }
      }
      (Some(key), None) => {
        // Hash and store the provided key
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hashed_key = hasher.finalize();
        service.store_encryption_key(hashed_key.to_vec())?;
      }
      (None, None) => {
        // Generate and store a new random key
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        service.store_encryption_key(key)?;
      }
      (None, Some(_)) => {
        // Key exists, nothing to do
      }
    }

    Ok(service)
  }

  fn store_encryption_key(&self, key: Vec<u8>) -> Result<Vec<u8>> {
    let key_str = BASE64.encode(&key);
    self.keyring.set_password(SECRET_KEY, &key_str)?;
    Ok(key)
  }

  fn get_encryption_key(&self) -> Result<Vec<u8>> {
    let key_str = self
      .keyring
      .get_password(SECRET_KEY)?
      .ok_or_else(|| SecretServiceError::KeyNotFound)?;

    Ok(BASE64.decode(&key_str)?)
  }

  fn derive_key(&self, salt: &[u8]) -> Result<Vec<u8>> {
    let encryption_key = self.get_encryption_key()?;
    let mut key = vec![0u8; 32];
    pbkdf2_hmac::<Sha256>(&encryption_key, salt, PBKDF2_ITERATIONS, &mut key);
    Ok(key)
  }

  fn read_secrets(&self, file: &std::fs::File) -> Result<SecretsData> {
    if !self.secrets_path.exists()
      || self.secrets_path.metadata().map(|m| m.len()).unwrap_or(0) == 0
    {
      return Ok(SecretsData::default());
    }

    let mut content = String::new();
    use std::io::Read;
    file.try_clone()?.read_to_string(&mut content)?;

    if content.trim().is_empty() {
      return Ok(SecretsData::default());
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
    let nonce = Nonce::from_slice(&nonce);

    let decrypted_data = cipher
      .decrypt(nonce, encrypted_data.as_ref())
      .map_err(|_| SecretServiceError::DecryptionError("Failed to decrypt data".into()))?;

    let json_str = String::from_utf8(decrypted_data)
      .map_err(|_| SecretServiceError::DecryptionError("Invalid UTF-8 in decrypted data".into()))?;

    serde_yaml::from_str(&json_str).map_err(|e| SecretServiceError::InvalidFormat(e.to_string()))
  }

  fn write_secrets(&self, data: &SecretsData, file: &std::fs::File) -> Result<()> {
    let mut salt = vec![0u8; SALT_SIZE];
    let mut nonce = vec![0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let key = self.derive_key(&salt)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce_slice = Nonce::from_slice(&nonce);

    let json_str =
      serde_yaml::to_string(data).map_err(|e| SecretServiceError::SerdeYamlError(e.into()))?;

    let encrypted_data = cipher
      .encrypt(nonce_slice, json_str.as_bytes())
      .map_err(|_| SecretServiceError::EncryptionError("Failed to encrypt data".into()))?;

    let encrypted = EncryptedData {
      salt: BASE64.encode(salt),
      nonce: BASE64.encode(nonce),
      data: BASE64.encode(encrypted_data),
    };

    serde_yaml::to_writer(file, &encrypted)
      .map_err(|e| SecretServiceError::SerdeYamlError(e.into()))
  }
}

impl SecretService for KeyringSecretService {
  fn set_secret_string(&self, key: &str, value: &str) -> Result<()> {
    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .truncate(true)
      .open(&self.secrets_path)?;

    file.lock_exclusive()?;

    let result = (|| {
      let mut data = self.read_secrets(&file)?;
      data.secrets.insert(key.to_string(), value.to_string());
      self.write_secrets(&data, &file)
    })();

    file.unlock()?;

    // Update cache only if write was successful
    if result.is_ok() {
      self.cache.set(key, value);
    }

    result
  }

  fn get_secret_string(&self, key: &str) -> Result<Option<String>> {
    // First check the cache
    if let Some(cached_value) = self.cache.get(key) {
      return Ok(Some(cached_value));
    }

    let file = OpenOptions::new().read(true).open(&self.secrets_path)?;

    file.lock_shared()?;

    let result = self
      .read_secrets(&file)
      .map(|data| data.secrets.get(key).cloned());

    file.unlock()?;

    // Update cache if value was found
    if let Ok(Some(value)) = &result {
      self.cache.set(key, value);
    }

    result
  }

  fn delete_secret(&self, key: &str) -> Result<()> {
    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .truncate(true)
      .open(&self.secrets_path)?;

    file.lock_exclusive()?;

    let result = (|| {
      let mut data = self.read_secrets(&file)?;
      data.secrets.remove(key);
      self.write_secrets(&data, &file)
    })();

    file.unlock()?;

    // Remove from cache if delete was successful
    if result.is_ok() {
      self.cache.remove(key);
    }

    result
  }
}

// New trait
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait KeyringStore: Send + Sync + std::fmt::Debug {
  fn set_password(&self, key: &str, value: &str) -> Result<()>;
  fn get_password(&self, key: &str) -> Result<Option<String>>;
  fn delete_password(&self, key: &str) -> Result<()>;
}

// Real implementation
#[derive(Debug)]
struct SystemKeyringStore {
  service_name: String,
}

impl SystemKeyringStore {
  fn new(service_name: String) -> Self {
    Self { service_name }
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

#[cfg(test)]
mod tests {
  use crate::{
    get_secret, set_secret, test_utils::KeyringStoreStub, CacheService, KeyringSecretService,
    MokaCacheService, SecretService, SecretServiceError,
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
  #[case(&SecretServiceError::KeyringError(keyring::Error::NoEntry), "No matching entry found in secure storage")]
  #[case(&SecretServiceError::KeyMismatch, "passed encryption key and encryption key stored on platform do not match")]
  #[case(&SecretServiceError::KeyNotFound, "encryption key not found on platform secure storage")]
  #[case(&SecretServiceError::DecodeError(base64::DecodeError::InvalidPadding), "invalid format: Invalid padding")]
  #[case(&SecretServiceError::EncryptionError("invalid format".to_string()), "invalid format")]
  fn test_secret_service_error_messages(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }

  fn secret_service(temp_dir: &TempDir) -> KeyringSecretService {
    KeyringSecretService::new_internal(
      Arc::new(KeyringStoreStub::default()),
      None,
      temp_dir.path().join("secrets.yaml").as_ref(),
      Arc::new(MokaCacheService::default()),
    )
    .unwrap()
  }

  #[rstest]
  fn test_secret_service_with_cache(temp_dir: TempDir) {
    let cache = Arc::new(MokaCacheService::default());
    let service = KeyringSecretService::new_internal(
      Arc::new(KeyringStoreStub::default()),
      None,
      temp_dir.path().join("secrets.yaml").as_ref(),
      cache.clone(),
    )
    .unwrap();

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

  #[rstest]
  fn test_secret_service_with_serialized_object(temp_dir: TempDir) -> anyhow::Result<()> {
    let cache = Arc::new(MokaCacheService::default());
    let service = KeyringSecretService::new_internal(
      Arc::new(KeyringStoreStub::default()),
      None,
      temp_dir.path().join("secrets.yaml").as_ref(),
      cache.clone(),
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
  fn test_secret_service_with_new_encryption_key(temp_dir: TempDir) -> anyhow::Result<()> {
    let secrets_path = temp_dir.path().join("secrets.yaml");
    let keyring = Arc::new(KeyringStoreStub::default());

    let service = KeyringSecretService::new_internal(
      keyring.clone(),
      Some("test-key".to_string()),
      secrets_path.as_ref(),
      Arc::new(MokaCacheService::default()),
    )?;

    // Store a secret
    service.set_secret_string("test-key", "test-value")?;

    // Verify it can be retrieved
    let value = service.get_secret_string("test-key")?;
    assert_eq!(value, Some("test-value".to_string()));

    // Create new service instance with same key
    let service2 = KeyringSecretService::new_internal(
      keyring,
      Some("test-key".to_string()),
      secrets_path.as_ref(),
      Arc::new(MokaCacheService::default()),
    )?;

    // Verify it can read the secret
    let value = service2.get_secret_string("test-key")?;
    assert_eq!(value, Some("test-value".to_string()));

    Ok(())
  }

  #[rstest]
  fn test_secret_service_with_wrong_key(temp_dir: TempDir) -> anyhow::Result<()> {
    let secrets_path = temp_dir.path().join("secrets.yaml");
    let keyring = Arc::new(KeyringStoreStub::default());

    let service = KeyringSecretService::new_internal(
      keyring.clone(),
      Some("key1".to_string()),
      secrets_path.as_ref(),
      Arc::new(MokaCacheService::default()),
    )?;
    drop(service);

    let service = KeyringSecretService::new_internal(
      keyring,
      Some("key2".to_string()),
      secrets_path.as_ref(),
      Arc::new(MokaCacheService::default()),
    );

    assert!(matches!(service, Err(SecretServiceError::KeyMismatch)));

    Ok(())
  }

  #[rstest]
  fn test_secret_service_with_auto_generated_key(temp_dir: TempDir) -> anyhow::Result<()> {
    let secrets_path = temp_dir.path().join("secrets.yaml");
    let keyring = Arc::new(KeyringStoreStub::default());
    let service = KeyringSecretService::new_internal(
      keyring.clone(),
      None,
      secrets_path.as_ref(),
      Arc::new(MokaCacheService::default()),
    )?;

    // Store and verify a secret
    service.set_secret_string("test-key", "test-value")?;
    let value = service.get_secret_string("test-key")?;
    assert_eq!(value, Some("test-value".to_string()));

    // Create new service instance without key
    let service2 = KeyringSecretService::new_internal(
      keyring,
      None,
      secrets_path.as_ref(),
      Arc::new(MokaCacheService::default()),
    )?;

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
