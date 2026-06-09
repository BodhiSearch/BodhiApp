use crate::ResourceRole;
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Error is boxed to avoid circular dependencies between crates.
pub type AuthTokenResult = Result<(String, ResourceRole), Box<dyn StdError + Send + Sync>>;

/// Result type for string-returning lock operations (e.g., dashboard token refresh).
pub type StringLockResult = Result<String, Box<dyn StdError + Send + Sync>>;

/// Cannot be auto-mocked by mockall due to the closure parameter;
/// use `LocalConcurrencyService` directly in tests.
#[async_trait]
pub trait ConcurrencyService: Send + Sync + std::fmt::Debug {
  /// Only one task runs the closure per key at a time; different keys run concurrently.
  async fn with_lock_auth(
    &self,
    key: &str,
    f: Box<dyn FnOnce() -> BoxFuture<'static, AuthTokenResult> + Send + 'static>,
  ) -> AuthTokenResult;

  async fn with_lock_str(
    &self,
    key: &str,
    f: Box<dyn FnOnce() -> BoxFuture<'static, StringLockResult> + Send + 'static>,
  ) -> StringLockResult;
}

/// Locks are never removed from the map; acceptable because the unique-key set
/// (e.g. session IDs) is bounded. Add TTL/weak-ref eviction if memory grows.
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

  /// Double-checked locking: read-lock fast path, write-lock to insert.
  async fn get_lock(&self, key: &str) -> Arc<Mutex<()>> {
    {
      let locks = self.locks.read().await;
      if let Some(lock) = locks.get(key) {
        return Arc::clone(lock);
      }
    }

    let mut locks = self.locks.write().await;
    // Re-check in case another task inserted while we waited for the write lock.
    if let Some(lock) = locks.get(key) {
      return Arc::clone(lock);
    }

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
    let lock = self.get_lock(key).await;
    let _guard = lock.lock().await;
    f().await
  }

  async fn with_lock_str(
    &self,
    key: &str,
    f: Box<dyn FnOnce() -> BoxFuture<'static, StringLockResult> + Send + 'static>,
  ) -> StringLockResult {
    let lock = self.get_lock(key).await;
    let _guard = lock.lock().await;
    f().await
  }
}

#[cfg(test)]
mod tests {
  use super::{ConcurrencyService, LocalConcurrencyService, StdError};
  use crate::ResourceRole;
  use pretty_assertions::assert_eq;
  use std::sync::atomic::{AtomicU32, Ordering};
  use std::sync::Arc;
  use std::time::Duration;
  use tokio::time::sleep;

  #[tokio::test]
  async fn test_sequential_execution_same_key() {
    let service = LocalConcurrencyService::new();
    let counter = Arc::new(AtomicU32::new(0));

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
              Ok(("token1".to_string(), ResourceRole::Guest))
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
              Ok(("token2".to_string(), ResourceRole::Guest))
            })
          }),
        )
        .await;
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    assert_eq!(2, counter.load(Ordering::SeqCst));
  }

  #[tokio::test]
  async fn test_concurrent_execution_different_keys() {
    let service = Arc::new(LocalConcurrencyService::new());
    let start_time = std::time::Instant::now();

    let service1 = Arc::clone(&service);
    let service2 = Arc::clone(&service);

    let handle1 = tokio::spawn(async move {
      let _ = service1
        .with_lock_auth(
          "key1",
          Box::new(|| {
            Box::pin(async {
              sleep(Duration::from_millis(50)).await;
              Ok(("token1".to_string(), ResourceRole::Guest))
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
              Ok(("token2".to_string(), ResourceRole::Guest))
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

    let _ = service
      .with_lock_auth(
        "reuse_key",
        Box::new(|| Box::pin(async { Ok(("token1".to_string(), ResourceRole::Guest)) })),
      )
      .await;

    let locks_count_before = {
      let locks = service.locks.read().await;
      locks.len()
    };

    let _ = service
      .with_lock_auth(
        "reuse_key",
        Box::new(|| Box::pin(async { Ok(("token2".to_string(), ResourceRole::Guest)) })),
      )
      .await;

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
        Box::new(|| Box::pin(async { Ok(("test_token".to_string(), ResourceRole::User)) })),
      )
      .await;

    assert!(result.is_ok());
    let (token, role) = result.unwrap();
    assert_eq!("test_token", token);
    assert_eq!(ResourceRole::User, role);
  }

  #[tokio::test]
  async fn test_error_propagation() {
    let service = LocalConcurrencyService::new();

    let result = service
      .with_lock_auth(
        "error_test",
        Box::new(|| {
          Box::pin(async {
            Err(Box::new(std::io::Error::other("test error")) as Box<dyn StdError + Send + Sync>)
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
                Ok(("token".to_string(), ResourceRole::Guest))
              })
            }),
          )
          .await;
      });

      handles.push(handle);
    }

    for handle in handles {
      handle.await.unwrap();
    }

    assert_eq!(10, counter.load(Ordering::SeqCst));
  }
}
