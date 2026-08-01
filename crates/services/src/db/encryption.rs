use aes_gcm::{
  aead::{Aead, KeyInit},
  Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use errmeta::{AppError, ErrorType};
use hkdf::Hkdf;
use pbkdf2::pbkdf2_hmac;
use rand::{rng, RngCore};
use sha2::Sha256;

const SALT_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

/// Legacy scheme: PBKDF2 was run per row against the row salt. Retained only to read
/// `tenants.encrypted_client_secret` rows written before the v2 scheme.
const PBKDF2_ITERATIONS_V1: u32 = 1000;

/// OWASP-recommended work factor. Paid once at startup deriving the KEK, never per row.
const PBKDF2_ITERATIONS: u32 = 600_000;

/// Domain separator for the KEK. With a single global master key a salt is a domain
/// separator rather than a security parameter, and it must be deterministic because the
/// KEK is needed before the database is open.
const KEK_SALT: &[u8] = b"bodhiapp:kek:v2";

/// Domain separator for per-row keys derived from the KEK.
const ROW_INFO: &[u8] = b"bodhiapp:row:v2";

/// Marks a ciphertext as v2. Base64's alphabet never contains `:`, so an unprefixed value
/// is unambiguously a legacy row — detection never requires attempting a decrypt.
const V2_PREFIX: &str = "v2:";

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum EncryptionError {
  #[error("Encryption failed.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  EncryptionFailed,
  #[error("Decryption failed.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DecryptionFailed,
  #[error("Invalid encryption format: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidFormat(String),
  #[error("This record was encrypted with an unsupported legacy scheme and must be recreated.")]
  #[error_meta(error_type = ErrorType::UnprocessableEntity)]
  LegacyCiphertextUnsupported,
}

pub type Result<T> = std::result::Result<T, EncryptionError>;

/// Master key material plus the key-encryption key derived from it.
///
/// `master` exists solely for [`decrypt_api_key_legacy`]; every other operation uses `kek`.
#[derive(Clone)]
pub struct EncryptionKeys {
  master: Vec<u8>,
  kek: [u8; 32],
}

impl std::fmt::Debug for EncryptionKeys {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("EncryptionKeys(<redacted>)")
  }
}

impl EncryptionKeys {
  /// Stretches the master key into a KEK. Costs ~70ms — call once at startup, never per request.
  pub fn derive(master: Vec<u8>) -> Self {
    let mut kek = [0u8; 32];
    pbkdf2_hmac::<Sha256>(&master, KEK_SALT, PBKDF2_ITERATIONS, &mut kek);
    Self { master, kek }
  }

  /// Skips the 600k-iteration stretch. Tests construct a `DbService` per case, so deriving
  /// for real would add ~70ms to every one of them.
  #[cfg(any(test, feature = "test-utils"))]
  pub fn for_test(master: Vec<u8>) -> Self {
    let mut kek = [0u8; 32];
    pbkdf2_hmac::<Sha256>(&master, KEK_SALT, 1, &mut kek);
    Self { master, kek }
  }

  pub fn master(&self) -> &[u8] {
    &self.master
  }
}

fn generate_salt() -> [u8; SALT_SIZE] {
  let mut salt = [0u8; SALT_SIZE];
  rng().fill_bytes(&mut salt);
  salt
}

fn generate_nonce() -> [u8; NONCE_SIZE] {
  let mut nonce = [0u8; NONCE_SIZE];
  rng().fill_bytes(&mut nonce);
  nonce
}

/// Per-row key from the KEK. HKDF is ~1µs, so the per-row salt keeps key separation
/// without the per-row stretch the legacy scheme paid for.
fn derive_row_key(kek: &[u8; 32], salt: &[u8]) -> Result<[u8; 32]> {
  let mut key = [0u8; 32];
  Hkdf::<Sha256>::new(Some(salt), kek)
    .expand(ROW_INFO, &mut key)
    .map_err(|_| EncryptionError::EncryptionFailed)?;
  Ok(key)
}

fn derive_row_key_v1(master: &[u8], salt: &[u8]) -> [u8; 32] {
  let mut key = [0u8; 32];
  pbkdf2_hmac::<Sha256>(master, salt, PBKDF2_ITERATIONS_V1, &mut key);
  key
}

fn decrypt_with(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8]) -> Result<String> {
  let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
  let decrypted = cipher
    .decrypt(Nonce::from_slice(nonce), ciphertext)
    .map_err(|_| EncryptionError::DecryptionFailed)?;
  String::from_utf8(decrypted)
    .map_err(|_| EncryptionError::InvalidFormat("Invalid UTF-8 in decrypted data".into()))
}

fn decode_parts(encrypted: &str, salt: &str, nonce: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
  let encrypted_data = BASE64
    .decode(encrypted)
    .map_err(|_| EncryptionError::InvalidFormat("Invalid encrypted data format".into()))?;
  let salt_bytes = BASE64
    .decode(salt)
    .map_err(|_| EncryptionError::InvalidFormat("Invalid salt format".into()))?;
  let nonce_bytes = BASE64
    .decode(nonce)
    .map_err(|_| EncryptionError::InvalidFormat("Invalid nonce format".into()))?;
  Ok((encrypted_data, salt_bytes, nonce_bytes))
}

/// Always emits the v2 format.
pub fn encrypt_api_key(keys: &EncryptionKeys, api_key: &str) -> Result<(String, String, String)> {
  let salt = generate_salt();
  let nonce = generate_nonce();

  let key = derive_row_key(&keys.kek, &salt)?;
  let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

  let encrypted_data = cipher
    .encrypt(Nonce::from_slice(&nonce), api_key.as_bytes())
    .map_err(|_| EncryptionError::EncryptionFailed)?;

  Ok((
    format!("{}{}", V2_PREFIX, BASE64.encode(encrypted_data)),
    BASE64.encode(salt),
    BASE64.encode(nonce),
  ))
}

/// Rejects legacy rows with [`EncryptionError::LegacyCiphertextUnsupported`] so callers can
/// tell "recreate this resource" apart from "the encryption key is wrong".
pub fn decrypt_api_key(
  keys: &EncryptionKeys,
  encrypted: &str,
  salt: &str,
  nonce: &str,
) -> Result<String> {
  let Some(body) = encrypted.strip_prefix(V2_PREFIX) else {
    return Err(EncryptionError::LegacyCiphertextUnsupported);
  };
  let (encrypted_data, salt_bytes, nonce_bytes) = decode_parts(body, salt, nonce)?;
  let key = derive_row_key(&keys.kek, &salt_bytes)?;
  decrypt_with(&key, &encrypted_data, &nonce_bytes)
}

/// Reads a pre-v2 row. Used only by the startup pass that migrates `tenants` — every other
/// table surfaces [`EncryptionError::LegacyCiphertextUnsupported`] instead.
pub fn decrypt_api_key_legacy(
  master: &[u8],
  encrypted: &str,
  salt: &str,
  nonce: &str,
) -> Result<String> {
  let (encrypted_data, salt_bytes, nonce_bytes) = decode_parts(encrypted, salt, nonce)?;
  let key = derive_row_key_v1(master, &salt_bytes);
  decrypt_with(&key, &encrypted_data, &nonce_bytes)
}

/// True when the stored ciphertext predates the v2 scheme.
pub fn is_legacy_ciphertext(encrypted: &str) -> bool {
  !encrypted.starts_with(V2_PREFIX)
}

/// Writes a row in the pre-v2 format so tests can exercise the upgrade path. Production code
/// never emits v1.
#[cfg(any(test, feature = "test-utils"))]
pub fn encrypt_api_key_legacy(master: &[u8], api_key: &str) -> Result<(String, String, String)> {
  let salt = generate_salt();
  let nonce = generate_nonce();
  let key = derive_row_key_v1(master, &salt);
  let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
  let encrypted = cipher
    .encrypt(Nonce::from_slice(&nonce), api_key.as_bytes())
    .map_err(|_| EncryptionError::EncryptionFailed)?;
  Ok((
    BASE64.encode(encrypted),
    BASE64.encode(salt),
    BASE64.encode(nonce),
  ))
}

#[cfg(test)]
mod tests {
  use crate::db::encryption::{
    decrypt_api_key, decrypt_api_key_legacy, encrypt_api_key, encrypt_api_key_legacy,
    is_legacy_ciphertext, EncryptionError, EncryptionKeys,
  };
  use errmeta::AppError;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  const MASTER: &[u8] = b"test_master_key_12345678901234567890";

  fn keys() -> EncryptionKeys {
    EncryptionKeys::for_test(MASTER.to_vec())
  }

  pub fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 4 {
      "*".repeat(api_key.len())
    } else {
      let prefix = &api_key[..4];
      let suffix = "*".repeat(api_key.len() - 4);
      format!("{}{}", prefix, suffix)
    }
  }

  #[rstest]
  fn test_encryption_decryption_round_trip() -> anyhow::Result<()> {
    let keys = keys();
    let api_key = "sk-1234567890abcdef";

    let (encrypted, salt, nonce) = encrypt_api_key(&keys, api_key)?;
    let decrypted = decrypt_api_key(&keys, &encrypted, &salt, &nonce)?;

    assert_eq!(api_key, decrypted);
    Ok(())
  }

  #[rstest]
  fn test_encrypt_emits_v2_prefix() -> anyhow::Result<()> {
    let (encrypted, _, _) = encrypt_api_key(&keys(), "sk-abc")?;
    assert!(encrypted.starts_with("v2:"));
    assert!(!is_legacy_ciphertext(&encrypted));
    Ok(())
  }

  #[rstest]
  fn test_encryption_with_different_salts() -> anyhow::Result<()> {
    let keys = keys();
    let api_key = "sk-abcdef123456";

    let (encrypted1, salt1, nonce1) = encrypt_api_key(&keys, api_key)?;
    let (encrypted2, salt2, nonce2) = encrypt_api_key(&keys, api_key)?;

    assert_ne!(encrypted1, encrypted2);
    assert_ne!(salt1, salt2);
    assert_ne!(nonce1, nonce2);

    assert_eq!(
      api_key,
      decrypt_api_key(&keys, &encrypted1, &salt1, &nonce1)?
    );
    assert_eq!(
      api_key,
      decrypt_api_key(&keys, &encrypted2, &salt2, &nonce2)?
    );
    Ok(())
  }

  #[rstest]
  fn test_decryption_with_wrong_key_fails() -> anyhow::Result<()> {
    let keys1 = keys();
    let keys2 = EncryptionKeys::for_test(b"different_key_1234567890123456789012".to_vec());

    let (encrypted, salt, nonce) = encrypt_api_key(&keys1, "sk-test12345")?;
    let err = decrypt_api_key(&keys2, &encrypted, &salt, &nonce).unwrap_err();

    // A wrong key must read as a decrypt failure, never as a legacy row.
    assert_eq!("encryption_error-decryption_failed", err.code());
    Ok(())
  }

  #[rstest]
  fn test_legacy_ciphertext_is_rejected_distinctly() -> anyhow::Result<()> {
    let (encrypted, salt, nonce) = encrypt_api_key_legacy(MASTER, "sk-legacy-secret")?;
    assert!(is_legacy_ciphertext(&encrypted));

    let err = decrypt_api_key(&keys(), &encrypted, &salt, &nonce).unwrap_err();
    assert_eq!("encryption_error-legacy_ciphertext_unsupported", err.code());
    assert_eq!("unprocessable_entity_error", err.error_type());
    assert!(matches!(err, EncryptionError::LegacyCiphertextUnsupported));
    Ok(())
  }

  #[rstest]
  fn test_legacy_decrypt_reads_v1_row() -> anyhow::Result<()> {
    let (encrypted, salt, nonce) = encrypt_api_key_legacy(MASTER, "sk-legacy-secret")?;
    let decrypted = decrypt_api_key_legacy(MASTER, &encrypted, &salt, &nonce)?;
    assert_eq!("sk-legacy-secret", decrypted);
    Ok(())
  }

  #[rstest]
  fn test_legacy_decrypt_with_wrong_master_fails() -> anyhow::Result<()> {
    let (encrypted, salt, nonce) = encrypt_api_key_legacy(MASTER, "sk-legacy-secret")?;
    let err =
      decrypt_api_key_legacy(b"some-other-master-key", &encrypted, &salt, &nonce).unwrap_err();
    assert_eq!("encryption_error-decryption_failed", err.code());
    Ok(())
  }

  #[rstest]
  #[case("sk-1234567890", "sk-1*********")] // 13 chars -> 4 shown + 9 masked
  #[case("abc", "***")] // 3 chars -> all masked
  #[case("", "")] // 0 chars -> empty
  #[case("a", "*")] // 1 char -> all masked
  #[case("abcd", "****")] // 4 chars -> all masked
  #[case("abcde", "abcd*")] // 5 chars -> 4 shown + 1 masked
  fn test_mask_api_key(#[case] input: &str, #[case] expected: &str) {
    let masked = mask_api_key(input);
    assert_eq!(expected, masked);
  }
}
