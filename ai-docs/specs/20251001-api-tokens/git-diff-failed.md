diff --git a/Cargo.lock b/Cargo.lock
index ce3ad23b..5c79d328 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -326,0 +327 @@ dependencies = [
+ "constant_time_eq",
@@ -1120,0 +1122,6 @@ checksum = "c2459377285ad874054d797f3ccebf984978aa39129f6eafde5cdc8315b612f8"
+[[package]]
+name = "constant_time_eq"
+version = "0.3.1"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "7c74b8349d32d297c9134b8c88677813a227df8f779daa29bfc29c183fe3dca6"
+
@@ -1656 +1663 @@ dependencies = [
- "windows-sys 0.59.0",
+ "windows-sys 0.60.2",
@@ -3447 +3454 @@ dependencies = [
- "windows-targets 0.52.6",
+ "windows-targets 0.53.3",
@@ -5472,0 +5480 @@ dependencies = [
+ "rand 0.9.1",
diff --git a/Cargo.toml b/Cargo.toml
index 8366f3b6..8e6d7c30 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -52,0 +53 @@ chrono = "0.4.41"
+constant_time_eq = "0.3.0"
@@ -106 +107 @@ quote = "1.0.40"
-rand = "0.9.1"
+rand = { version = "0.9.1", features = ["std_rng"] }
diff --git a/crates/auth_middleware/Cargo.toml b/crates/auth_middleware/Cargo.toml
index b4174d18..f9707c3d 100644
--- a/crates/auth_middleware/Cargo.toml
+++ b/crates/auth_middleware/Cargo.toml
@@ -17,0 +18 @@ chrono = { workspace = true }
+constant_time_eq = { workspace = true }
diff --git a/crates/auth_middleware/src/auth_middleware.rs b/crates/auth_middleware/src/auth_middleware.rs
index aaef2b5d..eae93c15 100644
--- a/crates/auth_middleware/src/auth_middleware.rs
+++ b/crates/auth_middleware/src/auth_middleware.rs
@@ -8 +7,0 @@ use axum::{
-
@@ -13 +12 @@ use server_core::RouterState;
-use services::{AppStatus, AuthServiceError, SecretServiceError, TokenError};
+use services::{db::DbError, AppStatus, AuthServiceError, SecretServiceError, TokenError};
@@ -99,0 +99,2 @@ pub enum AuthError {
+  #[error(transparent)]
+  DbError(#[from] DbError),
@@ -241 +242 @@ mod tests {
-  use anyhow_trace::anyhow_trace;
+
@@ -249,0 +251,2 @@ mod tests {
+  use base64::{engine::general_purpose, Engine};
+  use chrono::Utc;
@@ -255,0 +259 @@ mod tests {
+  use rand::RngCore;
@@ -262,0 +267 @@ mod tests {
+  use services::db::{ApiToken, TokenStatus};
@@ -265,3 +270,2 @@ mod tests {
-      access_token_claims, build_token, expired_token, offline_access_token_claims,
-      offline_token_claims, token, AppServiceStubBuilder, SecretServiceStub, TEST_CLIENT_ID,
-      TEST_CLIENT_SECRET,
+      access_token_claims, build_token, expired_token, AppServiceStubBuilder, SecretServiceStub,
+      TEST_CLIENT_ID, TEST_CLIENT_SECRET,
@@ -269,2 +273,2 @@ mod tests {
-    AppRegInfoBuilder, AuthServiceError, MockAuthService, SqliteSessionService, BODHI_HOST,
-    BODHI_PORT, BODHI_SCHEME,
+    AppRegInfoBuilder, AuthServiceError, MockAuthService, SessionService, SqliteSessionService,
+    BODHI_HOST, BODHI_PORT, BODHI_SCHEME,
@@ -271,0 +276 @@ mod tests {
+  use sha2::{Digest, Sha256};
@@ -279,0 +285,68 @@ mod tests {
+  use uuid::Uuid;
+
+  #[rstest]
+  #[tokio::test]
+  async fn test_auth_middleware_bodhiapp_token_success(
+    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
+    _temp_bodhi_home: TempDir,
+  ) -> anyhow::Result<()> {
+    let app_service = AppServiceStubBuilder::default()
+      .with_secret_service()
+      .with_session_service()
+      .await
+      .with_db_service()
+      .await
+      .build()?;
+
+    let mut random_bytes = [0u8; 32];
+    rand::rng().fill_bytes(&mut random_bytes);
+    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
+    let token_str = format!("bodhiapp_{}", random_string);
+    let token_prefix = &token_str[.."bodhiapp_".len() + 8];
+
+    let mut hasher = Sha256::new();
+    hasher.update(token_str.as_bytes());
+    let token_hash = format!("{:x}", hasher.finalize());
+
+    let mut api_token = ApiToken {
+      id: Uuid::new_v4().to_string(),
+      user_id: "test-user".to_string(),
+      name: "test-token".to_string(),
+      token_prefix: token_prefix.to_string(),
+      token_hash,
+      scopes: "scope_token_user".to_string(),
+      status: TokenStatus::Active,
+      created_at: Utc::now(),
+      updated_at: Utc::now(),
+    };
+    app_service
+      .db_service
+      .as_ref()
+      .unwrap()
+      .create_api_token(&mut api_token)
+      .await?;
+
+    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
+      Arc::new(MockSharedContext::new()),
+      Arc::new(app_service),
+    ));
+    let router = test_router(state);
+    let req = Request::get("/with_auth")
+      .header("Authorization", format!("Bearer {}", token_str))
+      .body(Body::empty())?;
+
+    let response = router.oneshot(req).await?;
+    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
+    let actual: TestResponse = response.json().await?;
+    assert_eq!(
+      TestResponse {
+        path: "/with_auth".to_string(),
+        x_resource_token: Some(token_str.clone()),
+        x_resource_role: None,
+        x_resource_scope: Some("scope_token_user".to_string()),
+        authorization_header: Some(format!("Bearer {}", token_str)),
+      },
+      actual
+    );
+    Ok(())
+  }
@@ -374 +446,0 @@ mod tests {
-  #[anyhow_trace]
@@ -441 +513 @@ mod tests {
-    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
+    let session_service_impl = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
@@ -450 +522,5 @@ mod tests {
-    session_service.session_store.create(&mut record).await?;
+    session_service_impl
+      .session_store
+      .create(&mut record)
+      .await?;
+    let session_service: Arc<dyn SessionService> = session_service_impl.clone();
@@ -453 +529 @@ mod tests {
-      .session_service(session_service.clone())
+      .session_service(session_service)
@@ -515 +591 @@ mod tests {
-    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
+    let session_service_impl = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
@@ -525 +601,5 @@ mod tests {
-    session_service.session_store.create(&mut record).await?;
+    session_service_impl
+      .session_store
+      .create(&mut record)
+      .await?;
+    let session_service: Arc<dyn SessionService> = session_service_impl.clone();
@@ -573 +653 @@ mod tests {
-    let updated_record = session_service.session_store.load(&id).await?.unwrap();
+    let updated_record = session_service_impl.session_store.load(&id).await?.unwrap();
@@ -605 +685 @@ mod tests {
-    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
+    let session_service_impl = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
@@ -630,0 +711 @@ mod tests {
+    let session_service: Arc<dyn SessionService> = session_service_impl.clone();
@@ -634 +715 @@ mod tests {
-      .session_service(session_service.clone())
+      .session_service(session_service)
@@ -653 +734,4 @@ mod tests {
-    session_service.session_store.create(&mut record).await?;
+    session_service_impl
+      .session_store
+      .create(&mut record)
+      .await?;
@@ -675 +759 @@ mod tests {
-    let updated_record = session_service.session_store.load(&id).await?.unwrap();
+    let updated_record = session_service_impl.session_store.load(&id).await?.unwrap();
@@ -774,134 +857,0 @@ mod tests {
-  #[rstest]
-  #[case::scope_token_user("offline_access scope_token_user", "scope_token_user")]
-  #[case::scope_token_user("offline_access scope_token_power_user", "scope_token_power_user")]
-  #[case::scope_token_user("offline_access scope_token_manager", "scope_token_manager")]
-  #[case::scope_token_user("offline_access scope_token_admin", "scope_token_admin")]
-  #[case::scope_token_user(
-    "offline_access scope_token_user scope_token_manager",
-    "scope_token_manager"
-  )]
-  #[case::scope_token_user(
-    "offline_access scope_token_user scope_token_power_user",
-    "scope_token_power_user"
-  )]
-  #[tokio::test]
-  async fn test_auth_middleware_bearer_token_success(
-    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
-    #[case] scope: &str,
-    #[case] expected_header: &str,
-  ) -> anyhow::Result<()> {
-    let (bearer_token, _) = build_token(offline_token_claims())?;
-    let mut access_token_claims = offline_access_token_claims();
-    access_token_claims["scope"] = Value::String(scope.to_string());
-    let (access_token, _) = build_token(access_token_claims)?;
-    let access_token_cl = access_token.clone();
-    let mut auth_service = MockAuthService::default();
-    auth_service
-      .expect_refresh_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(bearer_token.clone()),
-      )
-      .times(1)
-      .return_once(|_, _, _| Ok((access_token_cl, Some("refresh_token".to_string()))));
-    let app_service = AppServiceStubBuilder::default()
-      .with_secret_service()
-      .auth_service(Arc::new(auth_service))
-      .with_session_service()
-      .await
-      .with_db_service()
-      .await
-      .build()?;
-    let db = app_service.db_service.as_ref().unwrap();
-    let _ = db
-      .create_api_token_from("test-token-id", &bearer_token)
-      .await?;
-    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
-      Arc::new(MockSharedContext::new()),
-      Arc::new(app_service),
-    ));
-    let router = test_router(state);
-    let req = Request::get("/with_auth")
-      .header("Authorization", format!("Bearer {}", bearer_token))
-      .json(json! {{}})?;
-    let response = router.clone().oneshot(req).await?;
-    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
-    let actual: TestResponse = response.json().await?;
-    assert_eq!(
-      TestResponse {
-        path: "/with_auth".to_string(),
-        x_resource_token: Some(access_token),
-        x_resource_role: None,
-        x_resource_scope: Some(expected_header.to_string()),
-        authorization_header: Some(format!("Bearer {}", bearer_token)),
-      },
-      actual
-    );
-    Ok(())
-  }
-
-  #[anyhow_trace]
-  #[rstest]
-  #[awt]
-  #[tokio::test]
-  async fn test_auth_middleware_gives_precedence_to_token_over_session(
-    temp_bodhi_home: TempDir,
-  ) -> anyhow::Result<()> {
-    let (token, _) = token();
-    let dbfile = temp_bodhi_home.path().join("test.db");
-    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
-    let id = Id::default();
-    let mut record = Record {
-      id,
-      data: maplit::hashmap! {
-        "access_token".to_string() => Value::String(token.clone()),
-      },
-      expiry_date: OffsetDateTime::now_utc() + Duration::days(1),
-    };
-    session_service.session_store.create(&mut record).await?;
-    let offline_token_claims = offline_token_claims();
-    let (offline_token, _) = build_token(offline_token_claims)?;
-    let (offline_access_token, _) = build_token(offline_access_token_claims())?;
-    let app_service = AppServiceStubBuilder::default()
-      .with_secret_service()
-      .session_service(session_service.clone())
-      .with_db_service()
-      .await
-      .build()?;
-    let api_token = app_service
-      .db_service
-      .as_ref()
-      .unwrap()
-      .create_api_token_from("test-token", &offline_token)
-      .await?;
-    app_service.cache_service.as_ref().unwrap().set(
-      &format!("token:{}", api_token.token_id),
-      &offline_access_token,
-    );
-    let state: Arc<dyn RouterState> = Arc::new(DefaultRouterState::new(
-      Arc::new(MockSharedContext::new()),
-      Arc::new(app_service),
-    ));
-    let router = test_router(state);
-    let req = Request::get("/with_auth")
-      .header("Cookie", format!("bodhiapp_session_id={}", id))
-      .header("Sec-Fetch-Site", "same-origin")
-      .header("Authorization", format!("Bearer {}", offline_token))
-      .body(Body::empty())?;
-    let response = router.oneshot(req).await?;
-    assert_eq!(StatusCode::IM_A_TEAPOT, response.status());
-    let actual: TestResponse = response.json().await?;
-    assert_eq!(
-      TestResponse {
-        path: "/with_auth".to_string(),
-        x_resource_token: Some(offline_access_token),
-        x_resource_role: None,
-        x_resource_scope: Some("scope_token_user".to_string()),
-        authorization_header: Some(format!("Bearer {}", offline_token)),
-      },
-      actual
-    );
-    Ok(())
-  }
-
@@ -915 +865 @@ mod tests {
-    let session_service = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
+    let session_service_impl = Arc::new(SqliteSessionService::build_session_service(dbfile).await);
@@ -925 +875,4 @@ mod tests {
-    session_service.session_store.create(&mut record).await?;
+    session_service_impl
+      .session_store
+      .create(&mut record)
+      .await?;
@@ -926,0 +880 @@ mod tests {
+    let session_service: Arc<dyn SessionService> = session_service_impl.clone();
@@ -929 +883 @@ mod tests {
-      .session_service(session_service.clone())
+      .session_service(session_service)
diff --git a/crates/auth_middleware/src/token_service.rs b/crates/auth_middleware/src/token_service.rs
index 0d97597b..dffc941a 100644
--- a/crates/auth_middleware/src/token_service.rs
+++ b/crates/auth_middleware/src/token_service.rs
@@ -2,2 +2 @@ use crate::AuthError;
-use chrono::{Duration, Utc};
-use jsonwebtoken::{DecodingKey, Validation};
+use chrono::Utc;
@@ -7,2 +6,2 @@ use services::{
-  extract_claims, AppRegInfo, AuthService, CacheService, Claims, ExpClaims, OfflineClaims,
-  ScopeClaims, SecretService, SecretServiceExt, SettingService, TokenError, TOKEN_TYPE_OFFLINE,
+  extract_claims, AppRegInfo, AuthService, CacheService, Claims, ScopeClaims, SecretService,
+  SecretServiceExt, SettingService, TokenError,
@@ -10,0 +10 @@ use sha2::{Digest, Sha256};
+use std::str::FromStr;
@@ -15,8 +15 @@ const BEARER_PREFIX: &str = "Bearer ";
-const SCOPE_OFFLINE_ACCESS: &str = "offline_access";
-const LEEWAY_SECONDS: i64 = 60; // 1 minute leeway for clock skew
-
-pub fn create_token_digest(bearer_token: &str) -> String {
-  let mut hasher = Sha256::new();
-  hasher.update(bearer_token.as_bytes());
-  format!("{:x}", hasher.finalize())[0..12].to_string()
-}
+const BODHIAPP_TOKEN_PREFIX: &str = "bodhiapp_";
@@ -27 +20 @@ pub struct DefaultTokenService {
-  cache_service: Arc<dyn CacheService>,
+  _cache_service: Arc<dyn CacheService>,
@@ -36 +29 @@ impl DefaultTokenService {
-    cache_service: Arc<dyn CacheService>,
+    _cache_service: Arc<dyn CacheService>,
@@ -43 +36 @@ impl DefaultTokenService {
-      cache_service,
+      _cache_service,
@@ -53 +45,0 @@ impl DefaultTokenService {
-    // Extract token from header
@@ -57,0 +50 @@ impl DefaultTokenService {
+
@@ -64,20 +57,7 @@ impl DefaultTokenService {
-    // Check token is found and active
-    let api_token = if let Ok(Some(api_token)) = self
-      .db_service
-      .get_api_token_by_token_id(bearer_token)
-      .await
-    {
-      if api_token.status == TokenStatus::Inactive {
-        return Err(AuthError::TokenInactive);
-      } else {
-        api_token
-      }
-    } else {
-      let bearer_claims = extract_claims::<ExpClaims>(bearer_token)?;
-      if bearer_claims.exp < Utc::now().timestamp() as u64 {
-        return Err(TokenError::Expired)?;
-      }
-      let token_digest = create_token_digest(bearer_token);
-      let cached_token = if let Some(access_token) = self
-        .cache_service
-        .get(&format!("exchanged_token:{}", &token_digest))
+    if bearer_token.starts_with(BODHIAPP_TOKEN_PREFIX) {
+      // Handle new database-backed tokens
+      let token_prefix = &bearer_token[..BODHIAPP_TOKEN_PREFIX.len() + 8];
+      if let Some(api_token) = self
+        .db_service
+        .get_api_token_by_prefix(token_prefix)
+        .await?
@@ -85,6 +65,2 @@ impl DefaultTokenService {
-        let scope_claims = extract_claims::<ScopeClaims>(&access_token)?;
-        if scope_claims.exp < Utc::now().timestamp() as u64 {
-          None
-        } else {
-          let user_scope = UserScope::from_scope(&scope_claims.scope)?;
-          Some((access_token, ResourceScope::User(user_scope)))
+        if api_token.status == TokenStatus::Inactive {
+          return Err(AuthError::TokenInactive);
@@ -92,12 +67,0 @@ impl DefaultTokenService {
-      } else {
-        None
-      };
-      if let Some((access_token, resource_scope)) = cached_token {
-        return Ok((access_token, resource_scope));
-      }
-      let (access_token, resource_scope) = self.handle_external_client_token(bearer_token).await?;
-      self
-        .cache_service
-        .set(&format!("exchanged_token:{}", &token_digest), &access_token);
-      return Ok((access_token, resource_scope));
-    };
@@ -105,18 +69,12 @@ impl DefaultTokenService {
-    // Check if token is in cache and not expired
-    if let Some(access_token) = self
-      .cache_service
-      .get(&format!("token:{}", api_token.token_id))
-    {
-      let mut validation = Validation::default();
-      validation.insecure_disable_signature_validation();
-      validation.validate_exp = true;
-      validation.validate_aud = false;
-      let token_data = jsonwebtoken::decode::<ExpClaims>(
-        &access_token,
-        &DecodingKey::from_secret(&[]), // dummy key for parsing
-        &validation,
-      );
-      if let Ok(token_data) = token_data {
-        let offline_scope = token_data.claims.scope;
-        let scope = TokenScope::from_scope(&offline_scope)?;
-        return Ok((access_token, ResourceScope::Token(scope)));
+        let mut hasher = Sha256::new();
+        hasher.update(bearer_token.as_bytes());
+        let hash = format!("{:x}", hasher.finalize());
+
+        if constant_time_eq::constant_time_eq(hash.as_bytes(), api_token.token_hash.as_bytes()) {
+          let scope = TokenScope::from_str(&api_token.scopes)?;
+          // For database tokens, the "access token" is the token itself.
+          // The resource scope is derived from the database.
+          return Ok((bearer_token.to_string(), ResourceScope::Token(scope)));
+        } else {
+          return Err(AuthError::TokenNotFound);
+        }
@@ -124,3 +82 @@ impl DefaultTokenService {
-        self
-          .cache_service
-          .remove(&format!("token:{}", api_token.token_id));
+        return Err(AuthError::TokenNotFound);
@@ -127,0 +84,3 @@ impl DefaultTokenService {
+    } else {
+      // Handle external client JWTs (legacy or other integrations)
+      self.handle_external_client_token(bearer_token).await
@@ -129,28 +87,0 @@ impl DefaultTokenService {
-
-    // If token is active and not found in cache, proceed with full validation
-    let app_reg_info: AppRegInfo = self
-      .secret_service
-      .app_reg_info()?
-      .ok_or(AppRegInfoMissingError)?;
-
-    // Validate claims - iat, expiry, tpe, azp, scope: offline_access
-    let claims = extract_claims::<OfflineClaims>(bearer_token)?;
-    self.validate_token_claims(&claims, &app_reg_info.client_id)?;
-
-    // Exchange token
-    let (access_token, _) = self
-      .auth_service
-      .refresh_token(
-        &app_reg_info.client_id,
-        &app_reg_info.client_secret,
-        bearer_token,
-      )
-      .await?;
-
-    // store the retrieved access token in cache
-    self
-      .cache_service
-      .set(&format!("token:{}", api_token.token_id), &access_token);
-    let scope = extract_claims::<ScopeClaims>(&access_token)?;
-    let token_scope = TokenScope::from_scope(&scope.scope)?;
-    Ok((access_token, ResourceScope::Token(token_scope)))
@@ -217,47 +147,0 @@ impl DefaultTokenService {
-  fn validate_token_claims(
-    &self,
-    claims: &OfflineClaims,
-    client_id: &str,
-  ) -> Result<(), AuthError> {
-    // Validate token expiration
-    let now = Utc::now().timestamp();
-    let leeway = Duration::seconds(LEEWAY_SECONDS);
-
-    // Check if token is not yet valid (with leeway)
-    if claims.iat > (now + leeway.num_seconds()) as u64 {
-      return Err(AuthError::InvalidToken(format!(
-        "token is not yet valid, issued at {}",
-        claims.iat
-      )));
-    }
-
-    // Check token type
-    if claims.typ != TOKEN_TYPE_OFFLINE {
-      return Err(AuthError::InvalidToken(
-        "token type must be Offline".to_string(),
-      ));
-    }
-
-    // Check authorized party
-    if claims.azp != client_id {
-      return Err(AuthError::InvalidToken(
-        "invalid token authorized party".to_string(),
-      ));
-    }
-
-    // Check scope
-    if !claims
-      .scope
-      .split(' ')
-      .map(|s| s.to_string())
-      .collect::<Vec<_>>()
-      .contains(&SCOPE_OFFLINE_ACCESS.to_string())
-    {
-      return Err(AuthError::InvalidToken(
-        "token missing required scope: offline_access".to_string(),
-      ));
-    }
-
-    Ok(())
-  }
-
@@ -328,9 +212,6 @@ mod tests {
-  use crate::{
-    create_token_digest, token_service::SCOPE_OFFLINE_ACCESS, AuthError, DefaultTokenService,
-  };
-  use anyhow_trace::anyhow_trace;
-  use chrono::{Duration, Utc};
-  use mockall::predicate::*;
-  use objs::{
-    test_utils::setup_l10n, FluentLocalizationService, ResourceScope, TokenScope, UserScope,
-  };
+  use super::BODHIAPP_TOKEN_PREFIX;
+  use crate::{AuthError, DefaultTokenService};
+  use base64::engine::general_purpose;
+  use base64::Engine;
+  use objs::{test_utils::setup_l10n, FluentLocalizationService, ResourceScope, TokenScope};
+  use rand::RngCore;
@@ -338 +219 @@ mod tests {
-  use serde_json::{json, Value};
+  use services::db::TokenStatus;
@@ -340,7 +221,3 @@ mod tests {
-    db::DbService,
-    test_utils::{
-      build_token, offline_token_claims, test_db_service, SecretServiceStub, SettingServiceStub,
-      TestDbService, ISSUER, TEST_CLIENT_ID, TEST_CLIENT_SECRET,
-    },
-    AppRegInfoBuilder, AuthServiceError, CacheService, MockAuthService, MockSecretService,
-    MockSettingService, MokaCacheService, TOKEN_TYPE_OFFLINE,
+    db::{ApiToken, DbService},
+    test_utils::{test_db_service, TestDbService},
+    MockAuthService, MockSecretService, MockSettingService, MokaCacheService,
@@ -348 +225,2 @@ mod tests {
-  use std::{collections::HashMap, sync::Arc};
+  use sha2::{Digest, Sha256};
+  use std::sync::Arc;
@@ -351,23 +229,12 @@ mod tests {
-  #[rstest]
-  #[case::empty("")]
-  #[case::malformed("bearer foobar")]
-  #[case::empty_bearer("Bearer ")]
-  #[case::empty_bearer_2("Bearer  ")]
-  #[awt]
-  #[tokio::test]
-  async fn test_validate_bearer_token_header_errors(
-    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
-    #[case] header: &str,
-    #[future] test_db_service: TestDbService,
-  ) -> anyhow::Result<()> {
-    let token_service = Arc::new(DefaultTokenService::new(
-      Arc::new(MockAuthService::default()),
-      Arc::new(MockSecretService::default()),
-      Arc::new(MokaCacheService::default()),
-      Arc::new(test_db_service),
-      Arc::new(MockSettingService::default()),
-    ));
-    let result = token_service.validate_bearer_token(header).await;
-    assert!(result.is_err());
-    assert!(matches!(result, Err(AuthError::Token(_))));
-    Ok(())
+  fn generate_test_token() -> (String, String, String) {
+    let mut random_bytes = [0u8; 32];
+    rand::rng().fill_bytes(&mut random_bytes);
+    let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
+    let token_str = format!("{}{}", BODHIAPP_TOKEN_PREFIX, random_string);
+    let token_prefix = token_str[..BODHIAPP_TOKEN_PREFIX.len() + 8].to_string();
+
+    let mut hasher = Sha256::new();
+    hasher.update(token_str.as_bytes());
+    let token_hash = format!("{:x}", hasher.finalize());
+
+    (token_str, token_prefix, token_hash)
@@ -376 +242,0 @@ mod tests {
-  #[anyhow_trace]
@@ -378,7 +243,0 @@ mod tests {
-  #[case::scope_token_user("offline_access scope_token_user", TokenScope::User)]
-  #[case::scope_token_user_power_user(
-    "offline_access scope_token_power_user",
-    TokenScope::PowerUser
-  )]
-  #[case::scope_token_user_manager("offline_access scope_token_manager", TokenScope::Manager)]
-  #[case::scope_token_user_admin("offline_access scope_token_admin", TokenScope::Admin)]
@@ -389,3 +248,3 @@ mod tests {
-    #[future] test_db_service: TestDbService,
-    #[case] scope: &str,
-    #[case] expected_role: TokenScope,
+    #[future]
+    #[from(test_db_service)]
+    db_service: TestDbService,
@@ -394,21 +253,13 @@ mod tests {
-    let claims = offline_token_claims();
-    let (offline_token, _) = build_token(claims)?;
-    test_db_service
-      .create_api_token_from("test_token", &offline_token)
-      .await?;
-    let (refreshed_token, _) = build_token(
-      json! {{"iss": ISSUER, "azp": TEST_CLIENT_ID, "exp": Utc::now().timestamp() + 3600, "scope": scope}},
-    )?;
-    let refreshed_token_cl = refreshed_token.clone();
-    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
-    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
-    let mut mock_auth = MockAuthService::new();
-    mock_auth
-      .expect_refresh_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(offline_token.clone()),
-      )
-      .times(1)
-      .return_once(|_, _, _| Ok((refreshed_token_cl, Some("new_refresh_token".to_string()))));
+    let (token_str, token_prefix, token_hash) = generate_test_token();
+    let mut api_token = ApiToken {
+      id: Uuid::new_v4().to_string(),
+      user_id: "test-user".to_string(),
+      name: "test-token".to_string(),
+      token_prefix,
+      token_hash,
+      scopes: "scope_token_admin".to_string(),
+      status: TokenStatus::Active,
+      created_at: db_service.now(),
+      updated_at: db_service.now(),
+    };
+    db_service.create_api_token(&mut api_token).await?;
@@ -417,2 +268,2 @@ mod tests {
-      Arc::new(mock_auth),
-      Arc::new(secret_service),
+      Arc::new(MockAuthService::default()),
+      Arc::new(MockSecretService::default()),
@@ -420 +271 @@ mod tests {
-      Arc::new(test_db_service),
+      Arc::new(db_service),
@@ -425,2 +276,2 @@ mod tests {
-    let (result, role) = token_service
-      .validate_bearer_token(&format!("Bearer {}", offline_token))
+    let (validated_token, resource_scope) = token_service
+      .validate_bearer_token(&format!("Bearer {}", token_str))
@@ -430,2 +281,2 @@ mod tests {
-    assert_eq!(refreshed_token, result);
-    assert_eq!(ResourceScope::Token(expected_role), role);
+    assert_eq!(validated_token, token_str);
+    assert_eq!(resource_scope, ResourceScope::Token(TokenScope::Admin));
@@ -435 +285,0 @@ mod tests {
-  #[anyhow_trace]
@@ -437,3 +286,0 @@ mod tests {
-  #[case::scope_token_user("", "missing_offline_access")]
-  #[case::scope_token_user("scope_token_user", "missing_offline_access")]
-  #[case::scope_token_user("offline_access", "missing_token_scope")]
@@ -442 +289 @@ mod tests {
-  async fn test_token_service_bearer_token_exchanged_token_scope_invalid(
+  async fn test_validate_bearer_token_inactive(
@@ -444,3 +291,3 @@ mod tests {
-    #[future] test_db_service: TestDbService,
-    #[case] scope: &str,
-    #[case] err_msg: &str,
+    #[future]
+    #[from(test_db_service)]
+    db_service: TestDbService,
@@ -449,36 +296,13 @@ mod tests {
-    let claims = offline_token_claims();
-    let (offline_token, _) = build_token(claims)?;
-    test_db_service
-      .create_api_token_from("test_token", &offline_token)
-      .await?;
-    let (refreshed_token, _) = build_token(
-      json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "exp": Utc::now().timestamp() + 3600, "scope": scope}},
-    )?;
-    let refreshed_token_cl = refreshed_token.clone();
-    let mut mock_auth = MockAuthService::new();
-    mock_auth
-      .expect_refresh_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(offline_token.clone()),
-      )
-      .times(1)
-      .return_once(|_, _, _| Ok((refreshed_token_cl, Some("new_refresh_token".to_string()))));
-
-    let token_service = Arc::new(DefaultTokenService::new(
-      Arc::new(mock_auth),
-      Arc::new(SecretServiceStub::default().with_app_reg_info_default()),
-      Arc::new(MokaCacheService::default()),
-      Arc::new(test_db_service),
-      Arc::new(MockSettingService::default()),
-    ));
-
-    // When
-    let result = token_service
-      .validate_bearer_token(&format!("Bearer {}", offline_token))
-      .await;
-    assert!(result.is_err());
-    assert_eq!(err_msg, result.unwrap_err().to_string());
-    Ok(())
-  }
+    let (token_str, token_prefix, token_hash) = generate_test_token();
+    let mut api_token = ApiToken {
+      id: Uuid::new_v4().to_string(),
+      user_id: "test-user".to_string(),
+      name: "test-token".to_string(),
+      token_prefix,
+      token_hash,
+      scopes: "scope_token_user".to_string(),
+      status: TokenStatus::Inactive, // Inactive token
+      created_at: db_service.now(),
+      updated_at: db_service.now(),
+    };
+    db_service.create_api_token(&mut api_token).await?;
@@ -486,30 +309,0 @@ mod tests {
-  #[rstest]
-  #[case::invalid_type(
-    json!({"typ": "Invalid"}),"token type must be Offline"
-  )]
-  #[case::wrong_azp(
-    json!({"azp": "wrong-client"}),"invalid token authorized party"
-  )]
-  #[case::no_offline_access_scope(
-    json!({"scope": "openid profile"}),"token missing required scope: offline_access"
-  )]
-  #[awt]
-  #[tokio::test]
-  async fn test_validate_bearer_token_validation_errors(
-    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
-    #[case] claims_override: serde_json::Value,
-    #[case] expected: &str,
-    #[future] test_db_service: TestDbService,
-  ) -> anyhow::Result<()> {
-    // Given
-    let mut claims = offline_token_claims();
-    claims
-      .as_object_mut()
-      .unwrap()
-      .extend(claims_override.as_object().unwrap().clone());
-    let (offline_token, _) = build_token(claims)?;
-    test_db_service
-      .create_api_token_from("test_token", &offline_token)
-      .await?;
-    let secret_service =
-      SecretServiceStub::default().with_app_reg_info(&AppRegInfoBuilder::test_default().build()?);
@@ -518 +312 @@ mod tests {
-      Arc::new(secret_service),
+      Arc::new(MockSecretService::default()),
@@ -520 +314 @@ mod tests {
-      Arc::new(test_db_service),
+      Arc::new(db_service),
@@ -526 +320 @@ mod tests {
-      .validate_bearer_token(&format!("Bearer {}", offline_token))
+      .validate_bearer_token(&format!("Bearer {}", token_str))
@@ -530,4 +324 @@ mod tests {
-    assert!(result.is_err());
-    let api_error = objs::ApiError::from(result.unwrap_err());
-    assert_eq!(expected, api_error.args["var_0"]);
-    assert_eq!("auth_error-invalid_token", api_error.code);
+    assert!(matches!(result, Err(AuthError::TokenInactive)));
@@ -538,9 +328,0 @@ mod tests {
-  #[case( json!({
-    "iat": Utc::now().timestamp() + 3600,  // issued 1 hour in future
-    "jti": "test-jti",
-    "iss": ISSUER,
-    "sub": "test-sub",
-    "typ": TOKEN_TYPE_OFFLINE,
-    "azp": TEST_CLIENT_ID,
-    "scope": SCOPE_OFFLINE_ACCESS
-  }))]
@@ -549,2 +331 @@ mod tests {
-  async fn test_token_time_validation_failures(
-    #[case] claims: Value,
+  async fn test_validate_bearer_token_not_found(
@@ -552 +333,3 @@ mod tests {
-    #[future] test_db_service: TestDbService,
+    #[future]
+    #[from(test_db_service)]
+    db_service: TestDbService,
@@ -555,10 +338,4 @@ mod tests {
-    let (token, _) = build_token(claims)?;
-    test_db_service
-      .create_api_token_from("test_token", &token)
-      .await?;
-    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
-    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
-    let auth_service = MockAuthService::new();
-    let token_service = DefaultTokenService::new(
-      Arc::new(auth_service),
-      Arc::new(secret_service),
+    let (token_str, _, _) = generate_test_token();
+    let token_service = Arc::new(DefaultTokenService::new(
+      Arc::new(MockAuthService::default()),
+      Arc::new(MockSecretService::default()),
@@ -566 +343 @@ mod tests {
-      Arc::new(test_db_service),
+      Arc::new(db_service),
@@ -568 +345 @@ mod tests {
-    );
+    ));
@@ -572 +349 @@ mod tests {
-      .validate_bearer_token(&format!("Bearer {}", token))
+      .validate_bearer_token(&format!("Bearer {}", token_str))
@@ -576,4 +353 @@ mod tests {
-    assert!(result.is_err());
-    assert!(
-      matches!(result, Err(AuthError::InvalidToken(msg)) if msg.starts_with("token is not yet valid"))
-    );
+    assert!(matches!(result, Err(AuthError::TokenNotFound)));
@@ -586 +360 @@ mod tests {
-  async fn test_token_validation_success_with_leeway(
+  async fn test_validate_bearer_token_wrong_hash(
@@ -588 +362,3 @@ mod tests {
-    #[future] test_db_service: TestDbService,
+    #[future]
+    #[from(test_db_service)]
+    db_service: TestDbService,
@@ -591,49 +367,13 @@ mod tests {
-    let now = Utc::now().timestamp();
-    let claims = json!({
-      "exp": now + 30, // expires in 30 seconds
-      "iat": now - 30, // issued 30 seconds ago
-      "jti": "test-jti",
-      "iss": ISSUER,
-      "sub": "test-sub",
-      "typ": TOKEN_TYPE_OFFLINE,
-      "azp": TEST_CLIENT_ID,
-      "scope": SCOPE_OFFLINE_ACCESS
-    });
-    let (offline_token, _) = build_token(claims)?;
-    test_db_service
-      .create_api_token_from("test_token", &offline_token)
-      .await?;
-    let (refreshed_token, _) = build_token(
-      json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "exp": Utc::now().timestamp() + 3600, "scope": "offline_access scope_token_user"}},
-    )?;
-    let refreshed_token_cl = refreshed_token.clone();
-    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
-    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
-    let mut auth_service = MockAuthService::new();
-    auth_service
-      .expect_refresh_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(offline_token.clone()),
-      )
-      .times(1)
-      .return_once(|_, _, _| Ok((refreshed_token_cl, None)));
-    let token_service = DefaultTokenService::new(
-      Arc::new(auth_service),
-      Arc::new(secret_service),
-      Arc::new(MokaCacheService::default()),
-      Arc::new(test_db_service),
-      Arc::new(MockSettingService::default()),
-    );
-
-    // When
-    let (result, token_scope) = token_service
-      .validate_bearer_token(&format!("Bearer {}", offline_token))
-      .await?;
-
-    // Then
-    assert_eq!(refreshed_token, result);
-    assert_eq!(ResourceScope::Token(TokenScope::User), token_scope);
-    Ok(())
-  }
+    let (token_str, token_prefix, _) = generate_test_token();
+    let mut api_token = ApiToken {
+      id: Uuid::new_v4().to_string(),
+      user_id: "test-user".to_string(),
+      name: "test-token".to_string(),
+      token_prefix,
+      token_hash: "wrong_hash".to_string(), // Mismatched hash
+      scopes: "scope_token_user".to_string(),
+      status: TokenStatus::Active,
+      created_at: db_service.now(),
+      updated_at: db_service.now(),
+    };
+    db_service.create_api_token(&mut api_token).await?;
@@ -641,32 +381,3 @@ mod tests {
-  #[rstest]
-  #[awt]
-  #[tokio::test]
-  async fn test_token_validation_auth_service_error(
-    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
-    #[future] test_db_service: TestDbService,
-  ) -> anyhow::Result<()> {
-    // Given
-    let claims = offline_token_claims();
-    let (token, _) = build_token(claims)?;
-    test_db_service
-      .create_api_token_from("test_token", &token)
-      .await?;
-    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
-    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
-    let mut auth_service = MockAuthService::new();
-    auth_service
-      .expect_refresh_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(token.clone()),
-      )
-      .times(1)
-      .return_once(|_, _, _| {
-        Err(AuthServiceError::AuthServiceApiError(
-          "server unreachable".to_string(),
-        ))
-      });
-    let token_service = DefaultTokenService::new(
-      Arc::new(auth_service),
-      Arc::new(secret_service),
+    let token_service = Arc::new(DefaultTokenService::new(
+      Arc::new(MockAuthService::default()),
+      Arc::new(MockSecretService::default()),
@@ -674 +385 @@ mod tests {
-      Arc::new(test_db_service),
+      Arc::new(db_service),
@@ -676 +387 @@ mod tests {
-    );
+    ));
@@ -680 +391 @@ mod tests {
-      .validate_bearer_token(&format!("Bearer {}", token))
+      .validate_bearer_token(&format!("Bearer {}", token_str))
@@ -684,336 +395 @@ mod tests {
-    assert!(result.is_err());
-    assert!(matches!(result, Err(AuthError::AuthService(_))));
-    Ok(())
-  }
-
-  #[rstest]
-  #[awt]
-  #[tokio::test]
-  async fn test_token_validation_with_cache_hit(
-    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
-    #[future] test_db_service: TestDbService,
-  ) -> anyhow::Result<()> {
-    // Given
-    let claims = offline_token_claims();
-    let (offline_token, _) = build_token(claims)?;
-    let (access_token, _) = build_token(
-      json! {{"jti": "test-jti", "sub": "test-sub", "exp": Utc::now().timestamp() + 3600, "scope": "offline_access scope_token_user"}},
-    )?;
-    let api_token = test_db_service
-      .create_api_token_from("test-token", &offline_token)
-      .await?;
-    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
-    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
-    let auth_service = MockAuthService::new(); // Should not be called
-    let cache_service = MokaCacheService::default();
-    let cache_key = format!("token:{}", api_token.token_id);
-    cache_service.set(&cache_key, &access_token);
-    let token_service = DefaultTokenService::new(
-      Arc::new(auth_service),
-      Arc::new(secret_service),
-      Arc::new(cache_service),
-      Arc::new(test_db_service),
-      Arc::new(MockSettingService::default()),
-    );
-
-    // When
-    let (result, scope) = token_service
-      .validate_bearer_token(&format!("Bearer {}", offline_token))
-      .await?;
-
-    // Then
-    assert_eq!(access_token, result);
-    assert_eq!(ResourceScope::Token(TokenScope::User), scope);
-    Ok(())
-  }
-
-  #[rstest]
-  #[awt]
-  #[tokio::test]
-  async fn test_token_validation_with_expired_cache(
-    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
-    #[future] test_db_service: TestDbService,
-  ) -> anyhow::Result<()> {
-    // Given
-    let claims = offline_token_claims();
-    let (offline_token, _) = build_token(claims)?;
-    let api_token = test_db_service
-      .create_api_token_from("test_token", &offline_token)
-      .await?;
-    let (refreshed_token, _) = build_token(
-      json! {{"iss": ISSUER, "azp": TEST_CLIENT_ID, "exp": Utc::now().timestamp() + 3600, "scope": "offline_access scope_token_user"}},
-    )?;
-    let refreshed_token_cl = refreshed_token.clone();
-    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
-    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
-    let mut mock_auth = MockAuthService::new();
-    let cache_service = MokaCacheService::default();
-
-    // Create an expired access token and store it in cache
-    let expired_claims = json!({
-      "exp": Utc::now().timestamp() - 3600, // expired 1 hour ago
-      "iat": Utc::now().timestamp() - 7200,  // issued 2 hours ago
-    });
-    let (expired_access_token, _) = build_token(expired_claims)?;
-
-    // Store expired token in cache
-    cache_service.set(
-      &format!("token:{}", api_token.token_id),
-      &expired_access_token,
-    );
-
-    // Expect token exchange to be called since cached token is expired
-    mock_auth
-      .expect_refresh_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(offline_token.clone()),
-      )
-      .times(1)
-      .return_once(|_, _, _| Ok((refreshed_token_cl, None)));
-
-    let token_service = DefaultTokenService::new(
-      Arc::new(mock_auth),
-      Arc::new(secret_service),
-      Arc::new(cache_service),
-      Arc::new(test_db_service),
-      Arc::new(MockSettingService::default()),
-    );
-
-    // When
-    let (result, token_scope) = token_service
-      .validate_bearer_token(&format!("Bearer {}", offline_token))
-      .await?;
-
-    // Then
-    assert_eq!(refreshed_token, result);
-    assert_eq!(ResourceScope::Token(TokenScope::User), token_scope);
-    Ok(())
-  }
-
-  #[anyhow_trace]
-  #[rstest]
-  #[awt]
-  #[tokio::test]
-  async fn test_validate_external_client_token_success(
-    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
-    #[future] test_db_service: TestDbService,
-  ) -> anyhow::Result<()> {
-    // Given - Create a token from a different client but same issuer
-    let external_client_id = "external-client";
-    let sub = Uuid::new_v4().to_string();
-    let external_token_claims = json!({
-      "exp": (Utc::now() + Duration::hours(1)).timestamp(),
-      "iat": Utc::now().timestamp(),
-      "jti": Uuid::new_v4().to_string(),
-      "iss": ISSUER, // Same issuer as our app
-      "sub": sub,
-      "typ": TOKEN_TYPE_OFFLINE,
-      "azp": external_client_id, // Different client
-      "aud": TEST_CLIENT_ID, // Audience is our client
-      "session_state": Uuid::new_v4().to_string(),
-      "scope": "openid scope_user_user",
-      "sid": Uuid::new_v4().to_string(),
-    });
-    let (external_token, _) = build_token(external_token_claims)?;
-
-    // Setup mock auth service to return exchanged token
-    let (exchanged_token, _) = build_token(
-      json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "jti": "test-jti", "sub": sub, "exp": Utc::now().timestamp() + 3600, "scope": "scope_user_user"}},
-    )?;
-    let exchanged_token_cl = exchanged_token.clone();
-
-    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
-    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
-    let mut mock_auth = MockAuthService::new();
-
-    // Expect token exchange to be called
-    mock_auth
-      .expect_exchange_app_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(external_token.clone()),
-        eq(
-          vec!["scope_user_user", "openid", "email", "profile", "roles"]
-            .iter()
-            .map(|s| s.to_string())
-            .collect::<Vec<String>>(),
-        ),
-      )
-      .times(1)
-      .return_once(|_, _, _, _| Ok((exchanged_token_cl, None)));
-    let mut setting_service = MockSettingService::default();
-    setting_service
-      .expect_auth_issuer()
-      .return_once(|| ISSUER.to_string());
-
-    let token_service = Arc::new(DefaultTokenService::new(
-      Arc::new(mock_auth),
-      Arc::new(secret_service),
-      Arc::new(MokaCacheService::default()),
-      Arc::new(test_db_service),
-      Arc::new(setting_service),
-    ));
-
-    // When - Try to validate the external token
-    let (access_token, scope) = token_service
-      .validate_bearer_token(&format!("Bearer {}", external_token))
-      .await?;
-
-    // Then - Should succeed with exchanged token
-    assert_eq!(exchanged_token, access_token);
-    assert_eq!(ResourceScope::User(UserScope::User), scope);
-    Ok(())
-  }
-
-  #[anyhow_trace]
-  #[rstest]
-  #[awt]
-  #[tokio::test]
-  async fn test_external_client_token_cache_security_prevents_jti_forgery(
-    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
-    #[future] test_db_service: TestDbService,
-  ) -> anyhow::Result<()> {
-    // Given - Create a legitimate external token from a different client
-    let external_client_id = "external-client";
-    let sub = Uuid::new_v4().to_string();
-    let jti = Uuid::new_v4().to_string();
-    let legitimate_token_claims = json!({
-      "exp": (Utc::now() + Duration::hours(1)).timestamp(),
-      "iat": Utc::now().timestamp(),
-      "jti": jti.clone(),
-      "iss": ISSUER,
-      "sub": sub.clone(),
-      "typ": TOKEN_TYPE_OFFLINE,
-      "azp": external_client_id,
-      "aud": TEST_CLIENT_ID,
-      "session_state": Uuid::new_v4().to_string(),
-      "scope": "openid scope_user_user",
-      "sid": Uuid::new_v4().to_string(),
-    });
-    let (legitimate_token, _) = build_token(legitimate_token_claims)?;
-
-    // Create a forged token with the same JTI but different content
-    let forged_token_claims = json!({
-      "exp": (Utc::now() + Duration::hours(1)).timestamp(),
-      "iat": Utc::now().timestamp(),
-      "jti": jti.clone(), // Same JTI as legitimate token
-      "iss": ISSUER,
-      "sub": "malicious-user", // Different subject
-      "typ": TOKEN_TYPE_OFFLINE,
-      "azp": external_client_id,
-      "aud": TEST_CLIENT_ID,
-      "session_state": Uuid::new_v4().to_string(),
-      "scope": "openid scope_user_admin", // Different scope - trying to escalate
-      "sid": Uuid::new_v4().to_string(),
-    });
-    let (forged_token, _) = build_token(forged_token_claims)?;
-
-    // Setup mock auth service - legitimate token succeeds, forged token fails
-    let (legitimate_exchanged_token, _) = build_token(
-      json! {{ "iss": ISSUER, "azp": TEST_CLIENT_ID, "jti": "legitimate-jti", "sub": sub, "exp": Utc::now().timestamp() + 3600, "scope": "scope_user_user"}},
-    )?;
-
-    let app_reg_info = AppRegInfoBuilder::test_default().build()?;
-    let secret_service = SecretServiceStub::default().with_app_reg_info(&app_reg_info);
-    let mut mock_auth = MockAuthService::new();
-    let cache_service = Arc::new(MokaCacheService::default());
-
-    // Expect token exchange for legitimate token to succeed
-    mock_auth
-      .expect_exchange_app_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(legitimate_token.clone()),
-        eq(
-          vec!["scope_user_user", "openid", "email", "profile", "roles"]
-            .iter()
-            .map(|s| s.to_string())
-            .collect::<Vec<String>>(),
-        ),
-      )
-      .times(1)
-      .return_once({
-        let token = legitimate_exchanged_token.clone();
-        move |_, _, _, _| Ok((token, None))
-      });
-
-    // Expect token exchange for forged token to fail with auth service error
-    mock_auth
-      .expect_exchange_app_token()
-      .with(
-        eq(TEST_CLIENT_ID),
-        eq(TEST_CLIENT_SECRET),
-        eq(forged_token.clone()),
-        eq(
-          vec!["scope_user_admin", "openid", "email", "profile", "roles"]
-            .iter()
-            .map(|s| s.to_string())
-            .collect::<Vec<String>>(),
-        ),
-      )
-      .times(1)
-      .return_once(|_, _, _, _| {
-        Err(AuthServiceError::TokenExchangeError(
-          "forged token rejected".to_string(),
-        ))
-      });
-
-    let setting_service = SettingServiceStub::with_settings(HashMap::from([
-      (
-        "BODHI_AUTH_URL".to_string(),
-        "https://id.mydomain.com".to_string(),
-      ),
-      ("BODHI_AUTH_REALM".to_string(), "myapp".to_string()),
-    ]));
-
-    let token_service = Arc::new(DefaultTokenService::new(
-      Arc::new(mock_auth),
-      Arc::new(secret_service),
-      cache_service.clone(),
-      Arc::new(test_db_service),
-      Arc::new(setting_service),
-    ));
-
-    // When - First validate the legitimate token (this will cache it)
-    let (legitimate_access_token, legitimate_scope) = token_service
-      .validate_bearer_token(&format!("Bearer {}", legitimate_token))
-      .await?;
-
-    // Then - Verify legitimate token works as expected
-    assert_eq!(legitimate_exchanged_token, legitimate_access_token);
-    assert_eq!(ResourceScope::User(UserScope::User), legitimate_scope);
-
-    // When - Try to validate the forged token with same JTI
-    let forged_result = token_service
-      .validate_bearer_token(&format!("Bearer {}", forged_token))
-      .await;
-
-    assert!(matches!(
-      forged_result,
-      Err(AuthError::AuthService(
-        AuthServiceError::TokenExchangeError(_)
-      ))
-    ));
-    let legitimate_digest = create_token_digest(&legitimate_token);
-    let forged_digest = create_token_digest(&forged_token);
-    assert_ne!(
-      legitimate_digest, forged_digest,
-      "Token digests should be different even with same JTI"
-    );
-
-    let cached_legitimate = cache_service.get(&format!("exchanged_token:{}", legitimate_digest));
-    let cached_forged = cache_service.get(&format!("exchanged_token:{}", forged_digest));
-
-    assert!(
-      cached_legitimate.is_some(),
-      "Legitimate token should be cached"
-    );
-    assert!(
-      cached_forged.is_none(),
-      "Forged token should not be cached due to validation failure"
-    );
-
+    assert!(matches!(result, Err(AuthError::TokenNotFound)));
diff --git a/crates/bodhi/package-lock.json b/crates/bodhi/package-lock.json
index 8dc81e79..1dff8379 100644
--- a/crates/bodhi/package-lock.json
+++ b/crates/bodhi/package-lock.json
@@ -810,0 +811,36 @@
+    "node_modules/@bundled-es-modules/cookie": {
+      "version": "2.0.1",
+      "resolved": "https://registry.npmjs.org/@bundled-es-modules/cookie/-/cookie-2.0.1.tgz",
+      "integrity": "sha512-8o+5fRPLNbjbdGRRmJj3h6Hh1AQJf2dk3qQ/5ZFb+PXkRNiSoMGGUKlsgLfrxneb72axVJyIYji64E2+nNfYyw==",
+      "dev": true,
+      "license": "ISC",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "cookie": "^0.7.2"
+      }
+    },
+    "node_modules/@bundled-es-modules/cookie/node_modules/cookie": {
+      "version": "0.7.2",
+      "resolved": "https://registry.npmjs.org/cookie/-/cookie-0.7.2.tgz",
+      "integrity": "sha512-yki5XnKuf750l50uGTllt6kKILY4nQ1eNIQatoXEByZ5dWgnKqbnqmTrBE5B4N7lrMJKQ2ytWMiTO2o0v6Ew/w==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": ">= 0.6"
+      }
+    },
+    "node_modules/@bundled-es-modules/statuses": {
+      "version": "1.0.1",
+      "resolved": "https://registry.npmjs.org/@bundled-es-modules/statuses/-/statuses-1.0.1.tgz",
+      "integrity": "sha512-yn7BklA5acgcBr+7w064fGV+SGIFySjCKpqjcWgBAIfrAkY+4GQTJJHQMeT3V/sgz23VTEVV8TtOmkvJAhFVfg==",
+      "dev": true,
+      "license": "ISC",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "statuses": "^2.0.1"
+      }
+    },
@@ -1538,0 +1575,186 @@
+    "node_modules/@inquirer/confirm": {
+      "version": "5.1.16",
+      "resolved": "https://registry.npmjs.org/@inquirer/confirm/-/confirm-5.1.16.tgz",
+      "integrity": "sha512-j1a5VstaK5KQy8Mu8cHmuQvN1Zc62TbLhjJxwHvKPPKEoowSF6h/0UdOpA9DNdWZ+9Inq73+puRq1df6OJ8Sag==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "@inquirer/core": "^10.2.0",
+        "@inquirer/type": "^3.0.8"
+      },
+      "engines": {
+        "node": ">=18"
+      },
+      "peerDependencies": {
+        "@types/node": ">=18"
+      },
+      "peerDependenciesMeta": {
+        "@types/node": {
+          "optional": true
+        }
+      }
+    },
+    "node_modules/@inquirer/core": {
+      "version": "10.2.0",
+      "resolved": "https://registry.npmjs.org/@inquirer/core/-/core-10.2.0.tgz",
+      "integrity": "sha512-NyDSjPqhSvpZEMZrLCYUquWNl+XC/moEcVFqS55IEYIYsY0a1cUCevSqk7ctOlnm/RaSBU5psFryNlxcmGrjaA==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "@inquirer/figures": "^1.0.13",
+        "@inquirer/type": "^3.0.8",
+        "ansi-escapes": "^4.3.2",
+        "cli-width": "^4.1.0",
+        "mute-stream": "^2.0.0",
+        "signal-exit": "^4.1.0",
+        "wrap-ansi": "^6.2.0",
+        "yoctocolors-cjs": "^2.1.2"
+      },
+      "engines": {
+        "node": ">=18"
+      },
+      "peerDependencies": {
+        "@types/node": ">=18"
+      },
+      "peerDependenciesMeta": {
+        "@types/node": {
+          "optional": true
+        }
+      }
+    },
+    "node_modules/@inquirer/core/node_modules/ansi-escapes": {
+      "version": "4.3.2",
+      "resolved": "https://registry.npmjs.org/ansi-escapes/-/ansi-escapes-4.3.2.tgz",
+      "integrity": "sha512-gKXj5ALrKWQLsYG9jlTRmR/xKluxHV+Z9QEwNIgCfM1/uwPMCuzVVnh5mwTd+OuBZcwSIMbqssNWRm1lE51QaQ==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "type-fest": "^0.21.3"
+      },
+      "engines": {
+        "node": ">=8"
+      },
+      "funding": {
+        "url": "https://github.com/sponsors/sindresorhus"
+      }
+    },
+    "node_modules/@inquirer/core/node_modules/cli-width": {
+      "version": "4.1.0",
+      "resolved": "https://registry.npmjs.org/cli-width/-/cli-width-4.1.0.tgz",
+      "integrity": "sha512-ouuZd4/dm2Sw5Gmqy6bGyNNNe1qt9RpmxveLSO7KcgsTnU7RXfsw+/bukWGo1abgBiMAic068rclZsO4IWmmxQ==",
+      "dev": true,
+      "license": "ISC",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": ">= 12"
+      }
+    },
+    "node_modules/@inquirer/core/node_modules/emoji-regex": {
+      "version": "8.0.0",
+      "resolved": "https://registry.npmjs.org/emoji-regex/-/emoji-regex-8.0.0.tgz",
+      "integrity": "sha512-MSjYzcWNOA0ewAHpz0MxpYFvwg6yjy1NG3xteoqz644VCo/RPgnr1/GGt+ic3iJTzQ8Eu3TdM14SawnVUmGE6A==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true
+    },
+    "node_modules/@inquirer/core/node_modules/mute-stream": {
+      "version": "2.0.0",
+      "resolved": "https://registry.npmjs.org/mute-stream/-/mute-stream-2.0.0.tgz",
+      "integrity": "sha512-WWdIxpyjEn+FhQJQQv9aQAYlHoNVdzIzUySNV1gHUPDSdZJ3yZn7pAAbQcV7B56Mvu881q9FZV+0Vx2xC44VWA==",
+      "dev": true,
+      "license": "ISC",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": "^18.17.0 || >=20.5.0"
+      }
+    },
+    "node_modules/@inquirer/core/node_modules/string-width": {
+      "version": "4.2.3",
+      "resolved": "https://registry.npmjs.org/string-width/-/string-width-4.2.3.tgz",
+      "integrity": "sha512-wKyQRQpjJ0sIp62ErSZdGsjMJWsap5oRNihHhu6G7JVO/9jIB6UyevL+tXuOqrng8j/cxKTWyWUwvSTriiZz/g==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "emoji-regex": "^8.0.0",
+        "is-fullwidth-code-point": "^3.0.0",
+        "strip-ansi": "^6.0.1"
+      },
+      "engines": {
+        "node": ">=8"
+      }
+    },
+    "node_modules/@inquirer/core/node_modules/type-fest": {
+      "version": "0.21.3",
+      "resolved": "https://registry.npmjs.org/type-fest/-/type-fest-0.21.3.tgz",
+      "integrity": "sha512-t0rzBq87m3fVcduHDUFhKmyyX+9eo6WQjZvf51Ea/M0Q7+T374Jp1aUiyUl0GKxp8M/OETVHSDvmkyPgvX+X2w==",
+      "dev": true,
+      "license": "(MIT OR CC0-1.0)",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": ">=10"
+      },
+      "funding": {
+        "url": "https://github.com/sponsors/sindresorhus"
+      }
+    },
+    "node_modules/@inquirer/core/node_modules/wrap-ansi": {
+      "version": "6.2.0",
+      "resolved": "https://registry.npmjs.org/wrap-ansi/-/wrap-ansi-6.2.0.tgz",
+      "integrity": "sha512-r6lPcBGxZXlIcymEu7InxDMhdW0KDxpLgoFLcguasxCaJ/SOIZwINatK9KY/tf+ZrlywOKU0UDj3ATXUBfxJXA==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "ansi-styles": "^4.0.0",
+        "string-width": "^4.1.0",
+        "strip-ansi": "^6.0.0"
+      },
+      "engines": {
+        "node": ">=8"
+      }
+    },
+    "node_modules/@inquirer/figures": {
+      "version": "1.0.13",
+      "resolved": "https://registry.npmjs.org/@inquirer/figures/-/figures-1.0.13.tgz",
+      "integrity": "sha512-lGPVU3yO9ZNqA7vTYz26jny41lE7yoQansmqdMLBEfqaGsmdg7V3W9mK9Pvb5IL4EVZ9GnSDGMO/cJXud5dMaw==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": ">=18"
+      }
+    },
+    "node_modules/@inquirer/type": {
+      "version": "3.0.8",
+      "resolved": "https://registry.npmjs.org/@inquirer/type/-/type-3.0.8.tgz",
+      "integrity": "sha512-lg9Whz8onIHRthWaN1Q9EGLa/0LFJjyM8mEUbL1eTi6yMGvBf8gvyDLtxSXztQsxMvhxxNpJYrwa1YHdq+w4Jw==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": ">=18"
+      },
+      "peerDependencies": {
+        "@types/node": ">=18"
+      },
+      "peerDependenciesMeta": {
+        "@types/node": {
+          "optional": true
+        }
+      }
+    },
@@ -2558,0 +2781,22 @@
+    "node_modules/@open-draft/deferred-promise": {
+      "version": "2.2.0",
+      "resolved": "https://registry.npmjs.org/@open-draft/deferred-promise/-/deferred-promise-2.2.0.tgz",
+      "integrity": "sha512-CecwLWx3rhxVQF6V4bAgPS5t+So2sTbPgAzafKkVizyi7tlwpcFpdFqq+wqF2OwNBmqFuu6tOyouTuxgpMfzmA==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true
+    },
+    "node_modules/@open-draft/logger": {
+      "version": "0.3.0",
+      "resolved": "https://registry.npmjs.org/@open-draft/logger/-/logger-0.3.0.tgz",
+      "integrity": "sha512-X2g45fzhxH238HKO4xbSr7+wBS8Fvw6ixhTDuvLd5mqh6bJJCFAPwU9mPDxbcrRtfxv4u5IHCEH77BmxvXmmxQ==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "is-node-process": "^1.2.0",
+        "outvariant": "^1.4.0"
+      }
+    },
@@ -4303,0 +4548,9 @@
+    "node_modules/@types/statuses": {
+      "version": "2.0.6",
+      "resolved": "https://registry.npmjs.org/@types/statuses/-/statuses-2.0.6.tgz",
+      "integrity": "sha512-xMAgYwceFhRA2zY+XbEA7mxYbA093wdiW8Vu6gZPGWy9cmOyU9XesH1tNcEWsKFd5Vzrqx5T3D38PWx1FIIXkA==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true
+    },
@@ -15721,0 +15975,12 @@
+    "node_modules/statuses": {
+      "version": "2.0.2",
+      "resolved": "https://registry.npmjs.org/statuses/-/statuses-2.0.2.tgz",
+      "integrity": "sha512-DvEy55V3DB7uknRo+4iOGT5fP1slR8wQohVdknigZPMpMstaKJQWhwiYBACJE3Ul2pTnATihhBYnRhZQHGBiRw==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": ">= 0.8"
+      }
+    },
@@ -16404,0 +16670,39 @@
+    "node_modules/tough-cookie": {
+      "version": "6.0.0",
+      "resolved": "https://registry.npmjs.org/tough-cookie/-/tough-cookie-6.0.0.tgz",
+      "integrity": "sha512-kXuRi1mtaKMrsLUxz3sQYvVl37B0Ns6MzfrtV5DvJceE9bPyspOqk9xxv7XbZWcfLWbFmm997vl83qUWVJA64w==",
+      "dev": true,
+      "license": "BSD-3-Clause",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "tldts": "^7.0.5"
+      },
+      "engines": {
+        "node": ">=16"
+      }
+    },
+    "node_modules/tough-cookie/node_modules/tldts": {
+      "version": "7.0.12",
+      "resolved": "https://registry.npmjs.org/tldts/-/tldts-7.0.12.tgz",
+      "integrity": "sha512-M9ZQBPp6FyqhMcl233vHYyYRkxXOA1SKGlnq13S0mJdUhRSwr2w6I8rlchPL73wBwRlyIZpFvpu2VcdSMWLYXw==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "tldts-core": "^7.0.12"
+      },
+      "bin": {
+        "tldts": "bin/cli.js"
+      }
+    },
+    "node_modules/tough-cookie/node_modules/tldts-core": {
+      "version": "7.0.12",
+      "resolved": "https://registry.npmjs.org/tldts-core/-/tldts-core-7.0.12.tgz",
+      "integrity": "sha512-3K76aXywJFduGRsOYoY5JzINLs/WMlOkeDwPL+8OCPq2Rh39gkSDtWAxdJQlWjpun/xF/LHf29yqCi6VC/rHDA==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true
+    },
@@ -16532,0 +16837,15 @@
+    "node_modules/type-fest": {
+      "version": "4.41.0",
+      "resolved": "https://registry.npmjs.org/type-fest/-/type-fest-4.41.0.tgz",
+      "integrity": "sha512-TeTSQ6H5YHvpqVwBRcnLDCBnDOHWYu7IvGbHT6N8AOymcr9PJGjc1GTtiWZTYg0NCgYwvnYWEkVChQAr9bjfwA==",
+      "dev": true,
+      "license": "(MIT OR CC0-1.0)",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": ">=16"
+      },
+      "funding": {
+        "url": "https://github.com/sponsors/sindresorhus"
+      }
+    },
@@ -17124,0 +17444,38 @@
+    "node_modules/vitest/node_modules/@mswjs/interceptors": {
+      "version": "0.39.6",
+      "resolved": "https://registry.npmjs.org/@mswjs/interceptors/-/interceptors-0.39.6.tgz",
+      "integrity": "sha512-bndDP83naYYkfayr/qhBHMhk0YGwS1iv6vaEGcr0SQbO0IZtbOPqjKjds/WcG+bJA+1T5vCx6kprKOzn5Bg+Vw==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "@open-draft/deferred-promise": "^2.2.0",
+        "@open-draft/logger": "^0.3.0",
+        "@open-draft/until": "^2.0.0",
+        "is-node-process": "^1.2.0",
+        "outvariant": "^1.4.3",
+        "strict-event-emitter": "^0.5.1"
+      },
+      "engines": {
+        "node": ">=18"
+      }
+    },
+    "node_modules/vitest/node_modules/@open-draft/until": {
+      "version": "2.1.0",
+      "resolved": "https://registry.npmjs.org/@open-draft/until/-/until-2.1.0.tgz",
+      "integrity": "sha512-U69T3ItWHvLwGg5eJ0n3I62nWuE6ilHlmz7zM0npLBRvPRd7e6NYmg54vvRtP5mZG7kZqZCFVdsTWo7BPtBujg==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true
+    },
+    "node_modules/vitest/node_modules/@types/cookie": {
+      "version": "0.6.0",
+      "resolved": "https://registry.npmjs.org/@types/cookie/-/cookie-0.6.0.tgz",
+      "integrity": "sha512-4Kh9a6B2bQciAhf7FSuMRRkUWecJgJu9nPnx3yzpsfXX/c50REIqpHY4C82bXP90qrLtXtkDxTZosYO3UpOwlA==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true
+    },
@@ -17151,0 +17509,65 @@
+    "node_modules/vitest/node_modules/headers-polyfill": {
+      "version": "4.0.3",
+      "resolved": "https://registry.npmjs.org/headers-polyfill/-/headers-polyfill-4.0.3.tgz",
+      "integrity": "sha512-IScLbePpkvO846sIwOtOTDjutRMWdXdJmXdMvk6gCBHxFO8d+QKOQedyZSxFTTFYRSmlgSTDtXqqq4pcenBXLQ==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true
+    },
+    "node_modules/vitest/node_modules/msw": {
+      "version": "2.11.1",
+      "resolved": "https://registry.npmjs.org/msw/-/msw-2.11.1.tgz",
+      "integrity": "sha512-dGSRx0AJmQVQfpGXTsAAq4JFdwdhOBdJ6sJS/jnN0ac3s0NZB6daacHF1z5Pefx+IejmvuiLWw260RlyQOf3sQ==",
+      "dev": true,
+      "hasInstallScript": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "dependencies": {
+        "@bundled-es-modules/cookie": "^2.0.1",
+        "@bundled-es-modules/statuses": "^1.0.1",
+        "@inquirer/confirm": "^5.0.0",
+        "@mswjs/interceptors": "^0.39.1",
+        "@open-draft/deferred-promise": "^2.2.0",
+        "@open-draft/until": "^2.1.0",
+        "@types/cookie": "^0.6.0",
+        "@types/statuses": "^2.0.4",
+        "graphql": "^16.8.1",
+        "headers-polyfill": "^4.0.2",
+        "is-node-process": "^1.2.0",
+        "outvariant": "^1.4.3",
+        "path-to-regexp": "^6.3.0",
+        "picocolors": "^1.1.1",
+        "strict-event-emitter": "^0.5.1",
+        "tough-cookie": "^6.0.0",
+        "type-fest": "^4.26.1",
+        "yargs": "^17.7.2"
+      },
+      "bin": {
+        "msw": "cli/index.js"
+      },
+      "engines": {
+        "node": ">=18"
+      },
+      "funding": {
+        "url": "https://github.com/sponsors/mswjs"
+      },
+      "peerDependencies": {
+        "typescript": ">= 4.8.x"
+      },
+      "peerDependenciesMeta": {
+        "typescript": {
+          "optional": true
+        }
+      }
+    },
+    "node_modules/vitest/node_modules/strict-event-emitter": {
+      "version": "0.5.1",
+      "resolved": "https://registry.npmjs.org/strict-event-emitter/-/strict-event-emitter-0.5.1.tgz",
+      "integrity": "sha512-vMgjE/GGEPEFnhFub6pa4FmJBRBVOLpIII2hvCZ8Kzb7K0hlHo7mQv6xYrBvCL2LtAIBwFUK8wvuJgTVSQ5MFQ==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true
+    },
@@ -17641,0 +18064,15 @@
+    "node_modules/yoctocolors-cjs": {
+      "version": "2.1.3",
+      "resolved": "https://registry.npmjs.org/yoctocolors-cjs/-/yoctocolors-cjs-2.1.3.tgz",
+      "integrity": "sha512-U/PBtDf35ff0D8X8D0jfdzHYEPFxAI7jJlxZXwCSez5M3190m+QobIfh+sWDWSHMCWWJN2AWamkegn6vr6YBTw==",
+      "dev": true,
+      "license": "MIT",
+      "optional": true,
+      "peer": true,
+      "engines": {
+        "node": ">=18"
+      },
+      "funding": {
+        "url": "https://github.com/sponsors/sindresorhus"
+      }
+    },
diff --git a/crates/commands/src/cmd_create.rs b/crates/commands/src/cmd_create.rs
index bc646726..a2c39108 100644
--- a/crates/commands/src/cmd_create.rs
+++ b/crates/commands/src/cmd_create.rs
@@ -2 +2 @@ use objs::{
-  UserAliasBuilder, AliasSource, AppError, BuilderError, OAIRequestParams, ObjValidationError, Repo,
+  AliasSource, AppError, BuilderError, OAIRequestParams, ObjValidationError, Repo, UserAliasBuilder,
@@ -117 +117 @@ mod test {
-  use objs::{UserAlias, UserAliasBuilder, HubFile, OAIRequestParamsBuilder, Repo};
+  use objs::{HubFile, OAIRequestParamsBuilder, Repo, UserAlias, UserAliasBuilder};
diff --git a/crates/commands/src/cmd_pull.rs b/crates/commands/src/cmd_pull.rs
index 86b9ea61..7078bfed 100644
--- a/crates/commands/src/cmd_pull.rs
+++ b/crates/commands/src/cmd_pull.rs
@@ -1 +1 @@
-use objs::{UserAliasBuilder, AliasSource, AppError, BuilderError, ObjValidationError, Repo};
+use objs::{AliasSource, AppError, BuilderError, ObjValidationError, Repo, UserAliasBuilder};
@@ -106 +106 @@ mod test {
-  use objs::{UserAlias, HubFile, RemoteModel, Repo};
+  use objs::{HubFile, RemoteModel, Repo, UserAlias};
diff --git a/crates/commands/src/objs_ext.rs b/crates/commands/src/objs_ext.rs
index c95e8a2c..29744fcd 100644
--- a/crates/commands/src/objs_ext.rs
+++ b/crates/commands/src/objs_ext.rs
@@ -1 +1 @@
-use objs::{UserAlias, HubFile, RemoteModel};
+use objs::{HubFile, RemoteModel, UserAlias};
@@ -55 +55 @@ mod test {
-  use objs::{UserAlias, HubFile, RemoteModel, Repo};
+  use objs::{HubFile, RemoteModel, Repo, UserAlias};
diff --git a/crates/objs/src/alias.rs b/crates/objs/src/alias.rs
index e778706b..9d2f883b 100644
--- a/crates/objs/src/alias.rs
+++ b/crates/objs/src/alias.rs
@@ -1 +1 @@
-use crate::{UserAlias, ApiAlias};
+use crate::{ApiAlias, UserAlias};
@@ -38 +38 @@ mod tests {
-  use crate::{UserAliasBuilder, AliasSource, Repo};
+  use crate::{AliasSource, Repo, UserAliasBuilder};
diff --git a/crates/objs/src/lib.rs b/crates/objs/src/lib.rs
index fa8d262e..5f26a5f8 100644
--- a/crates/objs/src/lib.rs
+++ b/crates/objs/src/lib.rs
@@ -6 +6 @@ pub mod test_utils;
-mod user_alias;
+mod alias;
@@ -12 +12 @@ pub mod gguf;
-mod alias;
+mod user_alias;
@@ -26 +26 @@ mod utils;
-pub use user_alias::*;
+pub use alias::*;
@@ -31 +31 @@ pub use error::*;
-pub use alias::*;
+pub use user_alias::*;
diff --git a/crates/objs/src/test_utils/objs.rs b/crates/objs/src/test_utils/objs.rs
index 58e074bd..c6b3ac81 100644
--- a/crates/objs/src/test_utils/objs.rs
+++ b/crates/objs/src/test_utils/objs.rs
@@ -2,2 +2,2 @@ use crate::{
-  test_utils::SNAPSHOT, UserAlias, AliasSource, HubFile, HubFileBuilder, OAIRequestParams,
-  OAIRequestParamsBuilder, RemoteModel, Repo, TOKENIZER_CONFIG_JSON,
+  test_utils::SNAPSHOT, AliasSource, HubFile, HubFileBuilder, OAIRequestParams,
+  OAIRequestParamsBuilder, RemoteModel, Repo, UserAlias, TOKENIZER_CONFIG_JSON,
diff --git a/crates/objs/src/user_alias.rs b/crates/objs/src/user_alias.rs
index 38758c05..27a63427 100644
--- a/crates/objs/src/user_alias.rs
+++ b/crates/objs/src/user_alias.rs
@@ -59 +59 @@ mod test {
-  use crate::{UserAlias, OAIRequestParamsBuilder, Repo};
+  use crate::{OAIRequestParamsBuilder, Repo, UserAlias};
diff --git a/crates/routes_app/Cargo.toml b/crates/routes_app/Cargo.toml
index 945da499..98807cfc 100644
--- a/crates/routes_app/Cargo.toml
+++ b/crates/routes_app/Cargo.toml
@@ -28,0 +29 @@ sha2 = { workspace = true }
+rand = { workspace = true, features = ["std_rng"] }
diff --git a/crates/routes_app/src/objs.rs b/crates/routes_app/src/objs.rs
index 6297727b..d34d271b 100644
--- a/crates/routes_app/src/objs.rs
+++ b/crates/routes_app/src/objs.rs
@@ -1 +1 @@
-use objs::{UserAlias, HubFile, OAIRequestParams};
+use objs::{HubFile, OAIRequestParams, UserAlias};
diff --git a/crates/routes_app/src/routes_api_models.rs b/crates/routes_app/src/routes_api_models.rs
index 5977572a..5813408a 100644
--- a/crates/routes_app/src/routes_api_models.rs
+++ b/crates/routes_app/src/routes_api_models.rs
@@ -15 +15 @@ use objs::{
-  AliasSource, ApiError, ApiAlias, BadRequestError, ObjValidationError, API_TAG_API_MODELS,
+  AliasSource, ApiAlias, ApiError, BadRequestError, ObjValidationError, API_TAG_API_MODELS,
diff --git a/crates/routes_app/src/routes_api_token.rs b/crates/routes_app/src/routes_api_token.rs
index acb49a74..3fc9989a 100644
--- a/crates/routes_app/src/routes_api_token.rs
+++ b/crates/routes_app/src/routes_api_token.rs
@@ -2 +2,5 @@ use crate::{PaginatedApiTokenResponse, PaginatedResponse, PaginationSortParams,
-use auth_middleware::KEY_RESOURCE_TOKEN;
+use auth_middleware::{KEY_RESOURCE_ROLE, KEY_RESOURCE_TOKEN};
+use base64::engine::general_purpose;
+use base64::Engine;
+use objs::{Role, TokenScope};
+
@@ -9,4 +13,3 @@ use axum_extra::extract::WithRejection;
-use objs::{
-  ApiError, AppError, EntityError, ErrorType, OpenAIApiError, ServiceUnavailableError,
-  API_TAG_API_KEYS,
-};
+use chrono::Utc;
+use objs::{ApiError, AppError, EntityError, ErrorType, OpenAIApiError, API_TAG_API_KEYS};
+use rand::RngCore;
@@ -18,0 +22,2 @@ use services::{
+use sha2::{Digest, Sha256};
+use std::str::FromStr;
@@ -20,0 +26 @@ use utoipa::ToSchema;
+use uuid::Uuid;
@@ -87,3 +93,3 @@ pub async fn create_token_handler(
-  _headers: HeaderMap,
-  State(_state): State<Arc<dyn RouterState>>,
-  WithRejection(Json(_payload), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
+  headers: HeaderMap,
+  State(state): State<Arc<dyn RouterState>>,
+  WithRejection(Json(payload), _): WithRejection<Json<CreateApiTokenRequest>, ApiError>,
@@ -91,3 +97,57 @@ pub async fn create_token_handler(
-  Err(ServiceUnavailableError::new(
-    "api token feature is not available".to_string(),
-  ))?
+  let app_service = state.app_service();
+  let db_service = app_service.db_service();
+
+  let resource_token = headers
+    .get(KEY_RESOURCE_TOKEN)
+    .and_then(|token| token.to_str().ok())
+    .ok_or(ApiTokenError::AccessTokenMissing)?;
+
+  let user_id = extract_claims::<IdClaims>(resource_token)?.sub;
+  let user_role_str = headers
+    .get(KEY_RESOURCE_ROLE)
+    .and_then(|role| role.to_str().ok())
+    .unwrap_or("user");
+
+  let user_role = Role::from_str(user_role_str).unwrap_or(Role::User);
+
+  // For now, we only support creating user-scoped tokens.
+  // In the future, we can allow higher-role users to create tokens with different scopes.
+  let token_scope = match user_role {
+    Role::Admin => TokenScope::Admin,
+    Role::Manager => TokenScope::Manager,
+    Role::PowerUser => TokenScope::PowerUser,
+    Role::User => TokenScope::User,
+  };
+
+  // Generate a new token
+  let mut random_bytes = [0u8; 32];
+  rand::rng().fill_bytes(&mut random_bytes);
+  let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
+  let token_str = format!("bodhiapp_{}", random_string);
+
+  let token_prefix = &token_str[.."bodhiapp_".len() + 8];
+
+  let mut hasher = Sha256::new();
+  hasher.update(token_str.as_bytes());
+  let token_hash = format!("{:x}", hasher.finalize());
+
+  let mut api_token = ApiToken {
+    id: Uuid::new_v4().to_string(),
+    user_id,
+    name: payload.name.unwrap_or_default(),
+    token_prefix: token_prefix.to_string(),
+    token_hash,
+    scopes: token_scope.to_string(),
+    status: TokenStatus::Active,
+    created_at: Utc::now(),
+    updated_at: Utc::now(),
+  };
+
+  db_service.create_api_token(&mut api_token).await?;
+
+  Ok((
+    StatusCode::CREATED,
+    Json(ApiTokenResponse {
+      offline_token: token_str,
+    }),
+  ))
@@ -430 +490 @@ mod tests {
-    assert_eq!("test-jti", created_token.token_id);
+    assert_eq!("bodhiapp_test", created_token.token_prefix);
@@ -589 +649 @@ mod tests {
-        token_id: Uuid::new_v4().to_string(),
+        token_prefix: format!("bodhiapp_test{:02}", i),
@@ -590,0 +651 @@ mod tests {
+        scopes: "scope_token_user".to_string(),
@@ -695 +756 @@ mod tests {
-      token_id: "token123".to_string(),
+      token_prefix: "bodhiapp_test".to_string(),
@@ -696,0 +758 @@ mod tests {
+      scopes: "scope_token_user".to_string(),
diff --git a/crates/routes_oai/src/routes_oai_models.rs b/crates/routes_oai/src/routes_oai_models.rs
index be5d2181..50dc3088 100644
--- a/crates/routes_oai/src/routes_oai_models.rs
+++ b/crates/routes_oai/src/routes_oai_models.rs
@@ -7 +7 @@ use axum::{
-use objs::{UserAlias, ApiError, ApiAlias, OpenAIApiError, API_TAG_OPENAI};
+use objs::{ApiAlias, ApiError, OpenAIApiError, UserAlias, API_TAG_OPENAI};
diff --git a/crates/server_core/src/model_router.rs b/crates/server_core/src/model_router.rs
index ef34af51..8394dbdb 100644
--- a/crates/server_core/src/model_router.rs
+++ b/crates/server_core/src/model_router.rs
@@ -2 +2 @@ use async_trait::async_trait;
-use objs::{UserAlias, AliasSource, ApiAlias, AppError, ErrorType};
+use objs::{AliasSource, ApiAlias, AppError, ErrorType, UserAlias};
diff --git a/crates/server_core/src/shared_rw.rs b/crates/server_core/src/shared_rw.rs
index 524c3906..39f0a1e3 100644
--- a/crates/server_core/src/shared_rw.rs
+++ b/crates/server_core/src/shared_rw.rs
@@ -334 +334 @@ mod test {
-  use objs::{test_utils::temp_hf_home, UserAlias, HubFileBuilder};
+  use objs::{test_utils::temp_hf_home, HubFileBuilder, UserAlias};
diff --git a/crates/services/migrations/0003_create_api_tokens.up.sql b/crates/services/migrations/0003_create_api_tokens.up.sql
index 1fec29d6..1acc36e3 100644
--- a/crates/services/migrations/0003_create_api_tokens.up.sql
+++ b/crates/services/migrations/0003_create_api_tokens.up.sql
@@ -8 +8 @@ CREATE TABLE api_tokens (
-    token_id TEXT NOT NULL UNIQUE,
+    token_prefix TEXT NOT NULL UNIQUE,
@@ -9,0 +10 @@ CREATE TABLE api_tokens (
+    scopes TEXT NOT NULL,
@@ -15,2 +16,2 @@ CREATE TABLE api_tokens (
--- Create index on token_id for faster lookups
-CREATE INDEX idx_api_tokens_token_id ON api_tokens(token_id);
+-- Create index on token_prefix for faster lookups
+CREATE INDEX idx_api_tokens_token_prefix ON api_tokens(token_prefix);
\ No newline at end of file
diff --git a/crates/services/src/data_service.rs b/crates/services/src/data_service.rs
index 1064b2b1..3edd6da5 100644
--- a/crates/services/src/data_service.rs
+++ b/crates/services/src/data_service.rs
@@ -3 +3 @@ use objs::{
-  impl_error_from, UserAlias, AppError, ErrorType, IoDirCreateError, IoError, IoFileDeleteError,
+  impl_error_from, AppError, ErrorType, IoDirCreateError, IoError, IoFileDeleteError,
@@ -4,0 +5 @@ use objs::{
+  UserAlias,
@@ -282 +283 @@ mod test {
-    UserAlias, AppError, FluentLocalizationService, RemoteModel,
+    AppError, FluentLocalizationService, RemoteModel, UserAlias,
diff --git a/crates/services/src/db/objs.rs b/crates/services/src/db/objs.rs
index 42a60c1a..d65f6e0f 100644
--- a/crates/services/src/db/objs.rs
+++ b/crates/services/src/db/objs.rs
@@ -85 +85 @@ pub struct ApiToken {
-  pub token_id: String,
+  pub token_prefix: String,
@@ -86,0 +87 @@ pub struct ApiToken {
+  pub scopes: String,
diff --git a/crates/services/src/db/service.rs b/crates/services/src/db/service.rs
index 537544e1..1e7540a7 100644
--- a/crates/services/src/db/service.rs
+++ b/crates/services/src/db/service.rs
@@ -1,7 +1,4 @@
-use crate::{
-  db::{
-    encryption::{decrypt_api_key, encrypt_api_key},
-    AccessRequest, ApiToken, DownloadRequest, DownloadStatus, RequestStatus, SqlxError,
-    SqlxMigrateError, TokenStatus,
-  },
-  extract_claims,
+use crate::db::{
+  encryption::{decrypt_api_key, encrypt_api_key},
+  AccessRequest, ApiToken, DownloadRequest, DownloadStatus, RequestStatus, SqlxError,
+  SqlxMigrateError, TokenStatus,
@@ -15 +11,0 @@ use std::{fs, path::Path, str::FromStr, sync::Arc, time::UNIX_EPOCH};
-use uuid::Uuid;
@@ -99,2 +94,0 @@ pub trait DbService: std::fmt::Debug + Send + Sync {
-  async fn create_api_token_from(&self, name: &str, token: &str) -> Result<ApiToken, DbError>;
-
@@ -111 +105 @@ pub trait DbService: std::fmt::Debug + Send + Sync {
-  async fn get_api_token_by_token_id(&self, token: &str) -> Result<Option<ApiToken>, DbError>;
+  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError>;
@@ -121,5 +115 @@ pub trait DbService: std::fmt::Debug + Send + Sync {
-  async fn create_api_model_alias(
-    &self,
-    alias: &ApiAlias,
-    api_key: &str,
-  ) -> Result<(), DbError>;
+  async fn create_api_model_alias(&self, alias: &ApiAlias, api_key: &str) -> Result<(), DbError>;
@@ -167,0 +158 @@ impl SqliteDbService {
+        String,
@@ -178 +169,11 @@ impl SqliteDbService {
-      Some((id, user_id, name, token_id, token_hash, status, created_at, updated_at)) => {
+      Some((
+        id,
+        user_id,
+        name,
+        token_prefix,
+        token_hash,
+        scopes,
+        status,
+        created_at,
+        updated_at,
+      )) => {
@@ -188 +189 @@ impl SqliteDbService {
-          token_id,
+          token_prefix,
@@ -189,0 +191 @@ impl SqliteDbService {
+          scopes,
@@ -461,2 +463,2 @@ impl DbService for SqliteDbService {
-      INSERT INTO api_tokens (id, user_id, name, token_id, token_hash, status, created_at, updated_at)
-      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
+      INSERT INTO api_tokens (id, user_id, name, token_prefix, token_hash, scopes, status, created_at, updated_at)
+      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
@@ -468 +470 @@ impl DbService for SqliteDbService {
-    .bind(&token.token_id)
+    .bind(&token.token_prefix)
@@ -469,0 +472 @@ impl DbService for SqliteDbService {
+    .bind(&token.scopes)
@@ -479,44 +481,0 @@ impl DbService for SqliteDbService {
-  async fn create_api_token_from(&self, name: &str, token: &str) -> Result<ApiToken, DbError> {
-    use crate::IdClaims;
-    use sha2::{Digest, Sha256};
-
-    let claims =
-      extract_claims::<IdClaims>(token).map_err(|e| DbError::TokenValidation(e.to_string()))?;
-
-    let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
-    let token_hash = token_hash[..12].to_string();
-
-    let now = self.time_service.utc_now();
-    let id = Uuid::new_v4().to_string();
-
-    let api_token = ApiToken {
-      id,
-      user_id: claims.sub,
-      name: name.to_string(),
-      token_id: claims.jti,
-      token_hash,
-      status: TokenStatus::Active,
-      created_at: now,
-      updated_at: now,
-    };
-
-    sqlx::query(
-      r#"
-      INSERT INTO api_tokens (id, user_id, name, token_id, token_hash, status, created_at, updated_at)
-      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
-      "#,
-    )
-    .bind(&api_token.id)
-    .bind(&api_token.user_id)
-    .bind(&api_token.name)
-    .bind(&api_token.token_id)
-    .bind(&api_token.token_hash)
-    .bind(api_token.status.to_string())
-    .bind(api_token.created_at.timestamp())
-    .bind(api_token.updated_at.timestamp())
-    .execute(&self.pool)
-    .await?;
-
-    Ok(api_token)
-  }
-
@@ -538,0 +498 @@ impl DbService for SqliteDbService {
+        String,
@@ -548 +508 @@ impl DbService for SqliteDbService {
-        token_id,
+        token_prefix,
@@ -549,0 +510 @@ impl DbService for SqliteDbService {
+        scopes,
@@ -568 +529 @@ impl DbService for SqliteDbService {
-        |(id, user_id, name, token_id, token_hash, status, created_at, updated_at)| {
+        |(id, user_id, name, token_prefix, token_hash, scopes, status, created_at, updated_at)| {
@@ -578 +539 @@ impl DbService for SqliteDbService {
-            token_id,
+            token_prefix,
@@ -579,0 +541 @@ impl DbService for SqliteDbService {
+            scopes,
@@ -605 +567 @@ impl DbService for SqliteDbService {
-        token_id,
+        token_prefix,
@@ -606,0 +569 @@ impl DbService for SqliteDbService {
+        scopes,
@@ -616,6 +579,16 @@ impl DbService for SqliteDbService {
-  async fn get_api_token_by_token_id(&self, token: &str) -> Result<Option<ApiToken>, DbError> {
-    use crate::IdClaims;
-    use sha2::{Digest, Sha256};
-    let claims =
-      extract_claims::<IdClaims>(token).map_err(|e| DbError::TokenValidation(e.to_string()))?;
-    let query = r#"
+  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError> {
+    let result = query_as::<
+      _,
+      (
+        String,
+        String,
+        String,
+        String,
+        String,
+        String,
+        String,
+        DateTime<Utc>,
+        DateTime<Utc>,
+      ),
+    >(
+      r#"
@@ -626 +599 @@ impl DbService for SqliteDbService {
-        token_id,
+        token_prefix,
@@ -627,0 +601 @@ impl DbService for SqliteDbService {
+        scopes,
@@ -632,13 +606,36 @@ impl DbService for SqliteDbService {
-      WHERE user_id = ? AND token_id = ?
-      "#;
-    let api_token = self.get_by_col(query, &claims.sub, &claims.jti).await?;
-    match api_token {
-      None => Ok(None),
-      Some(api_token) => {
-        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
-        let token_hash = token_hash[..12].to_string();
-        if api_token.token_hash == token_hash {
-          Ok(Some(api_token))
-        } else {
-          Ok(None)
-        }
+      WHERE token_prefix = ?
+      "#,
+    )
+    .bind(prefix)
+    .fetch_optional(&self.pool)
+    .await?;
+
+    match result {
+      Some((
+        id,
+        user_id,
+        name,
+        token_prefix,
+        token_hash,
+        scopes,
+        status,
+        created_at,
+        updated_at,
+      )) => {
+        let Ok(status) = TokenStatus::from_str(&status) else {
+          tracing::warn!("unknown token status: {status} for id: {id}");
+          return Ok(None);
+        };
+
+        let result = ApiToken {
+          id,
+          user_id,
+          name,
+          token_prefix,
+          token_hash,
+          scopes,
+          status,
+          created_at,
+          updated_at,
+        };
+        Ok(Some(result))
@@ -645,0 +643 @@ impl DbService for SqliteDbService {
+      None => Ok(None),
@@ -727,5 +725 @@ impl DbService for SqliteDbService {
-  async fn create_api_model_alias(
-    &self,
-    alias: &ApiAlias,
-    api_key: &str,
-  ) -> Result<(), DbError> {
+  async fn create_api_model_alias(&self, alias: &ApiAlias, api_key: &str) -> Result<(), DbError> {
@@ -909 +903 @@ mod test {
-    test_utils::{build_token, test_db_service, TestDbService},
+    test_utils::{test_db_service, TestDbService},
@@ -1111 +1105 @@ mod test {
-      token_id: Uuid::new_v4().to_string(),
+      token_prefix: "bodhiapp_test".to_string(),
@@ -1112,0 +1107 @@ mod test {
+      scopes: "scope_token_user".to_string(),
@@ -1142 +1137 @@ mod test {
-      token_id: "token123".to_string(),
+      token_prefix: "bodhiapp_test".to_string(),
@@ -1143,0 +1139 @@ mod test {
+      scopes: "scope_token_user".to_string(),
@@ -1164 +1160 @@ mod test {
-    assert_eq!(updated.token_id, token.token_id);
+    assert_eq!(updated.token_prefix, token.token_prefix);
@@ -1171,39 +1166,0 @@ mod test {
-  #[rstest]
-  #[awt]
-  #[tokio::test]
-  async fn test_create_api_token_from(
-    #[future]
-    #[from(test_db_service)]
-    db_service: TestDbService,
-  ) -> anyhow::Result<()> {
-    // Create a test token with known claims
-    let test_jti = Uuid::new_v4().to_string();
-    let test_sub = Uuid::new_v4().to_string();
-    let (token, _) = build_token(serde_json::json!({
-      "jti": test_jti,
-      "sub": test_sub,
-    }))?;
-
-    // Create API token
-    let name = "Test Token";
-    let api_token = db_service.create_api_token_from(name, &token).await?;
-
-    // Verify the created token
-    assert_eq!(api_token.name, name);
-    assert_eq!(api_token.token_id, test_jti);
-    assert_eq!(api_token.user_id, test_sub);
-    assert_eq!(api_token.status, TokenStatus::Active);
-
-    // Verify we can retrieve it
-    let retrieved = db_service
-      .get_api_token_by_id(&test_sub, &api_token.id)
-      .await?;
-    assert!(retrieved.is_some());
-    let retrieved = retrieved.unwrap();
-    assert_eq!(retrieved.token_id, test_jti);
-    assert_eq!(retrieved.user_id, test_sub);
-    assert_eq!(retrieved.token_hash, api_token.token_hash);
-
-    Ok(())
-  }
-
@@ -1229 +1186 @@ mod test {
-      token_id: Uuid::new_v4().to_string(),
+      token_prefix: Uuid::new_v4().to_string(),
@@ -1230,0 +1188 @@ mod test {
+      scopes: "scope_token_user".to_string(),
@@ -1242 +1200 @@ mod test {
-      token_id: Uuid::new_v4().to_string(),
+      token_prefix: Uuid::new_v4().to_string(),
@@ -1243,0 +1202 @@ mod test {
+      scopes: "scope_token_user".to_string(),
@@ -1283 +1242 @@ mod test {
-      token_id: Uuid::new_v4().to_string(),
+      token_prefix: Uuid::new_v4().to_string(),
@@ -1284,0 +1244 @@ mod test {
+      scopes: "scope_token_user".to_string(),
diff --git a/crates/services/src/hub_service.rs b/crates/services/src/hub_service.rs
index bf6af12a..9c8103e9 100644
--- a/crates/services/src/hub_service.rs
+++ b/crates/services/src/hub_service.rs
@@ -3,2 +3,2 @@ use objs::{
-  impl_error_from, UserAlias, UserAliasBuilder, AliasSource, AppError, ErrorType, HubFile, IoError,
-  ObjValidationError, Repo,
+  impl_error_from, AliasSource, AppError, ErrorType, HubFile, IoError, ObjValidationError, Repo,
+  UserAlias, UserAliasBuilder,
diff --git a/crates/services/src/test_utils/data.rs b/crates/services/src/test_utils/data.rs
index ebfa063b..b11c6fc8 100644
--- a/crates/services/src/test_utils/data.rs
+++ b/crates/services/src/test_utils/data.rs
@@ -5 +5 @@ use crate::{
-use objs::{test_utils::temp_bodhi_home, UserAlias, RemoteModel};
+use objs::{test_utils::temp_bodhi_home, RemoteModel, UserAlias};
diff --git a/crates/services/src/test_utils/db.rs b/crates/services/src/test_utils/db.rs
index 665f3e8f..073055b5 100644
--- a/crates/services/src/test_utils/db.rs
+++ b/crates/services/src/test_utils/db.rs
@@ -169,8 +168,0 @@ impl DbService for TestDbService {
-  async fn create_api_token_from(&self, name: &str, token: &str) -> Result<ApiToken, DbError> {
-    self
-      .inner
-      .create_api_token_from(name, token)
-      .await
-      .tap(|_| self.notify("create_api_token_from"))
-  }
-
@@ -202 +194 @@ impl DbService for TestDbService {
-  async fn get_api_token_by_token_id(&self, token_id: &str) -> Result<Option<ApiToken>, DbError> {
+  async fn get_api_token_by_prefix(&self, prefix: &str) -> Result<Option<ApiToken>, DbError> {
@@ -205 +197 @@ impl DbService for TestDbService {
-      .get_api_token_by_token_id(token_id)
+      .get_api_token_by_prefix(prefix)
@@ -207 +199 @@ impl DbService for TestDbService {
-      .tap(|_| self.notify("get_api_token_by_token_id"))
+      .tap(|_| self.notify("get_api_token_by_prefix"))
@@ -230,5 +222 @@ impl DbService for TestDbService {
-  async fn create_api_model_alias(
-    &self,
-    alias: &ApiAlias,
-    api_key: &str,
-  ) -> Result<(), DbError> {
+  async fn create_api_model_alias(&self, alias: &ApiAlias, api_key: &str) -> Result<(), DbError> {
diff --git a/crates/services/src/test_utils/hf.rs b/crates/services/src/test_utils/hf.rs
index cfdfd918..b4af7d22 100644
--- a/crates/services/src/test_utils/hf.rs
+++ b/crates/services/src/test_utils/hf.rs
@@ -3 +3 @@ use derive_new::new;
-use objs::{test_utils::temp_hf_home, UserAlias, HubFile, Repo};
+use objs::{test_utils::temp_hf_home, HubFile, Repo, UserAlias};
