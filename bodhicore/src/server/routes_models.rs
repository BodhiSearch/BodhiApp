use super::RouterStateFn;
use crate::{oai::OpenAIApiError, objs::Alias};
use axum::{extract::State, Json};
use std::sync::Arc;

pub(crate) async fn ui_models_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Json<Vec<Alias>>, OpenAIApiError> {
  let aliases = state
    .app_service()
    .list_aliases()
    .map_err(|err| OpenAIApiError::InternalServer(err.to_string()))?;
  Ok(Json(aliases))
}
