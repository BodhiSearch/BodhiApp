use crate::middleware::auth::AuthError;
use crate::middleware::token_service::DefaultTokenService;
use anyhow_trace::anyhow_trace;
use chrono::{Duration, Utc};
use mockall::predicate::*;
use rstest::rstest;
use serde_json::json;
use services::AuthContext;
use services::{
  test_utils::{
    build_token, test_db_service, AppServiceStubBuilder, SettingServiceStub, TestDbService, ISSUER,
    TEST_CLIENT_ID, TEST_CLIENT_SECRET, TEST_TENANT_ID,
  },
  AppService, AppStatus, AuthServiceError, CacheService, DefaultTenantService,
  LocalConcurrencyService, MockAuthService, MockSettingService, MokaCacheService, Tenant,
  TenantRepository, TenantService, TokenError, TOKEN_TYPE_OFFLINE,
  {AppAccessRequestStatus, TokenEntity, TokenRepository, TokenStatus},
};
use services::{McpGrant, ModelGrant, TokenGrants, TokenGrantsV1, TokenScope, UserScope};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

fn create_token_digest(bearer_token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bearer_token.as_bytes());
  format!("{:x}", hasher.finalize())[0..32].to_string()
}

#[rstest]
#[case::user("scope_token_user", TokenScope::User)]
#[case::power_user("scope_token_power_user", TokenScope::PowerUser)]
#[awt]
#[tokio::test]
async fn test_validate_bodhiapp_token_scope_variations(
  #[case] scope_str: &str,
  #[case] expected_scope: TokenScope,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let token_str = "bodhiapp_test1234.test-client";
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  let mut api_token = TokenEntity {
    id: Uuid::new_v4().to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: scope_str.to_string(),
    status: TokenStatus::Active,
    grants: services::default_grants_json(),
    last_used_at: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service
    .create_api_token(TEST_TENANT_ID, &mut api_token)
    .await?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", token_str))
    .await?;

  match result {
    AuthContext::ApiToken {
      user_id,
      role,
      token,
      ..
    } => {
      assert_eq!("test-user", user_id);
      assert_eq!(expected_scope, role);
      assert_eq!(token_str, token);
    }
    _ => panic!("Expected ApiToken"),
  }
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_validate_bodhiapp_token_parses_grants(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let token_str = "bodhiapp_grant123.test-client";
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  let grants = TokenGrants::V1(TokenGrantsV1 {
    models_list: false,
    models: ModelGrant::Specific {
      ids: vec!["m1".to_string()],
    },
    mcps_list: true,
    mcps: McpGrant::Specific { ids: vec![] },
  });

  let mut api_token = TokenEntity {
    id: Uuid::new_v4().to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    grants: serde_json::to_string(&grants)?,
    last_used_at: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service
    .create_api_token(TEST_TENANT_ID, &mut api_token)
    .await?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", token_str))
    .await?;
  match result {
    AuthContext::ApiToken { grants: parsed, .. } => assert_eq!(grants, parsed),
    _ => panic!("Expected ApiToken"),
  }
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_validate_bodhiapp_token_rejects_malformed_grants(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let token_str = "bodhiapp_badgrnt.test-client";
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];
  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  let mut api_token = TokenEntity {
    id: Uuid::new_v4().to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    grants: "{ not valid json".to_string(),
    last_used_at: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service
    .create_api_token(TEST_TENANT_ID, &mut api_token)
    .await?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  // Fail closed: a corrupt grants payload rejects the token.
  let err = token_service
    .validate_bearer_token(&format!("Bearer {}", token_str))
    .await
    .unwrap_err();
  assert!(matches!(err, AuthError::Token(TokenError::InvalidToken(_))));
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bodhiapp_token_success(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let token_str = "bodhiapp_test1234.test-client";
  // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  let mut api_token = TokenEntity {
    id: Uuid::new_v4().to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    grants: services::default_grants_json(),
    last_used_at: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service
    .create_api_token(TEST_TENANT_ID, &mut api_token)
    .await?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", token_str))
    .await;

  assert!(result.is_ok());
  match result.unwrap() {
    AuthContext::ApiToken {
      user_id,
      role,
      token,
      ..
    } => {
      assert_eq!("test-user", user_id);
      assert_eq!(TokenScope::User, role);
      assert_eq!(token_str, token);
    }
    _ => panic!("Expected ApiToken"),
  }
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bodhiapp_token_inactive(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  let token_str = "bodhiapp_test1234.test-client";
  // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
  let token_prefix = &token_str[.."bodhiapp_".len() + 8];

  let mut hasher = Sha256::new();
  hasher.update(token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  let mut api_token = TokenEntity {
    id: Uuid::new_v4().to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Inactive,
    grants: services::default_grants_json(),
    last_used_at: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service
    .create_api_token(TEST_TENANT_ID, &mut api_token)
    .await?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

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
  let stored_token_str = "bodhiapp_test1234abc.test-client";
  let different_token_str = "bodhiapp_test1234xyz.test-client";
  // token_prefix is first 9 chars ("bodhiapp_") + next 8 chars = 17 chars total
  let token_prefix = &stored_token_str[.."bodhiapp_".len() + 8];

  let mut hasher = Sha256::new();
  hasher.update(stored_token_str.as_bytes());
  let token_hash = format!("{:x}", hasher.finalize());

  let mut api_token = TokenEntity {
    id: Uuid::new_v4().to_string(),
    tenant_id: TEST_TENANT_ID.to_string(),
    user_id: "test-user".to_string(),
    name: "Test Token".to_string(),
    token_prefix: token_prefix.to_string(),
    token_hash,
    scopes: "scope_token_user".to_string(),
    status: TokenStatus::Active,
    grants: services::default_grants_json(),
    last_used_at: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  };
  test_db_service
    .create_api_token(TEST_TENANT_ID, &mut api_token)
    .await?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  // Validate with a different token string than the one whose hash was stored
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
  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let token_service = Arc::new(DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(MockSettingService::default()),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
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
  // Token from a different client but same issuer
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
    "scope": "openid email profile roles",
    "sid": Uuid::new_v4().to_string(),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let (exchanged_token, _) = build_token(
    json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "jti": "test-jti", "sub": sub, "exp": Utc::now().timestamp() + 3600, "scope": "openid email profile roles"}},
  )?;
  let exchanged_token_cl = exchanged_token.clone();

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let mut mock_auth = MockAuthService::new();

  mock_auth
    .expect_exchange_app_token()
    .with(
      eq(TEST_CLIENT_ID),
      eq(TEST_CLIENT_SECRET),
      eq(external_token.clone()),
      eq(
        ["openid", "email", "profile", "roles"]
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
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  ));

  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await?;

  match result {
    AuthContext::ExternalApp {
      user_id,
      role,
      token,
      app_client_id,
      ..
    } => {
      assert_eq!(sub, user_id);
      assert_eq!(None, role);
      assert_eq!(exchanged_token, token);
      assert_eq!(external_client_id, app_client_id);
    }
    _ => panic!("Expected ExternalApp"),
  }
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_external_client_token_cache_security_prevents_jti_forgery(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
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
    "scope": "openid email profile roles",
    "sid": Uuid::new_v4().to_string(),
  });
  let (legitimate_token, _) = build_token(legitimate_token_claims)?;

  let forged_token_claims = json!({
    "exp": (Utc::now() + Duration::hours(1)).timestamp(),
    "iat": Utc::now().timestamp(),
    "jti": jti.clone(),
    "iss": ISSUER,
    "sub": "malicious-user",
    "typ": TOKEN_TYPE_OFFLINE,
    "azp": external_client_id,
    "aud": TEST_CLIENT_ID,
    "session_state": Uuid::new_v4().to_string(),
    "scope": "openid email profile roles",
    "sid": Uuid::new_v4().to_string(),
  });
  let (forged_token, _) = build_token(forged_token_claims)?;

  let (legitimate_exchanged_token, _) = build_token(
    json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "jti": "legitimate-jti", "sub": sub, "exp": Utc::now().timestamp() + 3600, "scope": "openid email profile roles"}},
  )?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let mut mock_auth = MockAuthService::new();
  let cache_service = Arc::new(MokaCacheService::default());

  mock_auth
    .expect_exchange_app_token()
    .with(
      eq(TEST_CLIENT_ID),
      eq(TEST_CLIENT_SECRET),
      eq(legitimate_token.clone()),
      eq(
        ["openid", "email", "profile", "roles"]
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

  mock_auth
    .expect_exchange_app_token()
    .with(
      eq(TEST_CLIENT_ID),
      eq(TEST_CLIENT_SECRET),
      eq(forged_token.clone()),
      eq(
        ["openid", "email", "profile", "roles"]
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
    tenant_svc,
    cache_service.clone(),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  ));

  let legitimate_result = token_service
    .validate_bearer_token(&format!("Bearer {}", legitimate_token))
    .await?;

  match legitimate_result {
    AuthContext::ExternalApp {
      user_id,
      role,
      token,
      app_client_id,
      ..
    } => {
      assert_eq!(sub, user_id);
      assert_eq!(None, role);
      assert_eq!(legitimate_exchanged_token, token);
      assert_eq!(external_client_id, app_client_id);
    }
    _ => panic!("Expected ExternalApp"),
  }

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
    "scope": "openid email profile roles scope_access_request:nonexistent",
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
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

// A bearer token whose access request is not Approved — Draft, Denied, or
// Revoked (inactive) — must be rejected at token validation, on every surface.
#[anyhow_trace]
#[rstest]
#[case::draft(AppAccessRequestStatus::Draft)]
#[case::denied(AppAccessRequestStatus::Denied)]
#[case::revoked(AppAccessRequestStatus::Revoked)]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_scope_not_approved(
  #[future] test_db_service: TestDbService,
  #[case] status: AppAccessRequestStatus,
) -> anyhow::Result<()> {
  use services::{AccessRequestRepository, AppAccessRequest};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:draft-test";

  let row = AppAccessRequest {
    id: "ar-draft".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    user_id: None,
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: Some(scope.to_string()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

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
  use services::{AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:app-mismatch-test";
  let sub = Uuid::new_v4().to_string();

  let row = AppAccessRequest {
    id: "ar-mismatch".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "app2".to_string(), // Different from token azp
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    user_id: Some(sub.clone()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );
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
  use services::{AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:user-mismatch-test";

  let row = AppAccessRequest {
    id: "ar-user-mismatch".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    user_id: Some("user2".to_string()), // Different from token sub
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
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
#[case::denied(AppAccessRequestStatus::Denied, "denied")]
#[case::draft(AppAccessRequestStatus::Draft, "draft")]
#[case::failed(AppAccessRequestStatus::Failed, "failed")]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_invalid_status(
  #[case] status: AppAccessRequestStatus,
  #[case] status_label: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::{AccessRequestRepository, AppAccessRequest};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = format!("scope_access_request:status-{}-test", status_label);
  let sub = Uuid::new_v4().to_string();

  let row = AppAccessRequest {
    id: format!("ar-{}", status_label),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: None,
    user_id: Some(sub.clone()),
    requested_role: "scope_user_user".to_string(),
    approved_role: None,
    access_request_scope: Some(scope.clone()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(MockAuthService::default()),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
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
  use services::{AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:mismatch-test";
  let sub = Uuid::new_v4().to_string();
  let record_id = "ar-correct-id";

  let row = AppAccessRequest {
    id: record_id.to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    user_id: Some(sub.clone()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": scope,
    "access_request_id": "wrong-id"
  }))?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
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
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
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
  use services::{AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus};

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:missing-claim-test";
  let sub = Uuid::new_v4().to_string();

  let row = AppAccessRequest {
    id: "ar-missing-claim".to_string(),
    tenant_id: Some(TEST_TENANT_ID.to_string()),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    user_id: Some(sub.clone()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": scope,
  }))?;

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
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
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
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
  use services::{AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus};

  let tenant_row = test_db_service
    .create_tenant(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "Test App",
      None,
      &AppStatus::Ready,
      Some("test-user".to_string()),
    )
    .await?;
  let actual_tenant_id = tenant_row.id.clone();

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:success-test";
  let sub = Uuid::new_v4().to_string();
  let sub_cl = sub.clone();
  let record_id = "ar-success";
  let row = AppAccessRequest {
    id: record_id.to_string(),
    tenant_id: Some(actual_tenant_id.clone()),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(
      r#"{"version":"1","models_list":true,"models_access":{"type":"specific","ids":["alias-x"]}}"#
        .to_string(),
    ),
    user_id: Some(sub.clone()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": scope,
    "access_request_id": record_id
  }))?;
  let exchanged_token_cl = exchanged_token.clone();

  // Use test_db_service for tenant_svc so the tenant IDs align
  let db_arc: Arc<dyn services::db::DbService> = Arc::new(test_db_service);
  let tenant_svc: Arc<dyn TenantService> = Arc::new(DefaultTenantService::new(Arc::clone(&db_arc)));
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
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    db_arc,
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_ok());
  match result.unwrap() {
    AuthContext::ExternalApp {
      user_id,
      role,
      token,
      app_client_id,
      access_request_id,
      grants,
      ..
    } => {
      assert_eq!(sub_cl, user_id);
      assert_eq!(Some(UserScope::User), role);
      assert_eq!(exchanged_token, token);
      assert_eq!("external-client", app_client_id);
      assert_eq!(Some(record_id.to_string()), access_request_id);
      // Approved grants are resolved from the access-request row and ride on the context.
      let grants = grants.expect("approved grants resolved");
      assert!(grants.v1().models_list);
      assert_eq!(
        services::ModelGrant::Specific {
          ids: vec!["alias-x".to_string()]
        },
        grants.v1().models_access
      );
    }
    _ => panic!("Expected ExternalApp"),
  }
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_cache_hit_returns_role(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::{AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus};

  let tenant_row = test_db_service
    .create_tenant(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "Test App",
      None,
      &AppStatus::Ready,
      Some("test-user".to_string()),
    )
    .await?;
  let actual_tenant_id = tenant_row.id.clone();

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:cache-role-test";
  let sub = Uuid::new_v4().to_string();
  let sub_cl = sub.clone();
  let record_id = "ar-cache-role";
  let row = AppAccessRequest {
    id: record_id.to_string(),
    tenant_id: Some(actual_tenant_id.clone()),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    user_id: Some(sub.clone()),
    requested_role: "scope_user_user".to_string(),
    approved_role: Some("scope_user_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": scope,
    "access_request_id": record_id
  }))?;
  let exchanged_token_cl = exchanged_token.clone();

  // Use test_db_service for tenant_svc so the tenant IDs align
  let db_arc: Arc<dyn services::db::DbService> = Arc::new(test_db_service);
  let tenant_svc: Arc<dyn TenantService> = Arc::new(DefaultTenantService::new(Arc::clone(&db_arc)));
  let mut mock_auth = MockAuthService::new();
  // exchange_app_token should only be called once; second call uses cache
  mock_auth
    .expect_exchange_app_token()
    .times(1)
    .return_once(move |_, _, _, _| Ok((exchanged_token_cl.clone(), None)));

  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(mock_auth),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    db_arc,
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  let bearer_header = format!("Bearer {}", external_token);

  // First call - hits exchange_app_token
  let result1 = token_service.validate_bearer_token(&bearer_header).await?;
  match &result1 {
    AuthContext::ExternalApp { role, .. } => {
      assert_eq!(&Some(UserScope::User), role);
    }
    _ => panic!("Expected ExternalApp on first call"),
  }

  // Second call - should return from cache with role: Some(UserScope::User)
  let result2 = token_service.validate_bearer_token(&bearer_header).await?;
  match result2 {
    AuthContext::ExternalApp {
      user_id,
      role,
      token,
      app_client_id,
      access_request_id,
      ..
    } => {
      assert_eq!(sub_cl, user_id);
      assert_eq!(Some(UserScope::User), role);
      assert_eq!(exchanged_token, token);
      assert_eq!("external-client", app_client_id);
      assert_eq!(Some(record_id.to_string()), access_request_id);
    }
    _ => panic!("Expected ExternalApp on cache hit"),
  }
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_without_access_request_scope(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
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
    "scope": "openid email profile roles",
  });
  let (external_token, _) = build_token(external_token_claims)?;

  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub.clone(),
    "exp": Utc::now().timestamp() + 3600,
    "scope": "openid email profile roles",
  }))?;
  let exchanged_token_cl = exchanged_token.clone();

  let tenant_svc = AppServiceStubBuilder::default()
    .with_tenant(Tenant::test_default())
    .await
    .build()
    .await?
    .tenant_service();
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
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    Arc::new(test_db_service),
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_ok());
  match result.unwrap() {
    AuthContext::ExternalApp {
      user_id,
      role,
      token,
      app_client_id,
      ..
    } => {
      assert_eq!(sub, user_id);
      assert_eq!(None, role);
      assert_eq!(exchanged_token, token);
      assert_eq!("external-client", app_client_id);
    }
    _ => panic!("Expected ExternalApp"),
  }
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[awt]
#[tokio::test]
async fn test_validate_bearer_token_privilege_escalation_rejected(
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  use services::{AccessRequestRepository, AppAccessRequest, AppAccessRequestStatus};

  let tenant_row = test_db_service
    .create_tenant(
      TEST_CLIENT_ID,
      TEST_CLIENT_SECRET,
      "Test App",
      None,
      &AppStatus::Ready,
      Some("test-user".to_string()),
    )
    .await?;
  let actual_tenant_id = tenant_row.id.clone();

  let now = test_db_service.now();
  let expires_at = now + chrono::Duration::hours(1);
  let scope = "scope_access_request:escalation-test";
  let sub = Uuid::new_v4().to_string();
  let record_id = "ar-escalation";

  // DB record has approved_role = scope_user_power_user (tampered or misconfigured)
  let row = AppAccessRequest {
    id: record_id.to_string(),
    tenant_id: Some(actual_tenant_id.clone()),
    app_client_id: "external-client".to_string(),
    app_name: Some("Test App".to_string()),
    app_description: None,
    status: AppAccessRequestStatus::Approved,
    requested: r#"{"version":"1"}"#.to_string(),
    approved: Some(r#"{"version":"1"}"#.to_string()),
    user_id: Some(sub.clone()),
    requested_role: "scope_user_power_user".to_string(),
    approved_role: Some("scope_user_power_user".to_string()),
    access_request_scope: Some(scope.to_string()),
    source_access_request_id: None,
    error_message: None,
    expires_at,
    created_at: now,
    updated_at: now,
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
    "scope": format!("openid email profile roles {}", scope),
  });
  let (external_token, _) = build_token(external_token_claims)?;

  // Exchanged token has resource_access with only resource_user role
  // This means max_user_scope() = UserScope::User, but approved_role = PowerUser → escalation
  let (exchanged_token, _) = build_token(json!({
    "iss": ISSUER,
    "azp": TEST_CLIENT_ID,
    "jti": "test-jti",
    "sub": sub,
    "exp": Utc::now().timestamp() + 3600,
    "scope": scope,
    "access_request_id": record_id,
    "resource_access": {
      TEST_CLIENT_ID: {
        "roles": ["resource_user"]
      }
    }
  }))?;

  // Use test_db_service for tenant_svc so the tenant IDs align
  let db_arc: Arc<dyn services::db::DbService> = Arc::new(test_db_service);
  let tenant_svc: Arc<dyn TenantService> = Arc::new(DefaultTenantService::new(Arc::clone(&db_arc)));
  let mut mock_auth = MockAuthService::new();
  mock_auth
    .expect_exchange_app_token()
    .return_once(move |_, _, _, _| Ok((exchanged_token, None)));

  let mut setting_service = MockSettingService::default();
  setting_service
    .expect_auth_issuer()
    .return_once(|| ISSUER.to_string());

  let token_service = DefaultTokenService::new(
    Arc::new(mock_auth),
    tenant_svc,
    Arc::new(MokaCacheService::default()),
    db_arc,
    Arc::new(setting_service),
    Arc::new(LocalConcurrencyService::new()),
    Arc::new(services::DefaultTimeService),
  );

  let result = token_service
    .validate_bearer_token(&format!("Bearer {}", external_token))
    .await;

  assert!(result.is_err());
  match result.unwrap_err() {
    AuthError::Token(TokenError::AccessRequestValidation(
      services::AccessRequestValidationError::PrivilegeEscalation {
        approved_role,
        max_scope,
      },
    )) => {
      assert_eq!("scope_user_power_user", approved_role);
      assert_eq!("scope_user_user", max_scope);
    }
    other => panic!("Expected PrivilegeEscalation, got: {:?}", other),
  }
  Ok(())
}

/// Couples `access_request_cache_needle` to the real serialized
/// `CachedExchangeResult`. If the cached JSON shape drifts (field rename,
/// `skip_serializing_if`, different envelope), the eviction needle stops
/// matching and a revoked app token keeps working until the 5-min TTL — this
/// fails loudly instead.
#[test]
fn test_access_request_cache_needle_matches_serialized_cached_exchange_result() {
  use crate::middleware::token_service::{access_request_cache_needle, CachedExchangeResult};
  use pretty_assertions::assert_eq;

  let result = CachedExchangeResult {
    token: "exchanged-token".to_string(),
    client_id: "client-id".to_string(),
    tenant_id: "tenant-id".to_string(),
    app_client_id: "app-client-id".to_string(),
    role: None,
    access_request_id: Some("ar-123".to_string()),
    grants: None,
    cached_at: 0,
  };

  let serialized = serde_json::to_string(&result).expect("serialize CachedExchangeResult");
  let needle = access_request_cache_needle("ar-123");

  assert!(
    serialized.contains(&needle),
    "eviction needle `{needle}` must be a substring of cached JSON `{serialized}`"
  );

  let other_needle = access_request_cache_needle("ar-999");
  assert!(
    !serialized.contains(&other_needle),
    "needle for a different access_request_id must NOT match — eviction must be targeted"
  );

  assert_eq!(r#""access_request_id":"ar-123""#, needle);
}
