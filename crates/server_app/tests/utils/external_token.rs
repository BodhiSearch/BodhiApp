#![allow(unused)]
use auth_middleware::CachedExchangeResult;
use chrono::{Duration, Utc};
use services::{test_utils::build_token, AppService, CacheService};
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
/// 4. The cache key is just SHA-256(bearer_token)[0..12]
pub struct ExternalTokenSimulator {
  cache_service: Arc<dyn CacheService>,
}

impl ExternalTokenSimulator {
  pub fn new(app_service: &Arc<dyn AppService>) -> Self {
    Self {
      cache_service: app_service.cache_service(),
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
    let token_digest = format!("{:x}", hasher.finalize())[0..12].to_string();

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
      app_client_id: azp.to_string(),
      role: role.map(|r| r.to_string()),
      access_request_id,
    };
    let cached_json = serde_json::to_string(&cached)?;
    self
      .cache_service
      .set(&format!("exchanged_token:{}", token_digest), &cached_json);

    Ok(bearer_jwt)
  }
}
