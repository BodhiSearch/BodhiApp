use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub jti: String,
  pub exp: u64,
  pub email: String,
}

pub fn decode_access_token(
  access_token: &str,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
  let mut validation = Validation::default();
  validation.insecure_disable_signature_validation();
  validation.validate_exp = false;
  let token_data = jsonwebtoken::decode::<Claims>(
    access_token,
    &DecodingKey::from_secret(&[]), // dummy key for parsing
    &validation,
  )?;
  Ok(token_data)
}
