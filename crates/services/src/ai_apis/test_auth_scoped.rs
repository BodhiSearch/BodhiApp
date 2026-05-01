use crate::ai_apis::ai_api_client_factory::LibertySource;
use crate::ai_apis::auth_scoped::AuthScopedAiApiClientFactory;
use crate::ai_apis::error::AiApiClientFactoryError;
use crate::auth::AuthContextError;
use crate::test_utils::{
  test_llm_liberty_envelope, test_resolved_llm_liberty_credentials, AppServiceStubBuilder,
  TEST_TENANT_ID, TEST_USER_ID,
};
use crate::{
  ApiAlias, ApiFormat, AppService, AuthContext, DeploymentMode, MockAiApiClientFactory,
  ResourceRole, SafeReqwest,
};
use anyhow_trace::anyhow_trace;
use rstest::rstest;
use std::sync::Arc;

fn safe_http() -> SafeReqwest {
  SafeReqwest::builder()
    .allow_private_ips()
    .build()
    .expect("safe reqwest builder")
}

fn liberty_alias() -> ApiAlias {
  ApiAlias::new(
    "alias-liberty",
    ApiFormat::LlmLibertyOauth,
    "https://api.example.com",
    Vec::<crate::ApiModel>::new(),
    None,
    false,
    chrono::Utc::now(),
    None,
    None,
  )
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn for_resolved_injects_tenant_and_user_from_session_context() -> anyhow::Result<()> {
  let mut mock = MockAiApiClientFactory::new();
  mock
    .expect_for_liberty()
    .withf(|source| match source {
      LibertySource::Resolved {
        tenant_id, user_id, ..
      } => *tenant_id == TEST_TENANT_ID && *user_id == TEST_USER_ID,
      LibertySource::Envelope(_) => false,
    })
    .times(1)
    .returning(|_| {
      // Returning an arbitrary error short-circuits without needing to construct
      // a full AiApiClient mock; we only assert the injected identifiers above.
      Err(AiApiClientFactoryError::ApiError("ok".to_string()))
    });
  mock.expect_safe_http_client().returning(safe_http);

  let app_service = AppServiceStubBuilder::default()
    .ai_api_client_factory(Arc::new(mock))
    .build()
    .await?;
  let app_service: Arc<dyn AppService> = Arc::new(app_service);
  let auth_context = AuthContext::test_session(TEST_USER_ID, "user", ResourceRole::User);

  let factory = AuthScopedAiApiClientFactory::new(app_service, auth_context);
  let creds = test_resolved_llm_liberty_credentials();
  let result = factory.for_resolved(&creds, &liberty_alias());

  match result {
    Err(AiApiClientFactoryError::ApiError(msg)) => assert_eq!("ok", msg),
    other => panic!(
      "expected mock pass-through error, got: {:?}",
      other.is_err()
    ),
  }
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn for_resolved_propagates_anonymous_auth_as_typed_error() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default().build().await?;
  let app_service: Arc<dyn AppService> = Arc::new(app_service);
  let auth_context = AuthContext::test_anonymous(DeploymentMode::Standalone);

  let factory = AuthScopedAiApiClientFactory::new(app_service, auth_context);
  let creds = test_resolved_llm_liberty_credentials();
  let result = factory.for_resolved(&creds, &liberty_alias());

  match result {
    Err(AiApiClientFactoryError::Auth(AuthContextError::MissingTenantId)) => {}
    other => panic!(
      "expected AuthContextError::MissingTenantId, got: {:?}",
      other.is_err()
    ),
  }
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn for_envelope_delegates_unchanged() -> anyhow::Result<()> {
  let mut mock = MockAiApiClientFactory::new();
  mock
    .expect_for_liberty()
    .withf(|source| matches!(source, LibertySource::Envelope(_)))
    .times(1)
    .returning(|_| Err(AiApiClientFactoryError::ApiError("delegated".to_string())));
  mock.expect_safe_http_client().returning(safe_http);

  let app_service = AppServiceStubBuilder::default()
    .ai_api_client_factory(Arc::new(mock))
    .build()
    .await?;
  let app_service: Arc<dyn AppService> = Arc::new(app_service);
  let auth_context = AuthContext::test_anonymous(DeploymentMode::Standalone);
  let factory = AuthScopedAiApiClientFactory::new(app_service, auth_context);

  let envelope = test_llm_liberty_envelope();
  let result = factory.for_envelope(&envelope);
  match result {
    Err(AiApiClientFactoryError::ApiError(msg)) => assert_eq!("delegated", msg),
    other => panic!(
      "expected pass-through delegation, got: {:?}",
      other.is_err()
    ),
  }
  Ok(())
}
