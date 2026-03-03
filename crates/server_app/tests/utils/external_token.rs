#![allow(unused)]
use chrono::{Duration, Utc};
use routes_app::middleware::CachedExchangeResult;
use services::{
  test_utils::{build_token, TEST_TENANT_ID},
  AppService, CacheService,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

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
  client_id: String,
}

impl ExternalTokenSimulator {
  pub fn new(app_service: &Arc<dyn AppService>) -> Self {
    Self {
      cache_service: app_service.cache_service(),
      client_id: "test-client-id".to_string(),
    }
  }

  pub fn new_with_client_id(app_service: &Arc<dyn AppService>, client_id: String) -> Self {
    Self {
      cache_service: app_service.cache_service(),
      client_id,
    }
  }

  /// Creates a fake external bearer token and seeds the cache so requests
  /// with this token bypass Keycloak and resolve to the given role.
  ///
  /// # Arguments
  /// * `role` - The approved role (e.g., Some("scope_user_user")) from the access request
  /// * `azp` - The authorized party / client ID (e.g., "test-external-app")
  ///
  /// # Returns
  /// The bearer token string to use in `Authorization: Bearer {token}` headers
  pub fn create_token_with_role(&self, role: Option<&str>, azp: &str) -> anyhow::Result<String> {
    let future_exp = (Utc::now() + Duration::hours(1)).timestamp() as u64;
    let access_request_id = role.map(|_| Uuid::new_v4().to_string());

    let bearer_claims = serde_json::json!({
      "jti": Uuid::new_v4().to_string(),
      "sub": "test-external-user",
      "exp": future_exp,
      "scope": "openid",
    });
    let (bearer_jwt, _) = build_token(bearer_claims)?;

    let mut hasher = Sha256::new();
    hasher.update(bearer_jwt.as_bytes());
    let token_digest = format!("{:x}", hasher.finalize())[0..32].to_string();

    let exchange_claims = serde_json::json!({
      "iss": "https://test-id.getbodhi.app/realms/bodhi",
      "sub": "test-external-user",
      "azp": azp,
      "exp": future_exp,
      "scope": "openid",
    });
    let (exchange_jwt, _) = build_token(exchange_claims)?;

    let cached = CachedExchangeResult {
      token: exchange_jwt,
      client_id: self.client_id.clone(),
      tenant_id: TEST_TENANT_ID.to_string(),
      app_client_id: azp.to_string(),
      role: role.map(|r| r.to_string()),
      access_request_id,
      cached_at: Utc::now().timestamp(),
    };
    let cached_json = serde_json::to_string(&cached)?;
    self
      .cache_service
      .set(&format!("exchanged_token:{}", token_digest), &cached_json);

    Ok(bearer_jwt)
  }
}
