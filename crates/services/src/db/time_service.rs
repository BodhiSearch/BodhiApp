use chrono::{DateTime, Timelike, Utc};
use std::{fs, path::Path, time::UNIX_EPOCH};

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait TimeService: std::fmt::Debug + Send + Sync {
  fn utc_now(&self) -> DateTime<Utc>;

  fn created_at(&self, path: &Path) -> u32;
}

#[derive(Debug, Clone, Default)]
pub struct DefaultTimeService;

impl TimeService for DefaultTimeService {
  fn utc_now(&self) -> DateTime<Utc> {
    let now = chrono::Utc::now();
    now.with_nanosecond(0).unwrap_or(now)
  }

  fn created_at(&self, path: &Path) -> u32 {
    fs::metadata(path)
      .map_err(|e| e.to_string())
      .and_then(|m| m.created().map_err(|e| e.to_string()))
      .and_then(|t| t.duration_since(UNIX_EPOCH).map_err(|e| e.to_string()))
      .unwrap_or_default()
      .as_secs() as u32
  }
}
