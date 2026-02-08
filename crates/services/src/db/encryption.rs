use aes_gcm::{
  aead::{Aead, KeyInit},
  Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use objs::{AppError, ErrorType};
use pbkdf2::pbkdf2_hmac;
use rand::{rng, RngCore};
use sha2::Sha256;

const SALT_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const PBKDF2_ITERATIONS: u32 = 1000;

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
}

pub type Result<T> = std::result::Result<T, EncryptionError>;

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

fn derive_key(master_key: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
  let mut key = [0u8; 32];
  pbkdf2_hmac::<Sha256>(master_key, salt, PBKDF2_ITERATIONS, &mut key);
  Ok(key)
}

pub fn encrypt_api_key(master_key: &[u8], api_key: &str) -> Result<(String, String, String)> {
  let salt = generate_salt();
  let nonce = generate_nonce();

  let key = derive_key(master_key, &salt)?;
  let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
  let nonce_slice = Nonce::from_slice(&nonce);

  let encrypted_data = cipher
    .encrypt(nonce_slice, api_key.as_bytes())
    .map_err(|_| EncryptionError::EncryptionFailed)?;

  let encrypted_b64 = BASE64.encode(encrypted_data);
  let salt_b64 = BASE64.encode(salt);
  let nonce_b64 = BASE64.encode(nonce);

  Ok((encrypted_b64, salt_b64, nonce_b64))
}

pub fn decrypt_api_key(
  master_key: &[u8],
  encrypted: &str,
  salt: &str,
  nonce: &str,
) -> Result<String> {
  let encrypted_data = BASE64
    .decode(encrypted)
    .map_err(|_| EncryptionError::InvalidFormat("Invalid encrypted data format".into()))?;
  let salt_bytes = BASE64
    .decode(salt)
    .map_err(|_| EncryptionError::InvalidFormat("Invalid salt format".into()))?;
  let nonce_bytes = BASE64
    .decode(nonce)
    .map_err(|_| EncryptionError::InvalidFormat("Invalid nonce format".into()))?;

  let key = derive_key(master_key, &salt_bytes)?;
  let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
  let nonce_slice = Nonce::from_slice(&nonce_bytes);

  let decrypted_data = cipher
    .decrypt(nonce_slice, encrypted_data.as_ref())
    .map_err(|_| EncryptionError::DecryptionFailed)?;

  String::from_utf8(decrypted_data)
    .map_err(|_| EncryptionError::InvalidFormat("Invalid UTF-8 in decrypted data".into()))
}

#[cfg(test)]
mod tests {
  use crate::db::encryption::{decrypt_api_key, encrypt_api_key};
  use rstest::rstest;

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
    let master_key = b"test_master_key_12345678901234567890";
    let api_key = "sk-1234567890abcdef";

    let (encrypted, salt, nonce) = encrypt_api_key(master_key, api_key)?;
    let decrypted = decrypt_api_key(master_key, &encrypted, &salt, &nonce)?;

    assert_eq!(api_key, decrypted);
    Ok(())
  }

  #[rstest]
  fn test_encryption_with_different_salts() -> anyhow::Result<()> {
    let master_key = b"test_master_key_12345678901234567890";
    let api_key = "sk-abcdef123456";

    let (encrypted1, salt1, nonce1) = encrypt_api_key(master_key, api_key)?;
    let (encrypted2, salt2, nonce2) = encrypt_api_key(master_key, api_key)?;

    assert_ne!(encrypted1, encrypted2);
    assert_ne!(salt1, salt2);
    assert_ne!(nonce1, nonce2);

    let decrypted1 = decrypt_api_key(master_key, &encrypted1, &salt1, &nonce1)?;
    let decrypted2 = decrypt_api_key(master_key, &encrypted2, &salt2, &nonce2)?;

    assert_eq!(api_key, decrypted1);
    assert_eq!(api_key, decrypted2);
    Ok(())
  }

  #[rstest]
  fn test_decryption_with_wrong_key_fails() -> anyhow::Result<()> {
    let master_key1 = b"test_master_key_12345678901234567890";
    let master_key2 = b"different_key_1234567890123456789012";
    let api_key = "sk-test12345";

    let (encrypted, salt, nonce) = encrypt_api_key(master_key1, api_key)?;
    let result = decrypt_api_key(master_key2, &encrypted, &salt, &nonce);

    assert!(result.is_err());
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
