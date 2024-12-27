use axum::{body::Body, extract::State, response::Response};
use objs::{ApiError, AppError, SerdeJsonError};
use serde_json::json;
use server_core::RouterState;
use services::{AppStatus, SecretServiceError, SecretServiceExt};
use std::sync::Arc;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DevError {
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
}

pub async fn dev_secrets_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let secret_service = state.app_service().secret_service();
  let value = json! {{
    "authz": secret_service.authz_or_default().to_string(),
    "status": secret_service.app_status().unwrap_or(AppStatus::default()).to_string(),
    "app_info": secret_service.app_reg_info().unwrap_or(None),
  }};
  Ok(
    Response::builder()
      .header("Content-Type", "application/json")
      .body(Body::from(value.to_string()))
      .unwrap(),
  )
}

// TODO: write tests
