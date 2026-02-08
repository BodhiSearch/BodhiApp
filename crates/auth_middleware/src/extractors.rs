use axum::{extract::FromRequestParts, http::request::Parts};
use objs::{ApiError, AppError, ErrorType, ResourceRole};
use std::str::FromStr;

use crate::{
  KEY_HEADER_BODHIAPP_ROLE, KEY_HEADER_BODHIAPP_SCOPE, KEY_HEADER_BODHIAPP_TOKEN,
  KEY_HEADER_BODHIAPP_USERNAME, KEY_HEADER_BODHIAPP_USER_ID,
};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HeaderExtractionError {
  #[error("Required header '{header}' is missing.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Missing { header: String },

  #[error("Header '{header}' contains invalid value: {reason}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Invalid { header: String, reason: String },

  #[error("Header '{header}' value is empty.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Empty { header: String },
}

fn extract_required_header(parts: &Parts, header: &str) -> Result<String, HeaderExtractionError> {
  let value = parts
    .headers
    .get(header)
    .ok_or_else(|| HeaderExtractionError::Missing {
      header: header.to_string(),
    })?;
  let value = value.to_str().map_err(|e| HeaderExtractionError::Invalid {
    header: header.to_string(),
    reason: e.to_string(),
  })?;
  if value.is_empty() {
    return Err(HeaderExtractionError::Empty {
      header: header.to_string(),
    });
  }
  Ok(value.to_string())
}

fn extract_optional_header(
  parts: &Parts,
  header: &str,
) -> Result<Option<String>, HeaderExtractionError> {
  match parts.headers.get(header) {
    None => Ok(None),
    Some(value) => {
      let value = value.to_str().map_err(|e| HeaderExtractionError::Invalid {
        header: header.to_string(),
        reason: e.to_string(),
      })?;
      if value.is_empty() {
        Ok(None)
      } else {
        Ok(Some(value.to_string()))
      }
    }
  }
}

/// Extracts the authentication token from the request headers.
pub struct ExtractToken(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ExtractToken {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let token =
      extract_required_header(parts, KEY_HEADER_BODHIAPP_TOKEN).map_err(ApiError::from)?;
    Ok(ExtractToken(token))
  }
}

/// Extracts the user ID from the request headers.
pub struct ExtractUserId(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ExtractUserId {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let user_id =
      extract_required_header(parts, KEY_HEADER_BODHIAPP_USER_ID).map_err(ApiError::from)?;
    Ok(ExtractUserId(user_id))
  }
}

/// Extracts the username from the request headers.
pub struct ExtractUsername(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ExtractUsername {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let username =
      extract_required_header(parts, KEY_HEADER_BODHIAPP_USERNAME).map_err(ApiError::from)?;
    Ok(ExtractUsername(username))
  }
}

/// Extracts the resource role from the request headers.
pub struct ExtractRole(pub ResourceRole);

impl<S: Send + Sync> FromRequestParts<S> for ExtractRole {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let role_str =
      extract_required_header(parts, KEY_HEADER_BODHIAPP_ROLE).map_err(ApiError::from)?;
    let role = ResourceRole::from_str(&role_str).map_err(|e| {
      ApiError::from(HeaderExtractionError::Invalid {
        header: KEY_HEADER_BODHIAPP_ROLE.to_string(),
        reason: e.to_string(),
      })
    })?;
    Ok(ExtractRole(role))
  }
}

/// Extracts the scope from the request headers.
pub struct ExtractScope(pub String);

impl<S: Send + Sync> FromRequestParts<S> for ExtractScope {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let scope =
      extract_required_header(parts, KEY_HEADER_BODHIAPP_SCOPE).map_err(ApiError::from)?;
    Ok(ExtractScope(scope))
  }
}

/// Optionally extracts the authentication token from the request headers.
/// Never fails - returns None if the header is missing or empty.
pub struct MaybeToken(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for MaybeToken {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let token =
      extract_optional_header(parts, KEY_HEADER_BODHIAPP_TOKEN).map_err(ApiError::from)?;
    Ok(MaybeToken(token))
  }
}

/// Optionally extracts the resource role from the request headers.
/// Returns None if the header is missing. Returns an error if the header is present but invalid.
pub struct MaybeRole(pub Option<ResourceRole>);

impl<S: Send + Sync> FromRequestParts<S> for MaybeRole {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let role_str =
      extract_optional_header(parts, KEY_HEADER_BODHIAPP_ROLE).map_err(ApiError::from)?;
    match role_str {
      None => Ok(MaybeRole(None)),
      Some(s) => {
        let role = ResourceRole::from_str(&s).map_err(|e| {
          ApiError::from(HeaderExtractionError::Invalid {
            header: KEY_HEADER_BODHIAPP_ROLE.to_string(),
            reason: e.to_string(),
          })
        })?;
        Ok(MaybeRole(Some(role)))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
  };
  use objs::test_utils::parse_txt;
  use rstest::rstest;
  use tower::ServiceExt;

  async fn token_handler(ExtractToken(token): ExtractToken) -> String {
    token
  }

  async fn user_id_handler(ExtractUserId(user_id): ExtractUserId) -> String {
    user_id
  }

  async fn role_handler(ExtractRole(role): ExtractRole) -> String {
    role.to_string()
  }

  async fn maybe_token_handler(MaybeToken(token): MaybeToken) -> String {
    token.unwrap_or_else(|| "none".to_string())
  }

  async fn maybe_role_handler(MaybeRole(role): MaybeRole) -> String {
    role
      .map(|r| r.to_string())
      .unwrap_or_else(|| "none".to_string())
  }

  #[rstest]
  #[tokio::test]
  async fn test_extract_token_present() {
    let router = Router::new().route("/", get(token_handler));
    let req = Request::get("/")
      .header(KEY_HEADER_BODHIAPP_TOKEN, "my-token")
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = parse_txt(response).await;
    assert_eq!("my-token", body);
  }

  #[rstest]
  #[tokio::test]
  async fn test_extract_token_missing() {
    let router = Router::new().route("/", get(token_handler));
    let req = Request::get("/").body(Body::empty()).unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, response.status());
  }

  #[rstest]
  #[tokio::test]
  async fn test_extract_user_id_present() {
    let router = Router::new().route("/", get(user_id_handler));
    let req = Request::get("/")
      .header(KEY_HEADER_BODHIAPP_USER_ID, "user-123")
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = parse_txt(response).await;
    assert_eq!("user-123", body);
  }

  #[rstest]
  #[tokio::test]
  async fn test_extract_role_valid() {
    let router = Router::new().route("/", get(role_handler));
    let req = Request::get("/")
      .header(KEY_HEADER_BODHIAPP_ROLE, "resource_admin")
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = parse_txt(response).await;
    assert_eq!("resource_admin", body);
  }

  #[rstest]
  #[tokio::test]
  async fn test_extract_role_invalid() {
    let router = Router::new().route("/", get(role_handler));
    let req = Request::get("/")
      .header(KEY_HEADER_BODHIAPP_ROLE, "invalid_role")
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, response.status());
  }

  #[rstest]
  #[tokio::test]
  async fn test_maybe_token_present() {
    let router = Router::new().route("/", get(maybe_token_handler));
    let req = Request::get("/")
      .header(KEY_HEADER_BODHIAPP_TOKEN, "my-token")
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = parse_txt(response).await;
    assert_eq!("my-token", body);
  }

  #[rstest]
  #[tokio::test]
  async fn test_maybe_token_missing() {
    let router = Router::new().route("/", get(maybe_token_handler));
    let req = Request::get("/").body(Body::empty()).unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = parse_txt(response).await;
    assert_eq!("none", body);
  }

  #[rstest]
  #[tokio::test]
  async fn test_maybe_role_present() {
    let router = Router::new().route("/", get(maybe_role_handler));
    let req = Request::get("/")
      .header(KEY_HEADER_BODHIAPP_ROLE, "resource_user")
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = parse_txt(response).await;
    assert_eq!("resource_user", body);
  }

  #[rstest]
  #[tokio::test]
  async fn test_maybe_role_missing() {
    let router = Router::new().route("/", get(maybe_role_handler));
    let req = Request::get("/").body(Body::empty()).unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let body = parse_txt(response).await;
    assert_eq!("none", body);
  }

  #[rstest]
  #[tokio::test]
  async fn test_maybe_role_invalid() {
    let router = Router::new().route("/", get(maybe_role_handler));
    let req = Request::get("/")
      .header(KEY_HEADER_BODHIAPP_ROLE, "bad_role")
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, response.status());
  }
}
