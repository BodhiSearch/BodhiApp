use crate::{
  api_models_dto::ApiModelResponse, AliasResponse, LocalModelResponse, PaginatedLocalModelResponse,
  PaginatedResponse, PaginatedUnifiedModelResponse, PaginationSortParams, UnifiedModelResponse,
  ENDPOINT_MODELS, ENDPOINT_MODEL_FILES,
};
use axum::{
  extract::{Query, State},
  Json,
};
use objs::{ApiError, HubFile, OpenAIApiError, API_TAG_MODELS};
use server_core::RouterState;
use services::AliasNotFoundError;
use std::sync::Arc;

/// List all model aliases (both local aliases and API models)
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    operation_id = "listAllModels",
    params(
        PaginationSortParams
    ),
    responses(
        (status = 200, description = "List of all configured models (local aliases and API models)", body = PaginatedUnifiedModelResponse,
         example = json!({
             "data": [{
                 "model_type": "local",
                 "alias": "llama2:chat",
                 "repo": "TheBloke/Llama-2-7B-Chat-GGUF",
                 "filename": "llama-2-7b-chat.Q4_K_M.gguf",
                 "snapshot": "abc123",
                 "source": "user",
                 "model_params": {},
                 "request_params": {
                     "temperature": 0.7,
                     "top_p": 0.95
                 },
                 "context_params": ["--ctx_size", "4096"]
             }, {
                 "model_type": "api",
                 "alias": "openai-gpt4",
                 "provider": "openai",
                 "base_url": "https://api.openai.com/v1",
                 "api_key_masked": "sk-...abc123",
                 "models": ["gpt-4", "gpt-3.5-turbo"],
                 "created_at": "2024-01-01T00:00:00Z",
                 "updated_at": "2024-01-01T00:00:00Z"
             }],
             "total": 2,
             "page": 1,
             "page_size": 10
         })
        ),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
    ),
)]
pub async fn list_local_aliases_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedUnifiedModelResponse>, ApiError> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);

  // Fetch local aliases
  let aliases = state.app_service().data_service().list_aliases()?;
  let local_models: Vec<UnifiedModelResponse> = aliases
    .into_iter()
    .map(|alias| UnifiedModelResponse::from(AliasResponse::from(alias)))
    .collect();

  // Fetch API model aliases
  let db_service = state.app_service().db_service();
  let api_aliases = db_service.list_api_model_aliases().await?;
  let api_models: Vec<UnifiedModelResponse> = api_aliases
    .into_iter()
    .map(|alias| {
      // For list view, we don't show the actual API key, just masked version
      let api_response = ApiModelResponse::from_alias(alias, None);
      UnifiedModelResponse::from(api_response)
    })
    .collect();

  // Combine both types
  let mut all_models = Vec::new();
  all_models.extend(local_models);
  all_models.extend(api_models);

  // Sort combined list
  sort_unified_models(&mut all_models, &sort, &sort_order);

  let total = all_models.len();
  let (start, end) = calculate_pagination(page, page_size, total);
  let data: Vec<UnifiedModelResponse> = all_models
    .into_iter()
    .skip(start)
    .take(end - start)
    .collect();

  let paginated = PaginatedResponse {
    data,
    total,
    page,
    page_size,
  };
  Ok(Json(paginated.into()))
}

/// List available model files in GGUF format from HuggingFace cache
#[utoipa::path(
    get,
    path = ENDPOINT_MODEL_FILES,
    tag = API_TAG_MODELS,
    operation_id = "listModelFiles",
    params(
        PaginationSortParams
    ),
    responses(
        (status = 200, description = "List of supported model files from local HuggingFace cache folder", body = PaginatedLocalModelResponse,
         example = json!({
             "data": [{
                 "repo": "TheBloke/Mistral-7B-Instruct-v0.1-GGUF",
                 "filename": "mistral-7b-instruct-v0.1.Q4_K_M.gguf",
                 "snapshot_id": "ab12cd34",
                 "size": 4815162
             }],
             "total": 1,
             "page": 1,
             "page_size": 10
         })
        ),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
      ("bearer_auth" = []),
    ),
)]
pub async fn list_local_modelfiles_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedLocalModelResponse>, ApiError> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);
  let mut models = state.app_service().hub_service().list_local_models();
  sort_models(&mut models, &sort, &sort_order);
  let total = models.len();
  let (start, end) = calculate_pagination(page, page_size, total);

  let data: Vec<LocalModelResponse> = models
    .into_iter()
    .skip(start)
    .take(end - start)
    .map(Into::into)
    .collect();

  let paginated = PaginatedResponse {
    data,
    total,
    page,
    page_size,
  };
  Ok(Json(paginated.into()))
}

fn extract_pagination_sort_params(params: PaginationSortParams) -> (usize, usize, String, String) {
  let page = params.page;
  let page_size = params.page_size.min(100);
  let sort = params.sort.unwrap_or_else(|| "name".to_string());
  let sort_order = params.sort_order;
  (page, page_size, sort, sort_order)
}

fn calculate_pagination(page: usize, page_size: usize, total: usize) -> (usize, usize) {
  let start = (page - 1) * page_size;
  let end = (start + page_size).min(total);
  (start, end)
}

fn sort_models(models: &mut [HubFile], sort: &str, sort_order: &str) {
  models.sort_by(|a, b| {
    let cmp = match sort {
      "repo" => a.repo.cmp(&b.repo),
      "filename" => a.filename.cmp(&b.filename),
      "snapshot" => a.snapshot.cmp(&b.snapshot),
      "size" => a.size.cmp(&b.size),
      _ => a.repo.cmp(&b.repo),
    };
    if sort_order.to_lowercase() == "desc" {
      cmp.reverse()
    } else {
      cmp
    }
  });
}

fn sort_unified_models(models: &mut [UnifiedModelResponse], sort: &str, sort_order: &str) {
  models.sort_by(|a, b| {
    let cmp = match sort {
      "alias" | "name" => get_model_alias(a).cmp(&get_model_alias(b)),
      "repo" => get_model_repo_or_provider(a).cmp(&get_model_repo_or_provider(b)),
      "filename" => get_model_filename_or_base_url(a).cmp(&get_model_filename_or_base_url(b)),
      "source" => get_model_source_or_type(a).cmp(&get_model_source_or_type(b)),
      _ => get_model_alias(a).cmp(&get_model_alias(b)),
    };
    if sort_order.to_lowercase() == "desc" {
      cmp.reverse()
    } else {
      cmp
    }
  });
}

fn get_model_alias(model: &UnifiedModelResponse) -> &str {
  match model {
    UnifiedModelResponse::Local { alias, .. } => alias,
    UnifiedModelResponse::Api { id, .. } => id,
  }
}

fn get_model_repo_or_provider(model: &UnifiedModelResponse) -> &str {
  match model {
    UnifiedModelResponse::Local { repo, .. } => repo,
    UnifiedModelResponse::Api { provider, .. } => provider,
  }
}

fn get_model_filename_or_base_url(model: &UnifiedModelResponse) -> &str {
  match model {
    UnifiedModelResponse::Local { filename, .. } => filename,
    UnifiedModelResponse::Api { base_url, .. } => base_url,
  }
}

fn get_model_source_or_type(model: &UnifiedModelResponse) -> &str {
  match model {
    UnifiedModelResponse::Local { source, .. } => source,
    UnifiedModelResponse::Api { .. } => "api",
  }
}

/// Get details for a specific model alias
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS.to_owned() + "/{alias}",
    tag = API_TAG_MODELS,
    operation_id = "getAlias",
    params(
        ("alias" = String, Path, description = "Alias identifier for the model")
    ),
    responses(
        (status = 200, description = "Model alias details", body = AliasResponse,
         example = json!({
             "alias": "llama2:chat",
             "repo": "TheBloke/Llama-2-7B-Chat-GGUF",
             "filename": "llama-2-7b-chat.Q8_0.gguf",
             "snapshot": "sha256:abc123",
             "source": "config",
             "chat_template": "llama2",
             "model_params": {},
             "request_params": {
                 "temperature": 0.7,
                 "top_p": 1.0,
                 "frequency_penalty": 0.0,
                 "presence_penalty": 0.0
             },
             "context_params": {
                 "n_keep": 24,
                 "stop": [
                     "<|end_of_turn|>"
                 ]
             }
         })),
        (status = 404, description = "Alias not found", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Alias 'unknown:model' not found",
                 "type": "not_found_error",
                 "code": "alias_not_found"
             }
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
        ("bearer_auth" = []),
    )
)]
pub async fn get_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<AliasResponse>, ApiError> {
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&id)
    .ok_or(AliasNotFoundError(id))?;
  Ok(Json(AliasResponse::from(alias)))
}

#[cfg(test)]
mod tests {
  use crate::{get_alias_handler, list_local_aliases_handler, AliasResponse, PaginatedResponse};
  use axum::{
    body::Body,
    http::{status::StatusCode, Request},
    routing::get,
    Router,
  };
  use objs::{test_utils::setup_l10n, FluentLocalizationService};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{router_state_stub, ResponseTestExt},
    DefaultRouterState,
  };
  use std::sync::Arc;

  use tower::ServiceExt;

  fn test_router(router_state_stub: DefaultRouterState) -> Router {
    Router::new()
      .route("/api/models", get(list_local_aliases_handler))
      .route("/api/models/{id}", get(get_alias_handler))
      .with_state(Arc::new(router_state_stub))
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_local_aliases_handler(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
      .oneshot(Request::get("/api/models").body(Body::empty()).unwrap())
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(1, response["page"]);
    assert_eq!(30, response["page_size"]);
    assert_eq!(8, response["total"]);
    let data = response["data"].as_array().unwrap();
    assert!(!data.is_empty());
    assert_eq!(
      "FakeFactory/fakemodel-gguf:Q4_0",
      data.first().unwrap()["alias"].as_str().unwrap(),
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_local_aliases_page_size(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
      .oneshot(
        Request::get("/api/models?page=2&page_size=4")
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(2, response["page"]);
    assert_eq!(4, response["page_size"]);
    assert_eq!(8, response["total"]);
    let data = response["data"].as_array().unwrap();
    assert_eq!(4, data.len());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_local_aliases_over_limit_page_size(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
      .oneshot(
        Request::get("/api/models?page_size=150")
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .json::<Value>()
      .await?;

    assert_eq!(100, response["page_size"]);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_local_aliases_response_structure(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
      .oneshot(Request::get("/api/models").body(Body::empty()).unwrap())
      .await?
      .json::<PaginatedResponse<AliasResponse>>()
      .await?;

    assert!(!response.data.is_empty());
    let first_alias = response
      .data
      .iter()
      .find(|a| a.alias == "llama3:instruct")
      .unwrap();
    let expected = AliasResponse::llama3();
    assert_eq!(expected, *first_alias);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_local_aliases_sorting(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
      .oneshot(
        Request::get("/api/models?sort=family&sort_order=desc")
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .json::<PaginatedResponse<AliasResponse>>()
      .await?;

    assert!(!response.data.is_empty());
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_alias_handler(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
      .oneshot(
        Request::get("/api/models/llama3:instruct")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;
    assert_eq!(StatusCode::OK, response.status());
    let alias_response = response.json::<AliasResponse>().await?;
    let expected = AliasResponse::llama3();
    assert_eq!(expected, alias_response);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_get_alias_handler_non_existent(
    #[future] router_state_stub: DefaultRouterState,
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
      .oneshot(
        Request::get("/api/models/non_existent_alias")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;
    assert_eq!(StatusCode::NOT_FOUND, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "message": "alias '\u{2068}non_existent_alias\u{2069}' not found in $BODHI_HOME/aliases",
          "code": "alias_not_found_error",
          "type": "not_found_error"
        }
      }},
      response
    );
    Ok(())
  }
}
