use crate::{AppError, ErrorType};
use std::collections::HashMap;

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("Concurrent access error: {reason}.")]
pub struct RwLockReadError {
  reason: String,
}

impl RwLockReadError {
  pub fn new(reason: impl Into<String>) -> Self {
    Self {
      reason: reason.into(),
    }
  }
}

impl AppError for RwLockReadError {
  fn error_type(&self) -> String {
    ErrorType::InternalServer.to_string()
  }

  fn code(&self) -> String {
    "rw_lock_read_error".to_string()
  }

  fn args(&self) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("reason".to_string(), self.reason.clone());
    map
  }
}

#[cfg(test)]
#[path = "test_rwlock_error.rs"]
mod test_rwlock_error;
