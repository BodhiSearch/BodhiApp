use super::ENDPOINT_OAI_MODELS;
use async_openai::types::models::{ListModelResponse, Model};
use axum::{
  extract::{Path, State},
  Json,
};
use objs::{Alias, ApiAlias, ApiError, ModelAlias, OpenAIApiError, UserAlias, API_TAG_OPENAI};
use server_core::RouterState;
use services::DataServiceError;
use std::{collections::HashSet, sync::Arc};

/// List available models
#[utoipa::path(
    get,
    path = ENDPOINT_OAI_MODELS,
    tag = API_TAG_OPENAI,
    operation_id = "listModels",
    summary = "List Available Models (OpenAI Compatible)",
    description = "Returns a list of all available models in OpenAI API compatible format. Includes user aliases, model aliases, and API provider aliases that can be used with the chat completions endpoint.",
    responses(
        (status = 200, description = "List of available models",
         body = ListModelResponse,
         example = json!({
             "object": "list",
             "data": [
                 {
                     "id": "llama2:chat",
                     "object": "model",
                     "created": 1677610602,
                     "owned_by": "bodhi"
                 },
                 {
                     "id": "mistral:instruct",
                     "object": "model",
                     "created": 1677610602,
                     "owned_by": "bodhi"
                 }
             ]
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn oai_models_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<ListModelResponse>, ApiError> {
  // Get all aliases from unified DataService
  let aliases = state
    .app_service()
    .data_service()
    .list_aliases()
    .await
    .map_err(ApiError::from)?;

  // Use HashSet to track model IDs and prevent duplicates
  let mut seen_models = HashSet::new();
  let mut models = Vec::new();

  // Process aliases in priority order: User > Model > API
  for alias in aliases {
    match alias {
      Alias::User(user_alias) => {
        if seen_models.insert(user_alias.alias.clone()) {
          models.push(user_alias_to_oai_model(state.clone(), user_alias));
        }
      }
      Alias::Model(model_alias) => {
        if seen_models.insert(model_alias.alias.clone()) {
          models.push(model_alias_to_oai_model(state.clone(), model_alias));
        }
      }
      Alias::Api(api_alias) => {
        // Use matchable_models() which returns from models_cache when forward_all=true
        for model_id in api_alias.matchable_models() {
          if seen_models.insert(model_id.clone()) {
            models.push(api_model_to_oai_model(model_id, &api_alias));
          }
        }

        // If forward_all and cache is empty/stale, spawn async refresh
        if api_alias.forward_all_with_prefix
          && (api_alias.is_cache_empty() || api_alias.is_cache_stale())
        {
          let app_service = state.app_service();
          let alias_id = api_alias.id.clone();
          tokio::spawn(async move {
            let db = app_service.db_service();
            let ai_api = app_service.ai_api_service();
            let time_service = app_service.time_service();

            if let Ok(Some(alias)) = db.get_api_model_alias(&alias_id).await {
              let api_key = db.get_api_key_for_alias(&alias_id).await.ok().flatten();
              if let Ok(models) = ai_api.fetch_models(api_key, &alias.base_url).await {
                let now = time_service.utc_now();
                let _ = db.update_api_model_cache(&alias_id, models, now).await;
              }
            }
          });
        }
      }
    }
  }

  Ok(Json(ListModelResponse {
    object: "list".to_string(),
    data: models,
  }))
}

/// Get details for a specific model
#[utoipa::path(
    get,
    path = "/v1/models/{id}",
    tag = API_TAG_OPENAI,
    operation_id = "getModel",
    summary = "Get Model Details (OpenAI Compatible)",
    description = "Retrieves details for a specific model by ID in OpenAI API compatible format. The model ID can be a user alias, model alias, or API provider alias.",
    params(
        ("id" = String, Path,
         description = "Model identifier - can be user alias (e.g., 'llama2:chat'), model alias, or API provider alias",
         example = "llama2:chat")
    ),
    responses(
        (status = 200, description = "Model details",
         body = Model,
         example = json!({
             "id": "llama2:chat",
             "object": "model",
             "created": 1677610602,
             "owned_by": "system"
         })),
        (status = 404, description = "Model not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Model 'unknown:model' not found",
                 "type": "not_found_error",
                 "code": "model_not_found"
             }
         })),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn oai_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<Model>, ApiError> {
  // Use unified DataService.find_alias
  if let Some(alias) = state.app_service().data_service().find_alias(&id).await {
    match alias {
      Alias::User(user_alias) => Ok(Json(user_alias_to_oai_model(state, user_alias))),
      Alias::Model(model_alias) => Ok(Json(model_alias_to_oai_model(state, model_alias))),
      Alias::Api(api_alias) => {
        // DataService.find_alias() already verified model exists via matchable_models()
        Ok(Json(api_model_to_oai_model(id, &api_alias)))
      }
    }
  } else {
    Err(ApiError::from(DataServiceError::AliasNotFound(id)))
  }
}

fn user_alias_to_oai_model(_state: Arc<dyn RouterState>, alias: UserAlias) -> Model {
  Model {
    id: alias.alias,
    object: "model".to_string(),
    created: alias.created_at.timestamp() as u32,
    owned_by: "system".to_string(),
  }
}

fn model_alias_to_oai_model(state: Arc<dyn RouterState>, alias: ModelAlias) -> Model {
  // For auto-discovered models, construct path from HF cache structure
  // Path structure: hf_cache/models--owner--repo/snapshots/snapshot/filename
  let hf_cache = state.app_service().setting_service().hf_cache();
  let path = hf_cache
    .join(alias.repo.path())
    .join("snapshots")
    .join(&alias.snapshot)
    .join(&alias.filename);
  let created = state.app_service().time_service().created_at(&path);
  Model {
    id: alias.alias,
    object: "model".to_string(),
    created,
    owned_by: "system".to_string(),
  }
}

fn api_model_to_oai_model(model_id: String, api_alias: &ApiAlias) -> Model {
  let created = api_alias.created_at.timestamp() as u32;
  Model {
    id: model_id, // Use the prefixed model ID (prefix + model_name)
    object: "model".to_string(),
    created,
    owned_by: api_alias.base_url.clone(),
  }
}

#[cfg(test)]
#[path = "test_models.rs"]
mod test_models;
