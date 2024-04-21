use super::utils;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Model {
  model: String,
  #[serde(rename = "displayName")]
  display_name: String,
}

#[derive(Debug, Error)]
pub(crate) enum ModelError {
  #[error(transparent)]
  HomeDirError(#[from] utils::HomeDirError),
}

impl IntoResponse for ModelError {
  fn into_response(self) -> Response<Body> {
    Json(super::utils::ApiError {
      error: format!("{}", self),
    })
    .into_response()
  }
}

pub(crate) async fn ui_models_handler() -> Result<Json<Vec<Model>>, ModelError> {
  let models = _ui_models_handler()?;
  Ok(Json(models))
}

fn _ui_models_handler() -> Result<Vec<Model>, ModelError> {
  let models = vec![
    Model {
      model: "llama-2-7b-chat.Q4_K_M".to_string(),
      display_name: "llama2-7b".to_string(),
    },
    Model {
      model: "llama-2-13b-chat.gguf".to_string(),
      display_name: "llama2-13b".to_string(),
    },
  ];
  Ok(models)
}
