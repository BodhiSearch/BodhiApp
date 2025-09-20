use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::errors::ErrorKind;
use objs::{impl_error_from, AppError, ErrorType, SerdeJsonError};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, result::Result};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::Authentication, code=self.code())]
#[error("json_web_token_error")]
pub struct JsonWebTokenError {
  #[from]
  source: jsonwebtoken::errors::Error,
}

impl JsonWebTokenError {
  fn code(&self) -> String {
    match self.source.kind() {
      ErrorKind::InvalidToken
      | ErrorKind::InvalidSignature
      | ErrorKind::InvalidIssuer
      | ErrorKind::InvalidAudience => {
        format!("json_web_token_error-{:?}", self.source.kind())
      }
      _ => "json_web_token_error-Unknown".to_string(),
    }
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum TokenError {
  #[error("invalid_token")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidToken(String),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
  #[error("invalid_issuer")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidIssuer(String),
  #[error("scope_empty")]
  #[error_meta(error_type = ErrorType::Authentication)]
  ScopeEmpty,
  #[error("expired")]
  #[error_meta(error_type = ErrorType::Authentication)]
  Expired,
  #[error("invalid_audience")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidAudience(String),
}

impl_error_from!(
  serde_json::Error,
  TokenError::SerdeJson,
  ::objs::SerdeJsonError
);

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceClaims {
  pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserIdClaims {
  pub jti: String,
  pub sub: String,
  pub preferred_username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScopeClaims {
  pub iss: String,
  pub azp: String,
  pub aud: Option<String>,
  pub exp: u64,
  pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdClaims {
  pub jti: String,
  pub sub: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpClaims {
  pub jti: String,
  pub sub: String,
  pub exp: u64,
  pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub exp: u64,
  pub iat: u64,
  pub jti: String,
  pub iss: String,
  pub sub: String,
  pub typ: String,
  pub azp: String,
  pub aud: Option<String>,
  pub scope: String,
  pub preferred_username: String,
  pub given_name: Option<String>,
  pub family_name: Option<String>,
  #[serde(default)]
  pub resource_access: HashMap<String, ResourceClaims>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfflineClaims {
  pub iat: u64,
  pub jti: String,
  pub iss: String,
  pub sub: String,
  pub typ: String,
  pub azp: String,
  pub scope: String,
}

pub fn extract_claims<T: for<'de> Deserialize<'de>>(access_token: &str) -> Result<T, TokenError> {
  let mut token_iter = access_token.splitn(3, '.');
  match (token_iter.next(), token_iter.next(), token_iter.next()) {
    (Some(_), Some(claims), Some(_)) => {
      let claims = URL_SAFE_NO_PAD
        .decode(claims)
        .map_err(|e| TokenError::InvalidToken(e.to_string()))?;
      let claims: T = serde_json::from_slice(&claims)?;
      Ok(claims)
    }
    _ => Err(TokenError::InvalidToken(
      "malformed token format".to_string(),
    )),
  }
}

#[cfg(test)]
mod tests {
  use crate::{extract_claims, test_utils::build_token, TokenError};
  use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
  use chrono::Utc;
  use objs::{test_utils::setup_l10n, FluentLocalizationService};
  use rstest::rstest;
  use serde::Deserialize;
  use serde_json::{json, Value};
  use std::{collections::HashMap, sync::Arc};
  use time::OffsetDateTime;

  #[derive(Debug, Deserialize, PartialEq)]
  struct TestClaims {
    sub: String,
    exp: i64,
    iat: i64,
    resource_access: HashMap<String, ResourceAccess>,
  }

  #[derive(Debug, Deserialize, PartialEq)]
  struct ResourceAccess {
    roles: Vec<String>,
  }

  #[derive(Debug, Deserialize, PartialEq)]
  struct MismatchClaims {
    different_field: String,
  }

  fn invalid_signature(claims: &Value, header: &str) -> String {
    // Concatenate parts with dots - header.claims.signature
    format!(
      "{}.{}.{}",
      URL_SAFE_NO_PAD.encode(header),
      URL_SAFE_NO_PAD.encode(claims.to_string()),
      "signature"
    )
  }

  #[test]
  fn test_extract_claims_token_valid() -> anyhow::Result<()> {
    let claims = json! {{
      "sub": "1234",
      "exp": Utc::now().timestamp(),
      "iat": Utc::now().timestamp() + 3600,
      "resource_access": {
        "test-client": {
          "roles": ["resource_user", "resource_power_user"]
        }
      }
    }};
    let token = invalid_signature(&claims, r#"{"alg":"HS256","typ":"JWT"}"#);
    println!("token: {}", token);
    let claims = extract_claims::<TestClaims>(&token)?;
    assert_eq!(
      claims.resource_access["test-client"].roles,
      vec!["resource_user", "resource_power_user"]
    );
    Ok(())
  }

  #[test]
  fn test_extract_claims_token_tampered_signature() {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let claims = json! {{
      "sub": "1234",
      "exp": now + 3600,
      "iat": now - 3600,
      "resource_access": {
        "test-client": {
          "roles": ["resource_user"]
        }
      }
    }};
    let token = format!(
      "{}.{}.{}",
      URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#),
      URL_SAFE_NO_PAD.encode(claims.to_string()),
      "tampered_signature"
    );

    // Should still work since we disabled signature validation
    let result = extract_claims::<TestClaims>(&token);
    assert!(result.is_ok());
  }

  #[test]
  fn test_extract_claims_token_tampered_claims() -> anyhow::Result<()> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let claims = json! {{
      "sub": "1234",
      "exp": now + 3600,
      "iat": now - 3600,
      "resource_access": {
        "test-client": {
          "roles": ["resource_user"]
        }
      }
    }};
    let (token, _) = build_token(claims)?;
    let mut token_splits = token.splitn(3, '.');
    let (header, _, signature) = (
      token_splits.next().unwrap(),
      token_splits.next().unwrap(),
      token_splits.next().unwrap(),
    );
    let tampered_claims = json! {{
      "sub": "1234",
      "exp": now + 3600,
      "iat": now - 3600,
      "resource_access": {
        "test-client": {
          "roles": ["resource_admin"]
        }
      }
    }};
    let tampered_token = format!(
      "{}.{}.{}",
      header,
      URL_SAFE_NO_PAD.encode(tampered_claims.to_string()),
      signature
    );

    // Should work since we're just decoding
    let claims = extract_claims::<TestClaims>(&tampered_token)?;
    assert_eq!(
      claims.resource_access["test-client"].roles,
      vec!["resource_admin".to_string()]
    );
    Ok(())
  }

  #[test]
  fn test_extract_claims_token_expired() -> anyhow::Result<()> {
    let past = Utc::now().timestamp() - 3600;
    let claims = json! {{
      "sub": "1234",
      "exp": past,
      "iat": past,
      "resource_access": {
        "test-client": {
          "roles": ["resource_user"]
        }
      }
    }};
    let (token, _) = build_token(claims)?;
    let claims = extract_claims::<TestClaims>(&token)?;
    assert_eq!(claims.sub, "1234");
    Ok(())
  }

  #[test]
  fn test_extract_claims_token_future_iat() -> anyhow::Result<()> {
    let future = Utc::now().timestamp() + 3600;
    let claims = json! {{
      "sub": "1234",
      "iat": future,
      "exp": future,
      "resource_access": {
        "test-client": {
          "roles": ["resource_user"]
        }
      }
    }};
    let (token, _) = build_token(claims)?;
    // Should work since we're not validating iat
    let claims = extract_claims::<TestClaims>(&token)?;
    assert_eq!(claims.sub, "1234");
    Ok(())
  }

  #[rstest]
  fn test_extract_claims_token_malformed_payload(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let token = format!(
      "{}.{}.{}",
      URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#),
      URL_SAFE_NO_PAD.encode("not a json payload"),
      "signature"
    );
    let result = extract_claims::<TestClaims>(&token);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TokenError::SerdeJson(_)));
    Ok(())
  }

  #[rstest]
  fn test_extract_claims_token_type_mismatch(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let now = Utc::now().timestamp();
    let claims = json! {{
      "sub": "1234",
      "iat": now - 3600,
      "exp": now + 3600,
      "resource_access": {
        "test-client": {
          "roles": ["resource_user"]
        }
      }
    }};
    let (token, _) = build_token(claims)?;
    // Try to decode into a type that doesn't match the payload structure
    let result = extract_claims::<MismatchClaims>(&token);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TokenError::SerdeJson(_)));
    Ok(())
  }
}
