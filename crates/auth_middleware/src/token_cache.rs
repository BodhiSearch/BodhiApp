use objs::{AppError, ErrorType};
use services::CacheService;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const ACCESS_TOKEN_PREFIX: &str = "exchange-access-token-";
const REFRESH_TOKEN_PREFIX: &str = "exchange-refresh-token-";

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum TokenCacheError {
  #[error("malformed_cache_value")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MalformedCacheValue(String),
}

pub trait TokenCache {
  fn get_access_token(&self, jti: &str) -> Result<Option<String>, TokenCacheError>;
  fn is_token_in_cache(&self, jti: &str, token: &str) -> Result<bool, TokenCacheError>;
  fn store_access_token(&self, jti: &str, token: &str);
  fn get_refresh_token(&self, jti: &str) -> Result<Option<String>, TokenCacheError>;
  fn store_refresh_token(&self, jti: &str, token: &str);
  fn store_token_pair(&self, jti: &str, access_token: &str, refresh_token: Option<String>);
}

pub struct DefaultTokenCache {
  cache_service: Arc<dyn CacheService>,
}

impl DefaultTokenCache {
  pub fn new(cache_service: Arc<dyn CacheService>) -> Self {
    Self { cache_service }
  }

  fn access_token_key(&self, jti: &str) -> String {
    format!("{}{}", ACCESS_TOKEN_PREFIX, jti)
  }

  fn refresh_token_key(&self, jti: &str) -> String {
    format!("{}{}", REFRESH_TOKEN_PREFIX, jti)
  }

  fn compute_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
  }

  fn to_cache_value(token: &str) -> String {
    format!("{}:{}", token, Self::compute_hash(token))
  }

  fn from_cache_value(value: &str) -> Result<String, TokenCacheError> {
    let (token, hash) = value.split_once(':').ok_or_else(|| {
      TokenCacheError::MalformedCacheValue("Invalid token:hash format".to_string())
    })?;

    let computed_hash = Self::compute_hash(token);
    if computed_hash != hash {
      return Err(TokenCacheError::MalformedCacheValue(
        "Token hash mismatch".to_string(),
      ));
    }

    Ok(token.to_string())
  }
}

impl TokenCache for DefaultTokenCache {
  fn get_access_token(&self, jti: &str) -> Result<Option<String>, TokenCacheError> {
    let key = self.access_token_key(jti);
    if let Some(value) = self.cache_service.get(&key) {
      Ok(Some(Self::from_cache_value(&value)?))
    } else {
      Ok(None)
    }
  }

  fn is_token_in_cache(&self, jti: &str, token: &str) -> Result<bool, TokenCacheError> {
    let key = self.access_token_key(jti);
    if let Some(value) = self.cache_service.get(&key) {
      let (_, hash) = value.split_once(':').ok_or_else(|| {
        TokenCacheError::MalformedCacheValue("Invalid token:hash format".to_string())
      })?;
      Ok(Self::compute_hash(token) == hash)
    } else {
      Ok(false)
    }
  }

  fn store_access_token(&self, jti: &str, token: &str) {
    let key = self.access_token_key(jti);
    let value = Self::to_cache_value(token);
    self.cache_service.set(&key, &value);
  }

  fn get_refresh_token(&self, jti: &str) -> Result<Option<String>, TokenCacheError> {
    let key = self.refresh_token_key(jti);
    Ok(self.cache_service.get(&key))
  }

  fn store_refresh_token(&self, jti: &str, token: &str) {
    let key = self.refresh_token_key(jti);
    self.cache_service.set(&key, token);
  }

  fn store_token_pair(&self, jti: &str, access_token: &str, refresh_token: Option<String>) {
    self.store_access_token(jti, access_token);
    if let Some(refresh_token) = refresh_token {
      self.store_refresh_token(jti, &refresh_token);
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use services::MokaCacheService;

  use crate::{TokenCache, TokenCacheError};

  use super::DefaultTokenCache;

  fn create_test_cache() -> (DefaultTokenCache, Arc<MokaCacheService>) {
    let cache_service = Arc::new(MokaCacheService::default());
    let token_cache = DefaultTokenCache::new(cache_service.clone());
    (token_cache, cache_service)
  }

  #[test]
  fn test_token_hash_verification() {
    let token = "test-token";
    let cache_value = DefaultTokenCache::to_cache_value(token);
    let retrieved = DefaultTokenCache::from_cache_value(&cache_value).unwrap();
    assert_eq!(retrieved, token);
  }

  #[test]
  fn test_token_invalid_format() {
    let result = DefaultTokenCache::from_cache_value("invalid_format");
    assert!(matches!(
      result,
      Err(TokenCacheError::MalformedCacheValue(_))
    ));

    let result = DefaultTokenCache::from_cache_value("token:invalid_hash");
    assert!(matches!(
      result,
      Err(TokenCacheError::MalformedCacheValue(_))
    ));
  }

  #[tokio::test]
  async fn test_store_and_get_access_token() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let token = "test-token";

    token_cache.store_access_token(jti, token);
    let retrieved = token_cache.get_access_token(jti).unwrap().unwrap();
    assert_eq!(retrieved, token);
  }

  #[tokio::test]
  async fn test_store_and_get_refresh_token() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let token = "refresh-token";

    token_cache.store_refresh_token(jti, token);
    let retrieved = token_cache.get_refresh_token(jti).unwrap().unwrap();
    assert_eq!(retrieved, token);
  }

  #[tokio::test]
  async fn test_store_token_pair() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let access_token = "access-token";
    let refresh_token = Some("refresh-token".to_string());

    token_cache.store_token_pair(jti, access_token, refresh_token.clone());

    let retrieved_access = token_cache.get_access_token(jti).unwrap().unwrap();
    assert_eq!(retrieved_access, access_token);

    let retrieved_refresh = token_cache.get_refresh_token(jti).unwrap();
    assert_eq!(retrieved_refresh, refresh_token);
  }

  #[tokio::test]
  async fn test_nonexistent_tokens() {
    let (token_cache, _) = create_test_cache();
    let jti = "nonexistent";

    assert!(token_cache.get_access_token(jti).unwrap().is_none());
    assert!(token_cache.get_refresh_token(jti).unwrap().is_none());
  }

  #[tokio::test]
  async fn test_is_token_in_cache_success() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let token = "test-token";

    // Store token
    token_cache.store_access_token(jti, token);

    // Verify with correct token
    let is_in_cache = token_cache.is_token_in_cache(jti, token).unwrap();
    assert!(is_in_cache);
  }

  #[tokio::test]
  async fn test_is_token_in_cache_fail() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let token = "test-token";

    // Store token
    token_cache.store_access_token(jti, token);

    // Verify with incorrect token
    let is_in_cache = token_cache.is_token_in_cache(jti, "wrong-token").unwrap();
    assert!(!is_in_cache);
  }
}
