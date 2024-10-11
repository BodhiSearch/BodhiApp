use rand::Rng;
use serde::{Deserialize, Serialize};
use services::{AppStatus, SecretService, KEY_APP_STATUS};
use std::{str::FromStr, sync::Arc};

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
  let value = secret_service.get_secret_string(KEY_APP_STATUS);
  match value {
    Ok(Some(value)) => AppStatus::from_str(&value).unwrap_or(AppStatus::default()),
    Ok(None) => AppStatus::default(),
    Err(_) => AppStatus::default(),
  }
}

#[cfg(test)]
mod tests {
  use crate::app_status_or_default;
  use rstest::rstest;
  use services::{
    test_utils::SecretServiceStub, AppStatus, SecretService, APP_STATUS_READY, APP_STATUS_SETUP,
    KEY_APP_STATUS,
  };
  use std::sync::Arc;

  #[rstest]
  #[case(APP_STATUS_SETUP, AppStatus::Setup)]
  #[case(APP_STATUS_READY, AppStatus::Ready)]
  #[case("resource-admin", AppStatus::ResourceAdmin)]
  fn test_app_status_or_default(
    #[case] status: &str,
    #[case] expected: AppStatus,
  ) -> anyhow::Result<()> {
    let secret_service = SecretServiceStub::with_map(maplit::hashmap! {
      KEY_APP_STATUS.to_string() => status.to_string(),
    });
    assert_eq!(
      expected,
      app_status_or_default(&(Arc::new(secret_service) as Arc<dyn SecretService>))
    );
    Ok(())
  }

  #[rstest]
  fn test_app_status_or_default_not_found() -> anyhow::Result<()> {
    let secret_service: &Arc<dyn SecretService> =
      &(Arc::new(SecretServiceStub::new()) as Arc<dyn SecretService>);
    assert_eq!(AppStatus::default(), app_status_or_default(secret_service));
    assert_eq!(AppStatus::Setup, app_status_or_default(secret_service));
    Ok(())
  }
}
