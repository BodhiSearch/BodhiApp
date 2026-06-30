use mini_moka::sync::Cache;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait CacheService: Send + Sync + std::fmt::Debug {
  fn get(&self, key: &str) -> Option<String>;

  fn set(&self, key: &str, value: &str);

  fn remove(&self, key: &str);

  /// Invalidate every entry whose stored value contains `needle`. Iterates in
  /// place, collecting only the matching keys (no full snapshot / value clones),
  /// then invalidates them. O(n) over the cache — for rare admin operations
  /// (e.g. evicting cached exchange results for a revoked access request), not
  /// the hot path.
  ///
  /// Substring match (not a closure predicate) because a `dyn`-safe trait can't
  /// take a generic `F: Fn` and mockall can't mock a `&dyn Fn` argument. Callers
  /// pass a value precise enough to avoid false matches (e.g. a serialized
  /// `"field":"<id>"` fragment).
  fn remove_entries_containing(&self, needle: &str);
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

  fn remove_entries_containing(&self, needle: &str) {
    // Two-pass: collect matching keys (mini-moka can't mutate while iterating),
    // then invalidate. Only matching keys are cloned — values are not.
    let keys: Vec<String> = self
      .cache
      .iter()
      .filter(|entry| entry.value().contains(needle))
      .map(|entry| entry.key().clone())
      .collect();
    for key in &keys {
      self.cache.invalidate(key);
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{CacheService, MokaCacheService};
  use pretty_assertions::assert_eq;

  #[test]
  fn test_cache_service() {
    let cache_service = MokaCacheService::default();

    cache_service.set("key1", "value1");
    assert_eq!(cache_service.get("key1"), Some("value1".to_string()));

    cache_service.remove("key1");
    assert_eq!(cache_service.get("key1"), None);
  }

  #[test]
  fn test_remove_entries_containing() {
    let cache = MokaCacheService::default();
    cache.set(
      "exchanged_token:aaa",
      r#"{"access_request_id":"ar-1","x":1}"#,
    );
    cache.set(
      "exchanged_token:bbb",
      r#"{"access_request_id":"ar-1","x":2}"#,
    );
    cache.set(
      "exchanged_token:ccc",
      r#"{"access_request_id":"ar-2","x":3}"#,
    );
    cache.set("unrelated", "no match here");

    cache.remove_entries_containing(r#""access_request_id":"ar-1""#);

    // Both ar-1 entries gone; ar-2 and the unrelated entry remain.
    assert_eq!(cache.get("exchanged_token:aaa"), None);
    assert_eq!(cache.get("exchanged_token:bbb"), None);
    assert!(cache.get("exchanged_token:ccc").is_some());
    assert!(cache.get("unrelated").is_some());
  }
}
