use rand::Rng;
use serde::{Deserialize, Serialize};
use services::{AppStatus, SecretService, SecretServiceExt};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorResponse {
  error: String,
}

pub fn generate_random_string(length: usize) -> String {
  const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let mut rng = rand::thread_rng();
  (0..length)
    .map(|_| {
      let idx = rng.gen_range(0..CHARSET.len());
      CHARSET[idx] as char
    })
    .collect()
}

pub fn app_status_or_default(secret_service: &Arc<dyn SecretService>) -> AppStatus {
  secret_service.app_status().unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use crate::app_status_or_default;
  use rstest::rstest;
  use services::{test_utils::SecretServiceStub, AppStatus, SecretService};
  use std::sync::Arc;

  #[rstest]
  fn test_app_status_or_default_not_found() -> anyhow::Result<()> {
    let secret_service: &Arc<dyn SecretService> =
      &(Arc::new(SecretServiceStub::new()) as Arc<dyn SecretService>);
    assert_eq!(AppStatus::default(), app_status_or_default(secret_service));
    assert_eq!(AppStatus::Setup, app_status_or_default(secret_service));
    Ok(())
  }
}
