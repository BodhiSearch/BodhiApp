use mini_moka::sync::Cache;
use std::time::Duration;

#[cfg_attr(test, mockall::automock)]
pub trait CacheService: Send + Sync + std::fmt::Debug {
  fn get(&self, key: &str) -> Option<String>;

  fn set(&self, key: &str, value: &str);

  fn remove(&self, key: &str);
}

#[derive(Debug)]
pub struct MokaCacheService {
  cache: Cache<String, String>,
}

impl MokaCacheService {
  pub fn new(max_capacity: Option<u64>, time_to_live: Option<Duration>) -> Self {
    let mut builder = Cache::builder();
    if let Some(max_capacity) = max_capacity {
      builder = builder.max_capacity(max_capacity);
    }
    if let Some(time_to_live) = time_to_live {
      builder = builder.time_to_live(time_to_live);
    }
    Self {
      cache: builder.build(),
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
  use super::*;
  use std::thread;

  #[test]
  fn test_cache_service() {
    let cache_service = MokaCacheService::new(Some(100), Some(Duration::from_secs(1)));

    cache_service.set("key1", "value1");
    assert_eq!(cache_service.get("key1"), Some("value1".to_string()));

    cache_service.remove("key1");
    assert_eq!(cache_service.get("key1"), None);

    cache_service.set("key2", "value2");
    thread::sleep(Duration::from_secs(2));
    assert_eq!(cache_service.get("key2"), None);
  }
}
