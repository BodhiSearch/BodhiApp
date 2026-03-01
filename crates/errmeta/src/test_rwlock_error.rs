use crate::{AppError, RwLockReadError};
use rstest::rstest;

#[rstest]
fn test_rwlock_read_error() {
  let error = RwLockReadError::new("lock poisoned");
  assert_eq!("Concurrent access error: lock poisoned.", error.to_string());
  assert_eq!("rw_lock_read_error", error.code());
  assert_eq!(500, error.status());
  assert_eq!("lock poisoned", error.args()["reason"]);
}
