use rand::Rng;
use serde::{Deserialize, Serialize};
use services::{AppStatus, AuthScopedTenantService, TenantError};

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

/// Standalone-only helper. Returns the app status of the single registered tenant,
/// or `AppStatus::Setup` if no tenant exists. In multi-tenant mode this always
/// returns `AppStatus::Setup` because `get_standalone_app()` returns `None`.
/// Propagates errors (e.g., encryption key mismatch) instead of silently defaulting.
pub async fn standalone_app_status_or_default(
  tenant_service: &AuthScopedTenantService,
) -> Result<AppStatus, TenantError> {
  Ok(
    tenant_service
      .get_standalone_app()
      .await?
      .map(|t| t.status)
      .unwrap_or_default(),
  )
}

#[cfg(test)]
mod tests {
  use crate::middleware::utils::standalone_app_status_or_default;
  use rstest::rstest;
  use services::test_utils::{test_db_service, TestDbService};
  use services::{
    AppError, AppService, AppStatus, AuthContext, AuthScopedTenantService, DefaultTenantService,
    DeploymentMode, TenantService,
  };
  use std::sync::Arc;

  fn make_auth_scoped_tenant(app_service: Arc<dyn AppService>) -> AuthScopedTenantService {
    AuthScopedTenantService::new(
      app_service,
      AuthContext::Anonymous {
        deployment: DeploymentMode::Standalone,
      },
    )
  }

  #[rstest]
  #[case(AppStatus::Setup, None)]
  #[case(AppStatus::Ready, Some("test-user".to_string()))]
  #[case(AppStatus::ResourceAdmin, None)]
  #[awt]
  #[tokio::test]
  async fn test_app_status_or_default(
    #[case] expected: AppStatus,
    #[case] created_by: Option<String>,
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(test_db_service);
    let tenant_svc: Arc<dyn TenantService> =
      Arc::new(DefaultTenantService::new(db_service.clone()));
    tenant_svc
      .create_tenant(
        "test-client",
        "test-secret",
        "Test App",
        None,
        expected.clone(),
        created_by,
      )
      .await?;
    let app_service = services::test_utils::AppServiceStubBuilder::default()
      .tenant_service(tenant_svc)
      .db_service(db_service)
      .build()
      .await?;
    let svc = make_auth_scoped_tenant(Arc::new(app_service));
    assert_eq!(expected, standalone_app_status_or_default(&svc).await?);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_app_status_or_default_not_found(
    #[future] test_db_service: TestDbService,
  ) -> anyhow::Result<()> {
    let db_service = Arc::new(test_db_service);
    let tenant_svc: Arc<dyn TenantService> =
      Arc::new(DefaultTenantService::new(db_service.clone()));
    let app_service = services::test_utils::AppServiceStubBuilder::default()
      .tenant_service(tenant_svc)
      .db_service(db_service)
      .build()
      .await?;
    let svc = make_auth_scoped_tenant(Arc::new(app_service));
    assert_eq!(
      AppStatus::default(),
      standalone_app_status_or_default(&svc).await?
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_app_status_or_default_propagates_encryption_error() -> anyhow::Result<()> {
    let mut mock_tenant_svc = services::MockTenantService::new();
    mock_tenant_svc.expect_get_standalone_app().returning(|| {
      Err(services::TenantError::from(
        services::DbError::EncryptionError("Decryption failed.".into()),
      ))
    });
    let app_service = services::test_utils::AppServiceStubBuilder::default()
      .tenant_service(Arc::new(mock_tenant_svc) as Arc<dyn TenantService>)
      .build()
      .await?;
    let svc = make_auth_scoped_tenant(Arc::new(app_service));
    let result = standalone_app_status_or_default(&svc).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code(), "db_error-encryption_error");
    Ok(())
  }
}
