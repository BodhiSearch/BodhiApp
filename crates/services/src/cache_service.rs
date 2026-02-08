use mini_moka::sync::Cache;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait CacheService: Send + Sync + std::fmt::Debug {
  fn get(&self, key: &str) -> Option<String>;

  fn set(&self, key: &str, value: &str);

  fn remove(&self, key: &str);
}

#[derive(Debug)]
pub struct MokaCacheService {
  cache: Cache<String, String>,
}

impl Default for MokaCacheService {
  fn default() -> Self {
    Self {
      cache: Cache::builder().build(),
    }
  }
}

impl CacheService for MokaCacheService {
  fn get(&self, key: &str) -> Option<String> {
    self.cache.get(&key.to_string())
  }

  fn set(&self, key: &str, value: &str) {
    self.cache.insert(key.to_string(), value.to_string());
  }

  fn remove(&self, key: &str) {
    self.cache.invalidate(&key.to_string());
  }
}

#[cfg(test)]
mod tests {
  use crate::{CacheService, MokaCacheService};
  use anyhow_trace::anyhow_trace;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_cache_service() {
    let cache_service = MokaCacheService::default();

    cache_service.set("key1", "value1");
    assert_eq!(cache_service.get("key1"), Some("value1".to_string()));

    cache_service.remove("key1");
    assert_eq!(cache_service.get("key1"), None);
  }
}
