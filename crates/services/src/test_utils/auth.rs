use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rsa::{
  pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
  RsaPrivateKey, RsaPublicKey,
};
use rstest::fixture;
use serde_json::json;
use uuid::Uuid;

#[fixture]
#[once]
pub fn key_pair() -> (RsaPrivateKey, RsaPublicKey) {
  let mut rng = rand::thread_rng();
  let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
  let public_key = private_key.to_public_key();
  (private_key, public_key)
}

#[fixture]
pub fn token(key_pair: &(RsaPrivateKey, RsaPublicKey)) -> anyhow::Result<(String, String, String)> {
  build_token(key_pair, (Utc::now() + Duration::hours(1)).timestamp())
}

#[fixture]
pub fn expired_token(
  key_pair: &(RsaPrivateKey, RsaPublicKey),
) -> anyhow::Result<(String, String, String)> {
  build_token(key_pair, (Utc::now() - Duration::hours(1)).timestamp())
}

pub fn build_token(
  key_pair: &(RsaPrivateKey, RsaPublicKey),
  exp: i64,
) -> anyhow::Result<(String, String, String)> {
  let (private_key, public_key) = key_pair;
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

  let pem_file = private_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::CRLF)?;
  let token = encode(
    &header,
    &claims,
    &EncodingKey::from_rsa_pem(pem_file.as_bytes())?,
  )?;

  let output = STANDARD.encode(public_key.to_pkcs1_der()?);
  Ok((jti, token, output))
}
