use crate::{HttpError, HttpErrorBuilder, RouterStateFn};
use async_openai::types::{ListModelResponse, Model};
use axum::{
  extract::{Path, State},
  Json,
};
use objs::Alias;
use std::{fs, sync::Arc, time::UNIX_EPOCH};

pub async fn oai_models_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Json<ListModelResponse>, HttpError> {
  let models = state
    .app_service()
    .data_service()
    .list_aliases()
    .map_err(|err| {
      HttpErrorBuilder::default()
        .internal_server(Some(&err.to_string()))
        .build()
        .unwrap()
    })?
    .into_iter()
    .map(|alias| to_oai_model(state.clone(), alias))
    .collect::<Vec<_>>();
  Ok(Json(ListModelResponse {
    object: "list".to_string(),
    data: models,
  }))
}

pub async fn oai_model_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
  Path(id): Path<String>,
) -> Result<Json<Model>, HttpError> {
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&id)
    .ok_or_else(|| {
      HttpErrorBuilder::default()
        .not_found(&format!("The model '{id}' does not exist"))
        .code("model_not_found")
        .param("model")
        .build()
        .unwrap()
    })?;
  let model = to_oai_model(state, alias);
  Ok(Json(model))
}

fn to_oai_model(state: Arc<dyn RouterStateFn>, alias: Alias) -> Model {
  let bodhi_home = &state.app_service().env_service().bodhi_home();
  let path = bodhi_home.join("aliases").join(alias.config_filename());
  let created = fs::metadata(path)
    .map_err(|e| e.to_string())
    .and_then(|m| m.created().map_err(|e| e.to_string()))
    .and_then(|t| t.duration_since(UNIX_EPOCH).map_err(|e| e.to_string()))
    .unwrap_or_default()
    .as_secs() as u32;
  Model {
    id: alias.alias,
    object: "model".to_string(),
    created,
    owned_by: "system".to_string(),
  }
}
