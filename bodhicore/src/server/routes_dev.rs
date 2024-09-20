use super::RouterStateFn;
use crate::server::{HttpError, HttpErrorBuilder};
use axum::{
  body::Body,
  extract::State,
  response::{IntoResponse, Response},
};
use objs::AppRegInfo;
use serde_json::json;
use services::{get_secret, SecretServiceError, KEY_APP_AUTHZ, KEY_APP_REG_INFO, KEY_APP_STATUS};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum DevError {
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error("serde_json: {0}")]
  SerdeJson(String),
}

impl From<serde_json::Error> for DevError {
  fn from(value: serde_json::Error) -> Self {
    DevError::SerdeJson(value.to_string())
  }
}

impl From<DevError> for HttpError {
  fn from(value: DevError) -> Self {
    HttpErrorBuilder::default()
      .internal_server(Some(&format!("{:?}", value)))
      .build()
      .unwrap()
  }
}

impl IntoResponse for DevError {
  fn into_response(self) -> axum::response::Response {
    HttpError::from(self).into_response()
  }
}

pub async fn dev_secrets_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
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
      .body(Body::from(serde_json::to_string(&value)?))
      .unwrap(),
  )
}
