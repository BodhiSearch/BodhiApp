use crate::oai::api_error::OaiApiError;
use axum::{body::to_bytes, response::IntoResponse};
use errmeta_derive::ErrorMeta;
use pretty_assertions::assert_eq;
use serde_json::{json, Value};
use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
enum SyntheticError {
  #[error("Bad input: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  BadInput(String),
}

#[tokio::test]
async fn test_oai_api_error_serializes_as_wrapped_error() {
  let err: OaiApiError = SyntheticError::BadInput("even".to_string()).into();
  assert_eq!(400, err.status);
  let response = err.into_response();
  assert_eq!(400, response.status().as_u16());
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body: Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(
    json!({
      "error": {
        "message": "Bad input: even.",
        "type": "invalid_request_error",
        "code": "synthetic_error-bad_input",
        "param": "var_0=even"
      }
    }),
    body
  );
}

#[tokio::test]
async fn test_oai_api_error_omits_param_when_no_args() {
  #[derive(Debug, thiserror::Error, ErrorMeta)]
  #[error_meta(trait_to_impl = AppError)]
  enum NoArgs {
    #[error("oh no")]
    #[error_meta(error_type = ErrorType::InternalServer)]
    Boom,
  }

  let err: OaiApiError = NoArgs::Boom.into();
  let response = err.into_response();
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body: Value = serde_json::from_slice(&bytes).unwrap();
  assert!(body["error"]["param"].is_null());
}
