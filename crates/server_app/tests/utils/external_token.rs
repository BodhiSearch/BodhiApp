#![allow(unused)]
use chrono::{Duration, Utc};
use routes_app::middleware::CachedExchangeResult;
use services::{
  test_utils::{approved_request, build_token, TEST_TENANT_ID},
  AppAccessRequest, AppService, CacheService, DbService,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

/// The `sub` claim minted into both the bearer and exchange JWTs.
pub const EXTERNAL_USER_ID: &str = "test-external-user";

/// Issuer matching the fake auth URL + realm in `setup_test_app_service`.
pub const TEST_AUTH_ISSUER: &str = "https://test-id.getbodhi.app/realms/bodhi";

/// Simulates external (3rd-party OAuth) token authentication by seeding the
/// token validation cache directly, bypassing Keycloak token exchange.
///
/// This works because:
/// 1. `extract_claims()` does NOT verify JWT signatures - it only base64-decodes
/// 2. The token service checks the cache before calling the auth server
/// 3. We can create valid JWTs with `build_token()` (RSA-signed test keys)
/// 4. The cache key is just SHA-256(bearer_token)[0..32]
pub struct ExternalTokenSimulator {
  cache_service: Arc<dyn CacheService>,
  db_service: Arc<dyn DbService>,
  client_id: String,
}

impl ExternalTokenSimulator {
  pub fn new(app_service: &Arc<dyn AppService>) -> Self {
    Self {
      cache_service: app_service.cache_service(),
      db_service: app_service.db_service(),
      client_id: "test-client-id".to_string(),
    }
  }

  pub fn new_with_client_id(app_service: &Arc<dyn AppService>, client_id: String) -> Self {
    Self {
      cache_service: app_service.cache_service(),
      db_service: app_service.db_service(),
      client_id,
    }
  }

  /// Creates a fake external bearer token and seeds the cache so requests
  /// with this token bypass Keycloak and resolve to the given role.
  ///
  /// Seeds `grants: None`, so the resolved `ExternalApp` principal is fail-closed
  /// (`AccessPolicy::Deny`). Delegates to `create_token_with_grants`.
  pub fn create_token_with_role(&self, role: Option<&str>, azp: &str) -> anyhow::Result<String> {
    self.create_token_with_grants(role, azp, None)
  }

  /// Like `create_token_with_role` but seeds the cached exchange result with the
  /// given approved `grants`, so the resolved `ExternalApp` principal flows through
  /// `AccessPolicy::Grants` and exercises real grant enforcement (model + MCP).
  ///
  /// The cached `access_request_id` is a random Uuid tied to NO DB row — only
  /// suitable for routes NOT behind `access_request_auth_middleware`.
  pub fn create_token_with_grants(
    &self,
    role: Option<&str>,
    azp: &str,
    grants: Option<services::ApprovedResources>,
  ) -> anyhow::Result<String> {
    let access_request_id = role.map(|_| Uuid::new_v4().to_string());
    self.seed_token(role, azp, grants, access_request_id)
  }

  /// Builds an Approved `app_access_requests` row consistent with a token this
  /// simulator would mint for `azp` (tenant `TEST_TENANT_ID`, user
  /// `EXTERNAL_USER_ID`, dotted `access_request_scope`, `approved` matching
  /// `grants`). Tests mutate fields for negative cases before inserting.
  pub fn approved_row(
    &self,
    azp: &str,
    grants: &Option<services::ApprovedResources>,
  ) -> anyhow::Result<AppAccessRequest> {
    let id = Uuid::new_v4().to_string();
    let approved_json = match grants {
      Some(g) => serde_json::to_string(g)?,
      None => r#"{"version":"1"}"#.to_string(),
    };
    Ok(AppAccessRequest {
      app_client_id: azp.to_string(),
      access_request_scope: Some(format!("scope_access_request:{}.{}", self.client_id, id)),
      approved: Some(approved_json),
      ..approved_request(&id, TEST_TENANT_ID, EXTERNAL_USER_ID, Utc::now())
    })
  }

  /// Inserts a REAL `app_access_requests` row and mints a token whose cached
  /// exchange result points at it (`access_request_id` = row id), so the
  /// per-request `access_request_auth_middleware` validation passes.
  ///
  /// Returns `(bearer_token, row_id)`.
  pub async fn create_token_with_backing_row(
    &self,
    role: Option<&str>,
    azp: &str,
    grants: Option<services::ApprovedResources>,
  ) -> anyhow::Result<(String, String)> {
    let row = self.approved_row(azp, &grants)?;
    self.create_token_for_row(role, azp, grants, row).await
  }

  /// Inserts the given `app_access_requests` row and mints a token bound to it.
  /// The row is caller-controlled so negative cases (Revoked status, mismatched
  /// app_client_id / user_id) can diverge from the token's cached identity.
  pub async fn create_token_for_row(
    &self,
    role: Option<&str>,
    azp: &str,
    grants: Option<services::ApprovedResources>,
    row: AppAccessRequest,
  ) -> anyhow::Result<(String, String)> {
    let inserted = self.db_service.create(&row).await?;
    let token = self.seed_token(role, azp, grants, Some(inserted.id.clone()))?;
    Ok((token, inserted.id))
  }

  /// Like `create_token_with_backing_row`, but the row lives under an arbitrary
  /// tenant and the bearer token carries full `ScopeClaims` (iss/aud/azp/scope
  /// with the dotted access-request scope). After cache eviction the token then
  /// survives `handle_external_client_token` up to the DB scope validation, so
  /// tests exercise the real post-revocation rejection path without Keycloak.
  pub async fn create_revalidatable_token(
    &self,
    role: Option<&str>,
    azp: &str,
    grants: Option<services::ApprovedResources>,
    tenant_id: &str,
    tenant_client_id: &str,
  ) -> anyhow::Result<(String, String)> {
    let id = Uuid::new_v4().to_string();
    let scope = format!("scope_access_request:{}.{}", tenant_client_id, id);
    let approved_json = match &grants {
      Some(g) => serde_json::to_string(g)?,
      None => r#"{"version":"1"}"#.to_string(),
    };
    let row = AppAccessRequest {
      app_client_id: azp.to_string(),
      access_request_scope: Some(scope.clone()),
      approved: Some(approved_json),
      ..approved_request(&id, tenant_id, EXTERNAL_USER_ID, Utc::now())
    };
    let inserted = self.db_service.create(&row).await?;

    let future_exp = (Utc::now() + Duration::hours(1)).timestamp() as u64;
    let bearer_claims = serde_json::json!({
      "jti": Uuid::new_v4().to_string(),
      "iss": TEST_AUTH_ISSUER,
      "sub": EXTERNAL_USER_ID,
      "azp": azp,
      "aud": tenant_client_id,
      "exp": future_exp,
      "scope": format!("openid {}", scope),
    });
    let token = self.seed_token_with_claims(
      bearer_claims,
      role,
      azp,
      grants,
      Some(inserted.id.clone()),
      tenant_id,
    )?;
    Ok((token, inserted.id))
  }

  fn seed_token(
    &self,
    role: Option<&str>,
    azp: &str,
    grants: Option<services::ApprovedResources>,
    access_request_id: Option<String>,
  ) -> anyhow::Result<String> {
    let future_exp = (Utc::now() + Duration::hours(1)).timestamp() as u64;
    let bearer_claims = serde_json::json!({
      "jti": Uuid::new_v4().to_string(),
      "sub": EXTERNAL_USER_ID,
      "exp": future_exp,
      "scope": "openid",
    });
    self.seed_token_with_claims(
      bearer_claims,
      role,
      azp,
      grants,
      access_request_id,
      TEST_TENANT_ID,
    )
  }

  fn seed_token_with_claims(
    &self,
    bearer_claims: serde_json::Value,
    role: Option<&str>,
    azp: &str,
    grants: Option<services::ApprovedResources>,
    access_request_id: Option<String>,
    tenant_id: &str,
  ) -> anyhow::Result<String> {
    let future_exp = (Utc::now() + Duration::hours(1)).timestamp() as u64;
    let (bearer_jwt, _) = build_token(bearer_claims)?;

    let mut hasher = Sha256::new();
    hasher.update(bearer_jwt.as_bytes());
    let token_digest = format!("{:x}", hasher.finalize())[0..32].to_string();

    let exchange_claims = serde_json::json!({
      "iss": TEST_AUTH_ISSUER,
      "sub": EXTERNAL_USER_ID,
      "azp": azp,
      "exp": future_exp,
      "scope": "openid",
    });
    let (exchange_jwt, _) = build_token(exchange_claims)?;

    let cached = CachedExchangeResult {
      token: exchange_jwt,
      client_id: self.client_id.clone(),
      tenant_id: tenant_id.to_string(),
      app_client_id: azp.to_string(),
      role: role.map(|r| r.to_string()),
      access_request_id,
      grants,
      cached_at: Utc::now().timestamp(),
    };
    let cached_json = serde_json::to_string(&cached)?;
    self
      .cache_service
      .set(&format!("exchanged_token:{}", token_digest), &cached_json);

    Ok(bearer_jwt)
  }
}
