use jsonwebtoken::{DecodingKey, Validation};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub(crate) fn to_safe_filename(input: &str) -> String {
  let illegal_chars = Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
  let mut sanitized = illegal_chars.replace_all(input, "--").to_string();
  sanitized = sanitized
    .chars()
    .filter(|c| !c.is_control() && !c.is_whitespace())
    .collect();
  if sanitized.len() > 255 {
    sanitized.truncate(255);
  }
  sanitized
}

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
