use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use rsa::{
  pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
  pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePublicKey},
  RsaPrivateKey, RsaPublicKey,
};
use rstest::fixture;
use serde_json::json;
use uuid::Uuid;

use crate::AppRegInfoBuilder;

static PUBLIC_KEY: Lazy<RsaPublicKey> = Lazy::new(|| {
  RsaPublicKey::from_public_key_pem(include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/test_public_key.pem"
  )))
  .expect("Failed to parse public key")
});

static PUBLIC_KEY_BASE64: Lazy<String> = Lazy::new(|| {
  let public_key_der = PUBLIC_KEY
    .to_public_key_der()
    .expect("Failed to convert to DER");
  STANDARD.encode(public_key_der.as_ref())
});

static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
  RsaPrivateKey::from_pkcs8_pem(include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/test_private_key.pem"
  )))
  .expect("Failed to parse private key")
});

impl AppRegInfoBuilder {
  pub fn test_default() -> Self {
    Self::default()
      .public_key(PUBLIC_KEY_BASE64.to_string())
      .issuer("https://id.mydomain.com/realms/myapp".to_string())
      .client_id("test-client".to_string())
      .client_secret("test-client-secret".to_string())
      .alg(Algorithm::RS256)
      .kid("test-kid".to_string())
      .to_owned()
  }
}

#[fixture]
pub fn token() -> (String, String, String) {
  build_token((Utc::now() + Duration::hours(1)).timestamp()).unwrap()
}

#[fixture]
pub fn expired_token() -> (String, String, String) {
  build_token((Utc::now() - Duration::hours(1)).timestamp()).unwrap()
}

pub fn build_token(exp: i64) -> anyhow::Result<(String, String, String)> {
  let jti = Uuid::new_v4().to_string();
  let claims = json!({
      "exp": exp,
      "jti": jti,
      "iss": "https://id.mydomain.com/realms/myapp".to_string(),
      "sub": Uuid::new_v4().to_string(),
      "typ": "Bearer",
      "azp": "test-client",
      "session_state": Uuid::new_v4().to_string(),
      "scope": "openid scope_user profile email scope_power_user",
      "sid": Uuid::new_v4().to_string(),
      "email_verified": true,
      "name": "Test User",
      "preferred_username": "testuser@email.com",
      "given_name": "Test",
      "family_name": "User",
      "email": "testuser@email.com"
  });

  let header = Header {
    kid: Some("test-kid".to_string()),
    alg: jsonwebtoken::Algorithm::RS256,
    ..Default::default()
  };

  let pem_file = PRIVATE_KEY.to_pkcs1_pem(rsa::pkcs8::LineEnding::CRLF)?;
  let token = encode(
    &header,
    &claims,
    &EncodingKey::from_rsa_pem(pem_file.as_bytes())?,
  )?;

  let output = STANDARD.encode(PUBLIC_KEY.to_pkcs1_der()?);
  Ok((jti, token, output))
}
