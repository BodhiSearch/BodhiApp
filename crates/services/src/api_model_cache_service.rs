use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use objs::{AppError, ErrorType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{db::DbService, AiApiService, AiApiServiceError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiModelCacheError {
  #[error(transparent)]
  AiApiService(#[from] AiApiServiceError),

  #[error("api_model_not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ApiModelNotFound(String),
}

type Result<T> = std::result::Result<T, ApiModelCacheError>;

#[derive(Debug, Clone)]
struct CacheEntry {
  models: Vec<String>,
  fetched_at: DateTime<Utc>,
}

/// Service for caching API model lists with stale-while-revalidate pattern
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait ApiModelCacheService: Send + Sync + std::fmt::Debug {
  /// Get models for API alias. Returns cached (even stale) and triggers async refresh if expired.
  /// If not in cache, fetches synchronously.
  async fn get_models(&self, api_alias_id: &str) -> Result<Vec<String>>;

  /// Clear cache for specific alias (called on delete)
  async fn invalidate(&self, api_alias_id: &str);
}

/// Default implementation of ApiModelCacheService with in-memory cache
#[derive(Debug, Clone)]
pub struct DefaultApiModelCacheService {
  cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
  ai_api_service: Arc<dyn AiApiService>,
  db_service: Arc<dyn DbService>,
  ttl: Duration,
}

impl DefaultApiModelCacheService {
  pub fn new(ai_api_service: Arc<dyn AiApiService>, db_service: Arc<dyn DbService>) -> Self {
    Self {
      cache: Arc::new(RwLock::new(HashMap::new())),
      ai_api_service,
      db_service,
      ttl: Duration::hours(24),
    }
  }

  /// Check if cache entry is stale (older than TTL)
  fn is_stale(&self, entry: &CacheEntry) -> bool {
    Utc::now() - entry.fetched_at > self.ttl
  }

  /// Fetch models from AI API for a given alias ID
  async fn fetch_from_api(&self, api_alias_id: &str) -> Result<Vec<String>> {
    // Get API alias configuration from database
    let api_alias = self
      .db_service
      .get_api_model_alias(api_alias_id)
      .await
      .map_err(|_| ApiModelCacheError::ApiModelNotFound(api_alias_id.to_string()))?
      .ok_or_else(|| ApiModelCacheError::ApiModelNotFound(api_alias_id.to_string()))?;

    // Get API key (optional)
    let api_key = self
      .db_service
      .get_api_key_for_alias(api_alias_id)
      .await
      .ok()
      .flatten();

    // Fetch models from API
    let models = self
      .ai_api_service
      .fetch_models(api_key, &api_alias.base_url)
      .await?;

    Ok(models)
  }

  /// Spawn async task to refresh cache entry
  fn spawn_refresh(&self, api_alias_id: String) {
    let cache = self.cache.clone();
    let service = self.clone();

    tokio::spawn(async move {
      match service.fetch_from_api(&api_alias_id).await {
        Ok(models) => {
          let mut cache_write = cache.write().await;
          cache_write.insert(
            api_alias_id.clone(),
            CacheEntry {
              models,
              fetched_at: Utc::now(),
            },
          );
        }
        Err(_) => {
          // Silently fail background refresh - stale data remains
        }
      }
    });
  }
}

#[async_trait]
impl ApiModelCacheService for DefaultApiModelCacheService {
  async fn get_models(&self, api_alias_id: &str) -> Result<Vec<String>> {
    // Check cache first
    let cache_read = self.cache.read().await;
    if let Some(entry) = cache_read.get(api_alias_id) {
      let is_stale = self.is_stale(entry);
      let models = entry.models.clone();
      drop(cache_read); // Release read lock

      if is_stale {
        // Return stale data immediately, refresh in background
        self.spawn_refresh(api_alias_id.to_string());
      }

      return Ok(models);
    }
    drop(cache_read); // Release read lock before fetching

    // Cache miss - fetch synchronously
    let models = self.fetch_from_api(api_alias_id).await?;

    // Update cache
    let mut cache_write = self.cache.write().await;
    cache_write.insert(
      api_alias_id.to_string(),
      CacheEntry {
        models: models.clone(),
        fetched_at: Utc::now(),
      },
    );

    Ok(models)
  }

  async fn invalidate(&self, api_alias_id: &str) {
    let mut cache_write = self.cache.write().await;
    cache_write.remove(api_alias_id);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::MockDbService;
  use crate::MockAiApiService;
  use chrono::Utc;
  use mockall::predicate::eq;
  use objs::{ApiAlias, ApiFormat};
  use rstest::rstest;
  use std::sync::Arc;

  #[rstest]
  #[tokio::test]
  async fn test_cache_miss_fetches_sync() -> anyhow::Result<()> {
    let mut mock_db = MockDbService::new();
    let mut mock_ai = MockAiApiService::new();

    let api_alias = ApiAlias::new(
      "test-api",
      ApiFormat::OpenAI,
      "https://api.test.com/v1",
      vec!["model-1".to_string(), "model-2".to_string()],
      None,
      false,
      Utc::now(),
    );

    // Mock DB calls
    mock_db
      .expect_get_api_model_alias()
      .with(eq("test-api"))
      .returning(move |_| Ok(Some(api_alias.clone())));

    mock_db
      .expect_get_api_key_for_alias()
      .with(eq("test-api"))
      .returning(|_| Ok(Some("test-key".to_string())));

    // Mock AI API call
    mock_ai
      .expect_fetch_models()
      .with(
        eq(Some("test-key".to_string())),
        eq("https://api.test.com/v1"),
      )
      .returning(|_, _| Ok(vec!["model-1".to_string(), "model-2".to_string()]));

    let service = DefaultApiModelCacheService::new(Arc::new(mock_ai), Arc::new(mock_db));

    let models = service.get_models("test-api").await?;
    assert_eq!(vec!["model-1", "model-2"], models);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_cache_hit_returns_immediately() -> anyhow::Result<()> {
    let mock_db = MockDbService::new();
    let mock_ai = MockAiApiService::new();

    let service = DefaultApiModelCacheService::new(Arc::new(mock_ai), Arc::new(mock_db));

    // Pre-populate cache
    {
      let mut cache = service.cache.write().await;
      cache.insert(
        "test-api".to_string(),
        CacheEntry {
          models: vec!["cached-model".to_string()],
          fetched_at: Utc::now(),
        },
      );
    }

    let models = service.get_models("test-api").await?;
    assert_eq!(vec!["cached-model"], models);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_stale_cache_returns_and_refreshes() -> anyhow::Result<()> {
    let mut mock_db = MockDbService::new();
    let mut mock_ai = MockAiApiService::new();

    let api_alias = ApiAlias::new(
      "test-api",
      ApiFormat::OpenAI,
      "https://api.test.com/v1",
      vec!["new-model".to_string()],
      None,
      false,
      Utc::now(),
    );

    // Mock DB calls for background refresh
    mock_db
      .expect_get_api_model_alias()
      .with(eq("test-api"))
      .returning(move |_| Ok(Some(api_alias.clone())));

    mock_db
      .expect_get_api_key_for_alias()
      .with(eq("test-api"))
      .returning(|_| Ok(Some("test-key".to_string())));

    // Mock AI API call for background refresh
    mock_ai
      .expect_fetch_models()
      .with(
        eq(Some("test-key".to_string())),
        eq("https://api.test.com/v1"),
      )
      .returning(|_, _| Ok(vec!["new-model".to_string()]));

    let service = DefaultApiModelCacheService::new(Arc::new(mock_ai), Arc::new(mock_db));

    // Pre-populate cache with stale entry (25 hours old)
    {
      let mut cache = service.cache.write().await;
      cache.insert(
        "test-api".to_string(),
        CacheEntry {
          models: vec!["stale-model".to_string()],
          fetched_at: Utc::now() - Duration::hours(25),
        },
      );
    }

    // Should return stale data immediately
    let models = service.get_models("test-api").await?;
    assert_eq!(vec!["stale-model"], models);

    // Wait for background refresh to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify cache was refreshed
    let cache_read = service.cache.read().await;
    let entry = cache_read.get("test-api");
    assert!(entry.is_some());
    assert_eq!(vec!["new-model"], entry.unwrap().models);

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_invalidate_clears_entry() -> anyhow::Result<()> {
    let mock_db = MockDbService::new();
    let mock_ai = MockAiApiService::new();

    let service = DefaultApiModelCacheService::new(Arc::new(mock_ai), Arc::new(mock_db));

    // Pre-populate cache
    {
      let mut cache = service.cache.write().await;
      cache.insert(
        "test-api".to_string(),
        CacheEntry {
          models: vec!["model-1".to_string()],
          fetched_at: Utc::now(),
        },
      );
    }

    // Invalidate
    service.invalidate("test-api").await;

    // Verify cleared
    let cache_read = service.cache.read().await;
    assert!(cache_read.get("test-api").is_none());

    Ok(())
  }
}
