use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use rsa::{
  pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
  pkcs8::{DecodePrivateKey, DecodePublicKey},
  RsaPrivateKey, RsaPublicKey,
};
use rstest::fixture;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{AppRegInfoBuilder, KeycloakAuthService, TOKEN_TYPE_BEARER, TOKEN_TYPE_OFFLINE};

pub const TEST_CLIENT_ID: &str = "test-client";
pub const TEST_CLIENT_SECRET: &str = "test-client-secret";
pub const ISSUER: &str = "https://id.mydomain.com/realms/myapp";
pub const TEST_KID: &str = "test-kid";

static PUBLIC_KEY: Lazy<RsaPublicKey> = Lazy::new(|| {
  RsaPublicKey::from_public_key_pem(include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/test_public_key.pem"
  )))
  .expect("Failed to parse public key")
});

pub static PUBLIC_KEY_BASE64: Lazy<String> = Lazy::new(|| {
  let input = PUBLIC_KEY.to_pkcs1_der().expect("failed to convert to DER");
  STANDARD.encode(input)
});

pub static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
  RsaPrivateKey::from_pkcs8_pem(include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/test_private_key.pem"
  )))
  .expect("Failed to parse private key")
});

pub static OTHER_KEY: Lazy<RsaPublicKey> = Lazy::new(|| {
  RsaPublicKey::from_public_key_pem(include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/other_public_key.pem"
  )))
  .expect("Failed to parse public key")
});

pub static OTHER_PUBLIC_KEY_BASE64: Lazy<String> = Lazy::new(|| {
  let input = OTHER_KEY.to_pkcs1_der().expect("failed to convert to DER");
  STANDARD.encode(input)
});

pub static OTHER_PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
  RsaPrivateKey::from_pkcs8_pem(include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/other_private_key.pem"
  )))
  .expect("Failed to parse private key")
});

impl AppRegInfoBuilder {
  pub fn test_default() -> Self {
    Self::default()
      .client_id(TEST_CLIENT_ID.to_string())
      .client_secret(TEST_CLIENT_SECRET.to_string())
      .to_owned()
  }
}

pub fn access_token_claims() -> Value {
  access_token_with_exp(Utc::now().timestamp() + 3600)
}

#[fixture]
pub fn token() -> (String, String) {
  build_token_with_exp((Utc::now() + Duration::hours(1)).timestamp()).unwrap()
}

#[fixture]
pub fn expired_token() -> (String, String) {
  build_token_with_exp((Utc::now() - Duration::hours(1)).timestamp()).unwrap()
}

pub fn build_token_with_exp(exp: i64) -> anyhow::Result<(String, String)> {
  let claims = access_token_with_exp(exp);
  build_token(claims)
}

pub fn access_token_with_exp(exp: i64) -> Value {
  json!({
      "exp": exp,
      "iat": Utc::now().timestamp(),
      "jti": Uuid::new_v4().to_string(),
      "iss": "https://id.mydomain.com/realms/myapp".to_string(),
      "sub": Uuid::new_v4().to_string(),
      "typ": "Bearer",
      "azp": TEST_CLIENT_ID,
      "session_state": Uuid::new_v4().to_string(),
      "resource_access": {
        TEST_CLIENT_ID: {
          "roles": [
            "resource_manager",
            "resource_power_user",
            "resource_user",
            "resource_admin"
          ]
        }
      },
      "scope": "openid profile email roles",
      "sid": Uuid::new_v4().to_string(),
      "email_verified": true,
      "name": "Test User",
      "preferred_username": "testuser@email.com",
      "given_name": "Test",
      "family_name": "User"
  })
}

pub fn build_token(claims: Value) -> anyhow::Result<(String, String)> {
  sign_token(&PRIVATE_KEY, &PUBLIC_KEY, &claims)
}

pub fn sign_token(
  private_key: &RsaPrivateKey,
  public_key: &RsaPublicKey,
  claims: &Value,
) -> Result<(String, String), anyhow::Error> {
  let header = Header {
    kid: Some("test-kid".to_string()),
    alg: jsonwebtoken::Algorithm::RS256,
    ..Default::default()
  };
  let pem_file = private_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::CRLF)?;
  let token = encode(
    &header,
    claims,
    &EncodingKey::from_rsa_pem(pem_file.as_bytes())?,
  )?;

  let output = STANDARD.encode(public_key.to_pkcs1_der()?);
  Ok((token, output))
}

pub fn offline_token_claims() -> Value {
  json!({
      "exp": (Utc::now() + Duration::hours(1)).timestamp(),
      "iat": Utc::now().timestamp(),
      "jti": Uuid::new_v4().to_string(),
      "iss": ISSUER.to_string(),
      "sub": Uuid::new_v4().to_string(),
      "typ": TOKEN_TYPE_OFFLINE,
      "azp": TEST_CLIENT_ID,
      "session_state": Uuid::new_v4().to_string(),
      "scope": "openid offline_access scope_token_user",
      "sid": Uuid::new_v4().to_string(),
  })
}

pub fn offline_access_token_claims() -> Value {
  json!({
      "exp": (Utc::now() + Duration::hours(1)).timestamp(),
      "iat": Utc::now().timestamp(),
      "auth_time": Utc::now().timestamp(),
      "jti": Uuid::new_v4().to_string(),
      "iss": ISSUER.to_string(),
      "sub": Uuid::new_v4().to_string(),
      "typ": TOKEN_TYPE_BEARER,
      "azp": TEST_CLIENT_ID,
      "session_state": Uuid::new_v4().to_string(),
      "scope": "offline_access scope_token_user",
      "sid": Uuid::new_v4().to_string(),
  })
}

pub fn test_auth_service(url: &str) -> KeycloakAuthService {
  KeycloakAuthService::new("test-version", url.to_string(), "test-realm".to_string())
}
