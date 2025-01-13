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

#[derive(Debug, Clone, PartialEq)]
pub struct CachedToken {
  pub token: String,
  pub hash: String,
}

impl CachedToken {
  pub fn new(token: String) -> Self {
    let hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    Self { token, hash }
  }

  pub fn new_with_token_and_hash(token: String, original_token: &str) -> Self {
    let hash = format!("{:x}", Sha256::digest(original_token.as_bytes()));
    Self { token, hash }
  }

  fn to_cache_value(&self) -> String {
    format!("{}:{}", self.token, self.hash)
  }

  fn from_cache_value(value: &str) -> Result<Self, TokenCacheError> {
    let (token, hash) = value.split_once(':').ok_or_else(|| {
      TokenCacheError::MalformedCacheValue("Invalid token:hash format".to_string())
    })?;

    Ok(Self {
      token: token.to_string(),
      hash: hash.to_string(),
    })
  }

  pub fn verify_hash(&self, token: &str) -> bool {
    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    self.hash == token_hash
  }
}

pub trait TokenCache {
  fn get_access_token(&self, jti: &str) -> Result<Option<CachedToken>, TokenCacheError>;
  fn store_access_token(&self, jti: &str, token: CachedToken) -> Result<(), TokenCacheError>;
  fn get_refresh_token(&self, jti: &str) -> Result<Option<String>, TokenCacheError>;
  fn store_refresh_token(&self, jti: &str, token: &str) -> Result<(), TokenCacheError>;
  fn store_token_pair(
    &self,
    jti: &str,
    access_token: CachedToken,
    refresh_token: Option<String>,
  ) -> Result<(), TokenCacheError>;
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
}

impl TokenCache for DefaultTokenCache {
  fn get_access_token(&self, jti: &str) -> Result<Option<CachedToken>, TokenCacheError> {
    let key = self.access_token_key(jti);
    Ok(
      self
        .cache_service
        .get(&key)
        .and_then(|value| CachedToken::from_cache_value(&value).ok()),
    )
  }

  fn store_access_token(&self, jti: &str, token: CachedToken) -> Result<(), TokenCacheError> {
    let key = self.access_token_key(jti);
    self.cache_service.set(&key, &token.to_cache_value());
    Ok(())
  }

  fn get_refresh_token(&self, jti: &str) -> Result<Option<String>, TokenCacheError> {
    let key = self.refresh_token_key(jti);
    Ok(self.cache_service.get(&key))
  }

  fn store_refresh_token(&self, jti: &str, token: &str) -> Result<(), TokenCacheError> {
    let key = self.refresh_token_key(jti);
    self.cache_service.set(&key, token);
    Ok(())
  }

  fn store_token_pair(
    &self,
    jti: &str,
    access_token: CachedToken,
    refresh_token: Option<String>,
  ) -> Result<(), TokenCacheError> {
    self.store_access_token(jti, access_token)?;
    if let Some(refresh) = refresh_token {
      self.store_refresh_token(jti, &refresh)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use services::MokaCacheService;

  fn create_test_cache() -> (DefaultTokenCache, Arc<MokaCacheService>) {
    let cache_service = Arc::new(MokaCacheService::default());
    let token_cache = DefaultTokenCache::new(cache_service.clone());
    (token_cache, cache_service)
  }

  #[test]
  fn test_cached_token_new() {
    let token = "test-token".to_string();
    let cached = CachedToken::new(token.clone());
    assert_eq!(cached.token, token);
    assert!(cached.verify_hash(&token));
  }

  #[test]
  fn test_cached_token_verify_hash() {
    let token = "test-token".to_string();
    let cached = CachedToken::new(token.clone());
    assert!(cached.verify_hash(&token));
    assert!(!cached.verify_hash("different-token"));
  }

  #[test]
  fn test_cached_token_serialization() {
    let token = "test-token".to_string();
    let cached = CachedToken::new(token.clone());
    let cache_value = cached.to_cache_value();
    let deserialized = CachedToken::from_cache_value(&cache_value).unwrap();
    assert_eq!(cached.token, deserialized.token);
    assert_eq!(cached.hash, deserialized.hash);
  }

  #[test]
  fn test_cached_token_invalid_format() {
    let result = CachedToken::from_cache_value("invalid-format");
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      TokenCacheError::MalformedCacheValue(_)
    ));
  }

  #[tokio::test]
  async fn test_store_and_get_access_token() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let token = CachedToken::new("test-token".to_string());

    token_cache.store_access_token(jti, token.clone()).unwrap();

    let retrieved = token_cache.get_access_token(jti).unwrap().unwrap();
    assert_eq!(token.token, retrieved.token);
    assert_eq!(token.hash, retrieved.hash);
  }

  #[tokio::test]
  async fn test_store_and_get_refresh_token() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let token = "refresh-token";

    token_cache.store_refresh_token(jti, token).unwrap();

    let retrieved = token_cache.get_refresh_token(jti).unwrap().unwrap();
    assert_eq!(token, retrieved);
  }

  #[tokio::test]
  async fn test_store_token_pair() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let access_token = CachedToken::new("access-token".to_string());
    let refresh_token = Some("refresh-token".to_string());

    token_cache
      .store_token_pair(jti, access_token.clone(), refresh_token.clone())
      .unwrap();

    let retrieved_access = token_cache.get_access_token(jti).unwrap().unwrap();
    assert_eq!(access_token.token, retrieved_access.token);

    let retrieved_refresh = token_cache.get_refresh_token(jti).unwrap().unwrap();
    assert_eq!(refresh_token.unwrap(), retrieved_refresh);
  }

  #[tokio::test]
  async fn test_nonexistent_tokens() {
    let (token_cache, _) = create_test_cache();
    let jti = "nonexistent-jti";

    assert!(token_cache.get_access_token(jti).unwrap().is_none());
    assert!(token_cache.get_refresh_token(jti).unwrap().is_none());
  }
}
