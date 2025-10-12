use async_trait::async_trait;
use objs::ResourceRole;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// A boxed future that can be sent across threads.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Result type for authentication token operations.
///
/// This type represents the result of token validation and refresh operations,
/// returning a tuple of (access_token, optional_role) on success, or a boxed error on failure.
/// The error is boxed to avoid circular dependencies between crates.
pub type AuthTokenResult = Result<(String, Option<ResourceRole>), Box<dyn StdError + Send + Sync>>;

/// Service for managing distributed locks and concurrency control.
///
/// This trait provides a foundation for implementing both local and distributed
/// locking mechanisms. The local implementation uses in-process locks, while
/// future distributed implementations could use Redis, etcd, or other distributed
/// coordination systems.
///
/// Currently, this trait provides an auth-specific method for token refresh operations.
/// Additional methods can be added in the future for other use cases.
///
/// Note: This trait cannot be auto-mocked by mockall due to the closure parameter.
/// Use `LocalConcurrencyService` directly in tests as it's lightweight.
#[async_trait]
pub trait ConcurrencyService: Send + Sync + std::fmt::Debug {
  /// Execute an authentication token operation while holding a lock for the given key.
  ///
  /// This ensures that only one task can execute the closure for a specific key
  /// at a time. Different keys can execute concurrently.
  ///
  /// This method is specifically designed for authentication token refresh operations,
  /// where multiple concurrent requests might try to refresh the same token.
  ///
  /// # Arguments
  /// * `key` - The lock key to acquire (e.g., "refresh_token:session_id")
  /// * `f` - The async closure to execute while holding the lock
  ///
  /// # Returns
  /// The result of the authentication operation: `(access_token, optional_role)`
  async fn with_lock_auth(
    &self,
    key: &str,
    f: Box<dyn FnOnce() -> BoxFuture<'static, AuthTokenResult> + Send + 'static>,
  ) -> AuthTokenResult;
}

/// Local implementation of ConcurrencyService using in-process locks.
///
/// This implementation uses a `RwLock<HashMap>` to manage per-key locks.
/// Each key gets its own `Arc<Mutex<()>>` which acts as the lock sentinel.
/// The RwLock allows concurrent reads to look up locks, while writes are
/// only needed when creating new lock entries.
///
/// # Lock Cleanup
/// Currently, locks are never removed from the map. For most use cases this
/// is acceptable since the number of unique keys (e.g., session IDs) is bounded.
/// If memory usage becomes a concern, a cleanup mechanism could be added using
/// weak references or TTL-based eviction.
#[derive(Debug, Default)]
pub struct LocalConcurrencyService {
  locks: RwLock<HashMap<String, Arc<Mutex<()>>>>,
}

impl LocalConcurrencyService {
  pub fn new() -> Self {
    Self {
      locks: RwLock::new(HashMap::new()),
    }
  }

  /// Get or create a lock for the given key.
  ///
  /// This uses a double-checked locking pattern:
  /// 1. Try to get the lock with a read lock (fast path)
  /// 2. If not found, acquire write lock and insert new lock (slow path)
  async fn get_lock(&self, key: &str) -> Arc<Mutex<()>> {
    // Fast path: try to get existing lock with read lock
    {
      let locks = self.locks.read().await;
      if let Some(lock) = locks.get(key) {
        return Arc::clone(lock);
      }
    }

    // Slow path: need to create new lock with write lock
    let mut locks = self.locks.write().await;
    // Double-check in case another task inserted while we waited for write lock
    if let Some(lock) = locks.get(key) {
      return Arc::clone(lock);
    }

    // Create new lock and insert
    let lock = Arc::new(Mutex::new(()));
    locks.insert(key.to_string(), Arc::clone(&lock));
    lock
  }
}

#[async_trait]
impl ConcurrencyService for LocalConcurrencyService {
  async fn with_lock_auth(
    &self,
    key: &str,
    f: Box<dyn FnOnce() -> BoxFuture<'static, AuthTokenResult> + Send + 'static>,
  ) -> AuthTokenResult {
    // Get or create the lock for this key
    let lock = self.get_lock(key).await;

    // Acquire the lock (blocks if another task holds it)
    let _guard = lock.lock().await;

    // Execute the closure while holding the lock
    f().await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicU32, Ordering};
  use std::time::Duration;
  use tokio::time::sleep;

  #[tokio::test]
  async fn test_sequential_execution_same_key() {
    let service = LocalConcurrencyService::new();
    let counter = Arc::new(AtomicU32::new(0));

    // Spawn two tasks that increment a counter
    let counter1 = Arc::clone(&counter);
    let service1 = Arc::new(service);
    let service2 = Arc::clone(&service1);

    let handle1 = tokio::spawn(async move {
      let _ = service1
        .with_lock_auth(
          "test_key",
          Box::new(move || {
            Box::pin(async move {
              let val = counter1.load(Ordering::SeqCst);
              sleep(Duration::from_millis(10)).await;
              counter1.store(val + 1, Ordering::SeqCst);
              Ok(("token1".to_string(), None))
            })
          }),
        )
        .await;
    });

    let counter2 = Arc::clone(&counter);
    let handle2 = tokio::spawn(async move {
      let _ = service2
        .with_lock_auth(
          "test_key",
          Box::new(move || {
            Box::pin(async move {
              let val = counter2.load(Ordering::SeqCst);
              sleep(Duration::from_millis(10)).await;
              counter2.store(val + 1, Ordering::SeqCst);
              Ok(("token2".to_string(), None))
            })
          }),
        )
        .await;
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    // Both increments should succeed due to locking
    assert_eq!(2, counter.load(Ordering::SeqCst));
  }

  #[tokio::test]
  async fn test_concurrent_execution_different_keys() {
    let service = Arc::new(LocalConcurrencyService::new());
    let start_time = std::time::Instant::now();

    // Spawn two tasks with different keys - they should run concurrently
    let service1 = Arc::clone(&service);
    let service2 = Arc::clone(&service);

    let handle1 = tokio::spawn(async move {
      let _ = service1
        .with_lock_auth(
          "key1",
          Box::new(|| {
            Box::pin(async {
              sleep(Duration::from_millis(50)).await;
              Ok(("token1".to_string(), None))
            })
          }),
        )
        .await;
    });

    let handle2 = tokio::spawn(async move {
      let _ = service2
        .with_lock_auth(
          "key2",
          Box::new(|| {
            Box::pin(async {
              sleep(Duration::from_millis(50)).await;
              Ok(("token2".to_string(), None))
            })
          }),
        )
        .await;
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    let elapsed = start_time.elapsed();

    // Should complete in ~50ms (concurrent) not ~100ms (sequential)
    // Using 80ms threshold to account for scheduling overhead
    assert!(
      elapsed < Duration::from_millis(80),
      "Elapsed time {:?} suggests sequential execution",
      elapsed
    );
  }

  #[tokio::test]
  async fn test_lock_reuse() {
    let service = Arc::new(LocalConcurrencyService::new());

    // First use of key
    let _ = service
      .with_lock_auth(
        "reuse_key",
        Box::new(|| Box::pin(async { Ok(("token1".to_string(), None)) })),
      )
      .await;

    // Get the lock entry count
    let locks_count_before = {
      let locks = service.locks.read().await;
      locks.len()
    };

    // Second use of same key
    let _ = service
      .with_lock_auth(
        "reuse_key",
        Box::new(|| Box::pin(async { Ok(("token2".to_string(), None)) })),
      )
      .await;

    // Lock entry should be reused, not duplicated
    let locks_count_after = {
      let locks = service.locks.read().await;
      locks.len()
    };

    assert_eq!(locks_count_before, locks_count_after);
    assert_eq!(1, locks_count_after);
  }

  #[tokio::test]
  async fn test_closure_return_value() {
    let service = LocalConcurrencyService::new();

    let result = service
      .with_lock_auth(
        "return_test",
        Box::new(|| Box::pin(async { Ok(("test_token".to_string(), Some(ResourceRole::User))) })),
      )
      .await;

    assert!(result.is_ok());
    let (token, role) = result.unwrap();
    assert_eq!("test_token", token);
    assert_eq!(Some(ResourceRole::User), role);
  }

  #[tokio::test]
  async fn test_error_propagation() {
    let service = LocalConcurrencyService::new();

    let result = service
      .with_lock_auth(
        "error_test",
        Box::new(|| {
          Box::pin(async {
            Err(
              Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
                as Box<dyn StdError + Send + Sync>,
            )
          })
        }),
      )
      .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("test error"));
  }

  #[tokio::test]
  async fn test_multiple_concurrent_tasks_same_key() {
    let service = Arc::new(LocalConcurrencyService::new());
    let counter = Arc::new(AtomicU32::new(0));
    let mut handles = vec![];

    // Spawn 10 tasks all using the same key
    for _ in 0..10 {
      let service_clone = Arc::clone(&service);
      let counter_clone = Arc::clone(&counter);

      let handle = tokio::spawn(async move {
        let _ = service_clone
          .with_lock_auth(
            "shared_key",
            Box::new(move || {
              Box::pin(async move {
                let val = counter_clone.load(Ordering::SeqCst);
                // Small delay to increase chance of race condition without lock
                sleep(Duration::from_millis(1)).await;
                counter_clone.store(val + 1, Ordering::SeqCst);
                Ok(("token".to_string(), None))
              })
            }),
          )
          .await;
      });

      handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
      handle.await.unwrap();
    }

    // All 10 increments should succeed without race conditions
    assert_eq!(10, counter.load(Ordering::SeqCst));
  }
}
