use auth_middleware::{SESSION_KEY_ACCESS_TOKEN, SESSION_KEY_REFRESH_TOKEN};
use axum::{
  body::Body,
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use objs::{ApiError, AppError, SerdeJsonError};
use serde_json::json;
use server_core::RouterState;
use services::{SecretServiceError, SecretServiceExt};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DevError {
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError),
}

pub async fn dev_secrets_handler(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Response, ApiError> {
  let secret_service = state.app_service().secret_service();

  // Read session tokens
  let access_token = session
    .get::<String>(SESSION_KEY_ACCESS_TOKEN)
    .await
    .ok()
    .flatten();
  let refresh_token = session
    .get::<String>(SESSION_KEY_REFRESH_TOKEN)
    .await
    .ok()
    .flatten();

  #[allow(unused_mut)]
  let mut value = json! {{
    "status": secret_service.app_status()?,
    "app_info": secret_service.app_reg_info()?,
    "session": {
      "access_token": access_token,
      "refresh_token": refresh_token,
    }
  }};
  #[cfg(debug_assertions)]
  {
    value["dump"] = serde_json::Value::String(secret_service.dump()?);
  }
  Ok(
    Response::builder()
      .header("Content-Type", "application/json")
      .body(Body::from(value.to_string()))
      .unwrap(),
  )
}

pub async fn envs_handler(State(state): State<Arc<dyn RouterState>>) -> Result<Response, ApiError> {
  let envs = state
    .app_service()
    .setting_service()
    .list()
    .into_iter()
    .collect::<Vec<_>>();
  Ok((StatusCode::OK, Json(envs)).into_response())
}
