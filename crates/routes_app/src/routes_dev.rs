use axum::{body::Body, extract::State, response::Response};
use objs::{AppError, SerdeJsonError};
use serde_json::json;
use server_core::RouterState;
use services::{
  get_secret, AppRegInfo, SecretServiceError, KEY_APP_AUTHZ, KEY_APP_REG_INFO, KEY_APP_STATUS,
};
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
) -> Result<Response, DevError> {
  let secret_service = state.app_service().secret_service();
  let value = json! {{
    "authz": secret_service.get_secret_string(KEY_APP_AUTHZ)?,
    "status": secret_service.get_secret_string(KEY_APP_STATUS)?,
    "app_info": get_secret::<_, AppRegInfo>(secret_service.clone(), KEY_APP_REG_INFO)?,
  }};
  Ok(
    Response::builder()
      .header("Content-Type", "application/json")
      .body(Body::from(value.to_string()))
      .unwrap(),
  )
}
