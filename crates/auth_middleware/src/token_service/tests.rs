use crate::{AuthError, DefaultTokenService, ResourceScope};
use anyhow_trace::anyhow_trace;
use chrono::{Duration, Utc};
use mockall::predicate::*;
use objs::{TokenScope, UserScope};
use rstest::rstest;
use serde_json::json;
use services::{
  db::{ApiToken, TokenRepository, TokenStatus},
  test_utils::{
    build_token, test_db_service, AppServiceStubBuilder, SettingServiceStub, TestDbService, ISSUER,
    TEST_CLIENT_ID, TEST_CLIENT_SECRET,
  },
  AppInstance, AppInstanceService, AppService, AuthServiceError, CacheService,
  LocalConcurrencyService, MockAuthService, MockSettingService, MokaCacheService,
  TOKEN_TYPE_OFFLINE,
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

fn create_token_digest(bearer_token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bearer_token.as_bytes());
  format!("{:x}", hasher.finalize())[0..12].to_string()
}

#[rstest]
#[case::user("scope_token_user", TokenScope::User)]
#[case::power_user("scope_token_power_user", TokenScope::PowerUser)]
#[case::manager("scope_token_manager", TokenScope::Manager)]
#[case::admin("scope_token_admin", TokenScope::Admin)]
#[awt]
#[tokio::test]
async fn test_validate_bodhiapp_token_scope_variations(
  #[case] scope_str: &str,
  #[case] expected_scope: TokenScope,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // Setup test database with token
  let token_str = "bodhiapp_test12345678901234567890123456789012";
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Hash the token
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // Create ApiToken in database with specified scope
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: scope_str.to_string(),
    status: TokenStatus::Active,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service.create_api_token(&mut api_token).await?;

  // Create token service
  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance_service()
    .await
    .build()
    .await?
    .app_instance_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Validate token
  let (access_token, scope, app_client_id) = token_service
    .validate_bearer_token(&format!("Bearer {}", token_str))
    .await?;

  assert_eq!(token_str, access_token);
  assert_eq!(ResourceScope::Token(expected_scope), scope);
  assert_eq!(None, app_client_id);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bodhiapp_token_success(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // Setup test database with token
  let token_str = "bodhiapp_test12345678901234567890123456789012";
  // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Hash the token
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // Create ApiToken in database
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service.create_api_token(&mut api_token).await?;

  // Create token service
  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance_service()
    .await
    .build()
    .await?
    .app_instance_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Validate token
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", token_str))
    .await;

  assert!(result.is_ok());
  let (access_token, scope, app_client_id) = result.unwrap();
  assert_eq!(token_str, access_token);
  assert_eq!(ResourceScope::Token(TokenScope::User), scope);
  assert_eq!(None, app_client_id);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bodhiapp_token_inactive(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // Setup test database with inactive token
  let token_str = "bodhiapp_test12345678901234567890123456789012";
  // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  // Hash the token
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // Create ApiToken in database with Inactive status
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Inactive,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service.create_api_token(&mut api_token).await?;

  // Create token service
  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance_service()
    .await
    .build()
    .await?
    .app_instance_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Validate token - should fail due to inactive status
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", token_str))
    .await;

  assert!(result.is_err());
  assert!(matches!(result, Err(AuthError::TokenInactive)));
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bodhiapp_token_invalid_hash(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // Setup test database with token
  let stored_token_str = "bodhiapp_test12345678901234567890123456789012";
  let different_token_str = "bodhiapp_test12399999999999999999999999999999";
  // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
  let token_prefix = &stored_token_str[.."bodhiapp_".len() + 8];

  // Hash the stored token
  let mut hasher = Sha256::new();
  hasher.update(stored_token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  // Create ApiToken in database
  let mut api_token = ApiToken {
    id: Uuid::new_v4().to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service.create_api_token(&mut api_token).await?;

  // Create token service
  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance_service()
    .await
    .build()
    .await?
    .app_instance_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Try to validate with different token string (wrong hash)
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", different_token_str))
    .await;

  assert!(result.is_err());
  assert!(matches!(result, Err(AuthError::Token(_))));
  Ok(())
}

#[rstest]
#[case::empty("")]
#[case::malformed("bearer foobar")]
#[case::empty_bearer("Bearer ")]
#[case::empty_bearer_2("Bearer  ")]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_header_errors(
  #[case] header: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance_service()
    .await
    .build()
    .await?
    .app_instance_service();
  let token_service = Arc::new(DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
  ));
  let result = token_service.validate_bearer_token(header).await;
  assert!(result.is_err());
  assert!(matches!(result, Err(AuthError::Token(_))));
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_external_client_token_success(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // Given - Create a token from a different client but same issuer
  let external_client_id = "external-client";
  let sub = Uuid::new_v4().to_string();
  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER, // Same issuer as our app
    "sub": sub,
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": external_client_id, // Different client
    "aud": TEST_CLIENT_ID, // Audience is our client
    "session_state": Uuid::new_v4().to_string(),
    "scope": "openid scope_user_user",
    "sid": Uuid::new_v4().to_string(),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  // Setup mock auth service to return exchanged token
  let (exchanged_token, _) = build_token(
    json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "jti": "test-jti", "sub": sub, "exp": Utc::now().timestamp() + 3600, "scope": "scope_user_user"}},
  )?;
  let exchanged_token_cl = exchanged_token.clone();

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut mock_auth = MockAuthService::new();

  // Expect token exchange to be called
  mock_auth
    .expect_exchange_app_token()
    .with(
      eq(TEST_CLIENT_ID),
      eq(TEST_CLIENT_SECRET),
      eq(external_token.clone()),
      eq(
        ["scope_user_user", "openid", "email", "profile", "roles"]
          .iter()
          .map(|s| s.to_string())
          .collect::<Vec<String>>(),
      ),
    )
    .times(1)
    .return_once(|_, _, _, _| Ok((exchanged_token_cl, None)));
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = Arc::new(DefaultTokenService::new(
    Arc::new(mock_auth),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  ));

  // When - Try to validate the external token
  let (access_token, scope, app_client_id) = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await?;

  // Then - Should succeed with exchanged token
  assert_eq!(exchanged_token, access_token);
  assert_eq!(ResourceScope::User(Some(UserScope::User)), scope);
  assert_eq!(Some(external_client_id.to_string()), app_client_id);
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_external_client_token_cache_security_prevents_jti_forgery(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // Given - Create a legitimate external token from a different client
  let external_client_id = "external-client";
  let sub = Uuid::new_v4().to_string();
  let jti = Uuid::new_v4().to_string();
  let legitimate_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": jti.clone(),
    "iss": ISSUER,
    "sub": sub.clone(),
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": external_client_id,
    "aud": TEST_CLIENT_ID,
    "session_state": Uuid::new_v4().to_string(),
    "scope": "openid scope_user_user",
    "sid": Uuid::new_v4().to_string(),
  });
  let (legitimate_token, _) = build_token(legitimate_token_claims)?;

  // Create a forged token with the same JTI but different content
  let forged_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": jti.clone(), // Same JTI as legitimate token
    "iss": ISSUER,
    "sub": "malicious-user", // Different subject
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": external_client_id,
    "aud": TEST_CLIENT_ID,
    "session_state": Uuid::new_v4().to_string(),
    "scope": "openid scope_user_admin", // Different scope - trying to escalate
    "sid": Uuid::new_v4().to_string(),
  });
  let (forged_token, _) = build_token(forged_token_claims)?;

  // Setup mock auth service - legitimate token succeeds, forged token fails
  let (legitimate_exchanged_token, _) = build_token(
    json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "jti": "legitimate-jti", "sub": sub, "exp": Utc::now().timestamp() + 3600, "scope": "scope_user_user"}},
  )?;

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut mock_auth = MockAuthService::new();
  let cache_service = Arc::new(MokaCacheService::default());

  // Expect token exchange for legitimate token to succeed
  mock_auth
    .expect_exchange_app_token()
    .with(
      eq(TEST_CLIENT_ID),
      eq(TEST_CLIENT_SECRET),
      eq(legitimate_token.clone()),
      eq(
        ["scope_user_user", "openid", "email", "profile", "roles"]
          .iter()
          .map(|s| s.to_string())
          .collect::<Vec<String>>(),
      ),
    )
    .times(1)
    .return_once({
      let token = legitimate_exchanged_token.clone();
      move |_, _, _, _| Ok((token, None))
    });

  // Expect token exchange for forged token to fail with auth service error
  mock_auth
    .expect_exchange_app_token()
    .with(
      eq(TEST_CLIENT_ID),
      eq(TEST_CLIENT_SECRET),
      eq(forged_token.clone()),
      eq(
        ["scope_user_admin", "openid", "email", "profile", "roles"]
          .iter()
          .map(|s| s.to_string())
          .collect::<Vec<String>>(),
      ),
    )
    .times(1)
    .return_once(|_, _, _, _| {
      Err(AuthServiceError::TokenExchangeError(
        "forged token rejected".to_string(),
      ))
    });

  let setting_service = SettingServiceStub::with_settings(HashMap::from([
    (
      "BODHI_AUTH_URL".to_string(),
      "https://id.mydomain.com".to_string(),
    ),
    ("BODHI_AUTH_REALM".to_string(), "myapp".to_string()),
  ]));

  let token_service = Arc::new(DefaultTokenService::new(
    Arc::new(mock_auth),
    app_instance_svc,
    cache_service.clone(),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  ));

  // When - First validate the legitimate token (this will cache it)
  let (legitimate_access_token, legitimate_scope, legitimate_azp) = token_service
    .validate_bearer_token(&format!("Bearer {}", legitimate_token))
    .await?;

  // Then - Verify legitimate token works as expected
  assert_eq!(legitimate_exchanged_token, legitimate_access_token);
  assert_eq!(ResourceScope::User(Some(UserScope::User)), legitimate_scope);
  assert_eq!(Some(external_client_id.to_string()), legitimate_azp);

  // When - Try to validate the forged token with same JTI
  let forged_result = token_service
    .validate_bearer_token(&format!("Bearer {}", forged_token))
    .await;

  assert!(matches!(
    forged_result,
    Err(AuthError::AuthService(
      AuthServiceError::TokenExchangeError(_)
    ))
  ));
  let legitimate_digest = create_token_digest(&legitimate_token);
  let forged_digest = create_token_digest(&forged_token);
  assert_ne!(
    legitimate_digest, forged_digest,
    "Token digests should be different even with same JTI"
  );

  let cached_legitimate = cache_service.get(&format!("exchanged_token:{}", legitimate_digest));
  let cached_forged = cache_service.get(&format!("exchanged_token:{}", forged_digest));

  assert!(
    cached_legitimate.is_some(),
    "Legitimate token should be cached"
  );
  assert!(
    cached_forged.is_none(),
    "Forged token should not be cached due to validation failure"
  );

  Ok(())
}

// ============================================================================
// Phase 4b: Access Request Pre/Post-Exchange Validation Tests
// ============================================================================

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_scope_not_found(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // External token with scope_access_request:nonexistent
  let sub = Uuid::new_v4().to_string();
  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": sub,
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": "openid scope_user_user scope_access_request:nonexistent",
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // DB lookup returns None, expect 403 ScopeNotFound
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  // Error should be TokenError::AccessRequestValidation(ScopeNotFound)
  assert!(matches!(err, AuthError::Token(_)));
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_scope_not_approved(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::db::{AccessRequestRepository, AppAccessRequestRow};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:draft-test";

  // Create access request with status=draft
  let row = AppAccessRequestRow {
    id: "ar-draft".to_string(),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "draft".to_string(),
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: None,
    user_id: None,
    resource_scope: None,
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  test_db_service.create(&row).await?;

  let sub = Uuid::new_v4().to_string();
  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": sub,
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": format!("openid scope_user_user {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Expect 403 NotApproved
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::Token(_)));
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_app_client_mismatch(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::db::{AccessRequestRepository, AppAccessRequestRow};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:app-mismatch-test";
  let sub = Uuid::new_v4().to_string();

  // Create approved access request with app_client_id=app2
  let row = AppAccessRequestRow {
    id: "ar-mismatch".to_string(),
    app_client_id: "app2".to_string(), // Different from token azp
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: Some(r#"{"toolsets":[]}"#.to_string()),
    user_id: Some(sub.clone()),
    resource_scope: Some("scope_resource-xyz".to_string()),
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  test_db_service.create(&row).await?;

  // External token azp=external-client (different from record)
  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": sub,
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": format!("openid scope_user_user {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Expect 403 AppClientMismatch
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::Token(_)));
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_user_mismatch(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::db::{AccessRequestRepository, AppAccessRequestRow};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:user-mismatch-test";

  // Create approved access request with user_id=user2
  let row = AppAccessRequestRow {
    id: "ar-user-mismatch".to_string(),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: Some(r#"{"toolsets":[]}"#.to_string()),
    user_id: Some("user2".to_string()), // Different from token sub
    resource_scope: Some("scope_resource-xyz".to_string()),
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  test_db_service.create(&row).await?;

  // External token sub=user1 (different from record)
  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": "user1",
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": format!("openid scope_user_user {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Expect 403 UserMismatch
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::Token(_)));
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[case::denied("denied")]
#[case::expired("expired")]
#[case::failed("failed")]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_invalid_status(
  #[case] status: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::db::{AccessRequestRepository, AppAccessRequestRow};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = format!("scope_access_request:status-{}-test", status);
  let sub = Uuid::new_v4().to_string();

  // Create access request with invalid status
  let row = AppAccessRequestRow {
    id: format!("ar-{}", status),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: status.to_string(),
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: None,
    user_id: Some(sub.clone()),
    resource_scope: Some("scope_resource-xyz".to_string()),
    access_request_scope: Some(scope.clone()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  test_db_service.create(&row).await?;

  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": sub,
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": format!("openid scope_user_user {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Expect 403 NotApproved
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::Token(_)));
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_access_request_id_mismatch(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::db::{AccessRequestRepository, AppAccessRequestRow};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:mismatch-test";
  let sub = Uuid::new_v4().to_string();
  let record_id = "ar-correct-id";

  // Create approved access request
  let row = AppAccessRequestRow {
    id: record_id.to_string(),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: Some(r#"{"toolsets":[]}"#.to_string()),
    user_id: Some(sub.clone()),
    resource_scope: Some("scope_resource-xyz".to_string()),
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  test_db_service.create(&row).await?;

  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": sub.clone(),
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": format!("openid scope_user_user {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  // Mock token exchange to return token with WRONG access_request_id
  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": format!("scope_user_user {}", scope),
    "access_request_id": "wrong-id" // Different from record.id
  }))?;

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut mock_auth = MockAuthService::new();
  mock_auth
    .expect_exchange_app_token()
    .return_once(|_, _, _, _| Ok((exchanged_token, None)));

  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(mock_auth),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Expect 403 AccessRequestIdMismatch
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::Token(_)));
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_missing_access_request_id_claim(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::db::{AccessRequestRepository, AppAccessRequestRow};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:missing-claim-test";
  let sub = Uuid::new_v4().to_string();

  // Create approved access request
  let row = AppAccessRequestRow {
    id: "ar-missing-claim".to_string(),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: Some(r#"{"toolsets":[]}"#.to_string()),
    user_id: Some(sub.clone()),
    resource_scope: Some("scope_resource-xyz".to_string()),
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  test_db_service.create(&row).await?;

  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": sub.clone(),
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": format!("openid scope_user_user {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  // Mock token exchange to return token WITHOUT access_request_id claim
  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": format!("scope_user_user {}", scope),
    // NO access_request_id claim
  }))?;

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut mock_auth = MockAuthService::new();
  mock_auth
    .expect_exchange_app_token()
    .return_once(|_, _, _, _| Ok((exchanged_token, None)));

  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(mock_auth),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Expect 403 AccessRequestIdMismatch (claim="missing")
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::Token(_)));
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_with_access_request_scope_success(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::db::{AccessRequestRepository, AppAccessRequestRow};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:success-test";
  let sub = Uuid::new_v4().to_string();
  let record_id = "ar-success";

  // Create approved access request with matching details
  let row = AppAccessRequestRow {
    id: record_id.to_string(),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    flow_type: "redirect".to_string(),
    redirect_uri: Some("http://localhost:3000/callback".to_string()),
    status: "approved".to_string(),
    requested: r#"{"toolset_types":[]}"#.to_string(),
    approved: Some(r#"{"toolsets":[]}"#.to_string()),
    user_id: Some(sub.clone()),
    resource_scope: Some("scope_resource-xyz".to_string()),
    access_request_scope: Some(scope.to_string()),
    error_message: None,
    expires_at: expires_at.timestamp(),
    created_at: now.timestamp(),
    updated_at: now.timestamp(),
  };
  test_db_service.create(&row).await?;

  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": sub.clone(),
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": format!("openid scope_user_user {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  // Mock token exchange to return token with MATCHING access_request_id
  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": format!("scope_user_user {}", scope),
    "access_request_id": record_id // Matches record.id
  }))?;
  let exchanged_token_cl = exchanged_token.clone();

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut mock_auth = MockAuthService::new();
  mock_auth
    .expect_exchange_app_token()
    .return_once(move |_, _, _, _| Ok((exchanged_token_cl.clone(), None)));

  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(mock_auth),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Expect success
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_ok());
  let (access_token, scope, app_client_id) = result.unwrap();
  assert_eq!(exchanged_token, access_token);
  assert_eq!(ResourceScope::User(Some(UserScope::User)), scope);
  assert_eq!(Some("external-client".to_string()), app_client_id);
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_without_access_request_scope(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // External token with only scope_user_* (no scope_access_request:*)
  let sub = Uuid::new_v4().to_string();
  let external_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": Uuid::new_v4().to_string(),
    "iss": ISSUER,
    "sub": sub.clone(),
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": "external-client",
    "aud": TEST_CLIENT_ID,
    "scope": "openid scope_user_user", // No scope_access_request:*
  });
  let (external_token, _) = build_token(external_token_claims)?;

  // Mock token exchange
  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": "scope_user_user",
  }))?;
  let exchanged_token_cl = exchanged_token.clone();

  let app_instance_svc = AppServiceStubBuilder::default()
    .with_app_instance(AppInstance::test_default())
    .await
    .build()
    .await?
    .app_instance_service();
  let mut mock_auth = MockAuthService::new();
  mock_auth
    .expect_exchange_app_token()
    .return_once(move |_, _, _, _| Ok((exchanged_token_cl.clone(), None)));

  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(mock_auth),
    app_instance_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
  );

  // Expect success - validation skipped, token exchange proceeds normally
  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_ok());
  let (access_token, scope, app_client_id) = result.unwrap();
  assert_eq!(exchanged_token, access_token);
  assert_eq!(ResourceScope::User(Some(UserScope::User)), scope);
  assert_eq!(Some("external-client".to_string()), app_client_id);
  Ok(())
}
