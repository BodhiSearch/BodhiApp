use std::sync::Arc;

use super::{router_state::RouterState, utils::ApiError, RouterStateFn};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Model {
  model: String,
  #[serde(rename = "displayName")]
  display_name: String,
}

pub(crate) async fn ui_models_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Json<Vec<Model>>, ApiError> {
  // let models = state
  //   .app_service
  //   .list_aliases()
  //   .unwrap()
  //   .into_iter()
  //   .map(|alias| Model {
  //     model: alias.alias,
  //     display_name: alias.filename,
  //   })
  //   .collect::<Vec<_>>();
  // TODO: fix, add list_aliases to routerstatefn
  Ok(Json(vec![]))
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
