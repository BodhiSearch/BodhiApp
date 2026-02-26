use rand::Rng;
use serde::{Deserialize, Serialize};
use services::{AppInstanceService, AppStatus};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiErrorResponse {
  error: String,
}

pub fn generate_random_string(length: usize) -> String {
  const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let mut rng = rand::rng();
  (0..length)
    .map(|_| {
      let idx = rng.random_range(0..CHARSET.len());
      CHARSET[idx] as char
    })
    .collect()
}

pub async fn app_status_or_default(
  app_instance_service: &Arc<dyn AppInstanceService>,
) -> AppStatus {
  app_instance_service.get_status().await.unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use crate::app_status_or_default;
  use rstest::rstest;
  use services::test_utils::{test_db_service, TestDbService};
  use services::{AppInstanceService, AppStatus, DefaultAppInstanceService};
  use std::sync::Arc;

  #[rstest]
  #[case(AppStatus::Setup)]
  #[case(AppStatus::Ready)]
  #[case(AppStatus::ResourceAdmin)]
  #[awt]
  #[tokio::test]
  async fn test_app_status_or_default(
    #[case] expected: AppStatus,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let svc = DefaultAppInstanceService::new(Arc::new(test_db_service));
    svc
      .create_instance("test-client", "test-secret", expected.clone())
      .await?;
    let svc: Arc<dyn AppInstanceService> = Arc::new(svc);
    assert_eq!(expected, app_status_or_default(&svc).await);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_app_status_or_default_not_found(
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let svc: Arc<dyn AppInstanceService> =
      Arc::new(DefaultAppInstanceService::new(Arc::new(test_db_service)));
    assert_eq!(AppStatus::default(), app_status_or_default(&svc).await);
    Ok(())
  }
}
