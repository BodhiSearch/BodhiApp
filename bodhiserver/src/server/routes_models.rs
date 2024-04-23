use super::utils::ApiError;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Model {
  model: String,
  #[serde(rename = "displayName")]
  display_name: String,
}

pub(crate) async fn ui_models_handler() -> Result<Json<Vec<String>>, ApiError> {
  // let models = _ui_models_handler()?;
  let models = vec!["llama2-7b".to_string(), "llama2-13b".to_string()];
  Ok(Json(models))
}

fn _ui_models_handler() -> Result<Vec<Model>, ApiError> {
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
