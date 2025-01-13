use objs::{impl_error_from, AppError, ErrorType};
use services::CacheService;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

const ACCESS_TOKEN_PREFIX: &str = "exchange-access-token-";
const REFRESH_TOKEN_PREFIX: &str = "exchange-refresh-token-";
const TOKEN_SEPARATOR: &str = ":";
const TIME_SEPARATOR: &str = "@";

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
  pub created_at: OffsetDateTime,
  pub expires_at: OffsetDateTime,
}

impl CachedToken {
  pub fn new(token: String, expires_in: Duration) -> Self {
    let hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    let now = OffsetDateTime::now_utc();
    Self {
      token,
      hash,
      created_at: now,
      expires_at: now + expires_in,
    }
  }

  fn to_cache_value(&self) -> String {
    format!(
      "{}{}{}{}{}",
      self.token,
      TOKEN_SEPARATOR,
      self.hash,
      TIME_SEPARATOR,
      self.expires_at.unix_timestamp()
    )
  }

  fn from_cache_value(value: &str) -> Result<Self, TokenCacheError> {
    let parts: Vec<&str> = value.split(TIME_SEPARATOR).collect();
    if parts.len() != 2 {
      return Err(TokenCacheError::MalformedCacheValue(
        "Missing timestamp separator".to_string(),
      ));
    }

    let (token_part, timestamp_str) = (parts[0], parts[1]);
    let (token, hash) = token_part.split_once(TOKEN_SEPARATOR).ok_or_else(|| {
      TokenCacheError::MalformedCacheValue("Invalid token:hash format".to_string())
    })?;

    let expires_at =
      OffsetDateTime::from_unix_timestamp(timestamp_str.parse().map_err(|_| {
        TokenCacheError::MalformedCacheValue("Invalid timestamp format".to_string())
      })?)
      .map_err(|_| TokenCacheError::MalformedCacheValue("Invalid timestamp".to_string()))?;

    Ok(Self {
      token: token.to_string(),
      hash: hash.to_string(),
      created_at: OffsetDateTime::now_utc(),
      expires_at,
    })
  }

  pub fn verify_hash(&self, token: &str) -> bool {
    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    self.hash == token_hash
  }

  pub fn is_expired(&self) -> bool {
    OffsetDateTime::now_utc() > self.expires_at
  }
}

pub trait TokenCache {
  fn get_access_token(&self, jti: &str) -> Result<Option<CachedToken>, TokenCacheError>;
  fn store_access_token(&self, jti: &str, token: CachedToken) -> Result<(), TokenCacheError>;
  fn get_refresh_token(
    &self,
    jti: &str,
  ) -> Result<Option<(String, OffsetDateTime)>, TokenCacheError>;
  fn store_refresh_token(
    &self,
    jti: &str,
    token: &str,
    expires_at: OffsetDateTime,
  ) -> Result<(), TokenCacheError>;
  fn store_token_pair(
    &self,
    jti: &str,
    access_token: CachedToken,
    refresh_token: Option<(String, OffsetDateTime)>,
  ) -> Result<(), TokenCacheError>;
  fn invalidate(&self, jti: &str) -> Result<(), TokenCacheError>;
}

pub struct DefaultTokenCache {
  cache_service: Arc<dyn CacheService>,
}

impl DefaultTokenCache {
  pub fn new(cache_service: Arc<dyn CacheService>) -> Self {
    Self { cache_service }
  }

  fn access_token_key(jti: &str) -> String {
    format!("{}{}", ACCESS_TOKEN_PREFIX, jti)
  }

  fn refresh_token_key(jti: &str) -> String {
    format!("{}{}", REFRESH_TOKEN_PREFIX, jti)
  }
}

impl TokenCache for DefaultTokenCache {
  fn get_access_token(&self, jti: &str) -> Result<Option<CachedToken>, TokenCacheError> {
    let key = Self::access_token_key(jti);
    match self.cache_service.get(&key) {
      Some(value) => {
        let token = CachedToken::from_cache_value(&value)?;
        if token.is_expired() {
          self.cache_service.remove(&key);
          Ok(None)
        } else {
          Ok(Some(token))
        }
      }
      None => Ok(None),
    }
  }

  fn store_access_token(&self, jti: &str, token: CachedToken) -> Result<(), TokenCacheError> {
    let key = Self::access_token_key(jti);
    self.cache_service.set(&key, &token.to_cache_value());
    Ok(())
  }

  fn get_refresh_token(
    &self,
    jti: &str,
  ) -> Result<Option<(String, OffsetDateTime)>, TokenCacheError> {
    let key = Self::refresh_token_key(jti);
    Ok(self.cache_service.get(&key).map(|value| {
      let parts: Vec<&str> = value.split(TIME_SEPARATOR).collect();
      let token = parts[0].to_string();
      let expires_at = OffsetDateTime::from_unix_timestamp(parts[1].parse().unwrap()).unwrap();
      (token, expires_at)
    }))
  }

  fn store_refresh_token(
    &self,
    jti: &str,
    token: &str,
    expires_at: OffsetDateTime,
  ) -> Result<(), TokenCacheError> {
    let key = Self::refresh_token_key(jti);
    let value = format!("{}{}{}", token, TIME_SEPARATOR, expires_at.unix_timestamp());
    self.cache_service.set(&key, &value);
    Ok(())
  }

  fn store_token_pair(
    &self,
    jti: &str,
    access_token: CachedToken,
    refresh_token: Option<(String, OffsetDateTime)>,
  ) -> Result<(), TokenCacheError> {
    self.store_access_token(jti, access_token)?;
    if let Some((token, expires_at)) = refresh_token {
      self.store_refresh_token(jti, &token, expires_at)?;
    }
    Ok(())
  }

  fn invalidate(&self, jti: &str) -> Result<(), TokenCacheError> {
    self.cache_service.remove(&Self::access_token_key(jti));
    self.cache_service.remove(&Self::refresh_token_key(jti));
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
    let cached = CachedToken::new(token.clone(), Duration::minutes(5));
    assert_eq!(cached.token, token);
    assert!(cached.verify_hash(&token));
    assert!(!cached.is_expired());
  }

  #[test]
  fn test_cached_token_verify_hash() {
    let token = "test-token".to_string();
    let cached = CachedToken::new(token.clone(), Duration::minutes(5));
    assert!(cached.verify_hash(&token));
    assert!(!cached.verify_hash("different-token"));
  }

  #[test]
  fn test_cached_token_expiration() {
    let token = "test-token".to_string();
    let cached = CachedToken::new(token.clone(), Duration::minutes(-5)); // Expired
    assert!(cached.is_expired());
  }

  #[test]
  fn test_cached_token_serialization() {
    let token = "test-token".to_string();
    let cached = CachedToken::new(token, Duration::minutes(5));
    let cache_value = cached.to_cache_value();
    let deserialized = CachedToken::from_cache_value(&cache_value).unwrap();
    assert_eq!(cached.token, deserialized.token);
    assert_eq!(cached.hash, deserialized.hash);
    assert_eq!(
      cached.expires_at.unix_timestamp(),
      deserialized.expires_at.unix_timestamp()
    );
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
    let token = CachedToken::new("test-token".to_string(), Duration::minutes(5));

    token_cache.store_access_token(jti, token.clone()).unwrap();

    let retrieved = token_cache.get_access_token(jti).unwrap().unwrap();
    assert_eq!(token.token, retrieved.token);
    assert_eq!(token.hash, retrieved.hash);
  }

  #[tokio::test]
  async fn test_expired_access_token_is_removed() {
    let (token_cache, cache_service) = create_test_cache();
    let jti = "test-jti";
    let token = CachedToken::new("test-token".to_string(), Duration::minutes(-5)); // Expired

    token_cache.store_access_token(jti, token).unwrap();
    assert!(token_cache.get_access_token(jti).unwrap().is_none());
    assert!(cache_service
      .get(&DefaultTokenCache::access_token_key(jti))
      .is_none());
  }

  #[tokio::test]
  async fn test_store_and_get_refresh_token() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let token = "refresh-token";
    let expires_at = OffsetDateTime::now_utc() + Duration::days(30);

    token_cache
      .store_refresh_token(jti, token, expires_at)
      .unwrap();

    let retrieved = token_cache.get_refresh_token(jti).unwrap().unwrap();
    assert_eq!(token, retrieved.0);
    assert_eq!(expires_at.unix_timestamp(), retrieved.1.unix_timestamp());
  }

  #[tokio::test]
  async fn test_store_token_pair() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let access_token = CachedToken::new("access-token".to_string(), Duration::minutes(5));
    let expires_at = OffsetDateTime::now_utc() + Duration::days(30);
    let refresh_token = Some(("refresh-token".to_string(), expires_at));

    token_cache
      .store_token_pair(jti, access_token.clone(), refresh_token.clone())
      .unwrap();

    let retrieved_access = token_cache.get_access_token(jti).unwrap().unwrap();
    assert_eq!(access_token.token, retrieved_access.token);

    let retrieved_refresh = token_cache.get_refresh_token(jti).unwrap().unwrap();
    assert_eq!(refresh_token.unwrap().0, retrieved_refresh.0);
  }

  #[tokio::test]
  async fn test_invalidate_tokens() {
    let (token_cache, _) = create_test_cache();
    let jti = "test-jti";
    let access_token = CachedToken::new("access-token".to_string(), Duration::minutes(5));
    let expires_at = OffsetDateTime::now_utc() + Duration::days(30);
    let refresh_token = Some(("refresh-token".to_string(), expires_at));

    token_cache
      .store_token_pair(jti, access_token, refresh_token)
      .unwrap();

    token_cache.invalidate(jti).unwrap();

    assert!(token_cache.get_access_token(jti).unwrap().is_none());
    assert!(token_cache.get_refresh_token(jti).unwrap().is_none());
  }

  #[tokio::test]
  async fn test_nonexistent_tokens() {
    let (token_cache, _) = create_test_cache();
    let jti = "nonexistent-jti";

    assert!(token_cache.get_access_token(jti).unwrap().is_none());
    assert!(token_cache.get_refresh_token(jti).unwrap().is_none());
  }
}
