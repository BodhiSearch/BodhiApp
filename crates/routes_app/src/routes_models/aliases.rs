use crate::{
  AliasResponse, CreateAliasError, CreateAliasRequest, LocalModelResponse, ModelError,
  PaginatedAliasResponse, PaginatedLocalModelResponse, PaginationSortParams, UpdateAliasRequest,
  UserAliasResponse, ENDPOINT_MODELS, ENDPOINT_MODEL_FILES,
};
use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Json,
};
use axum_extra::extract::WithRejection;
use objs::{
  Alias, ApiError, HubFile, OAIRequestParams, OpenAIApiError, Repo, UserAliasBuilder,
  API_TAG_MODELS,
};
use server_core::RouterState;
use services::{AppService, DataServiceError, HubServiceError, SNAPSHOT_MAIN};
use std::sync::Arc;
use tracing::debug;

/// List all model aliases as discriminated union (user, model, and API aliases)
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    operation_id = "listAllModels",
    summary = "List All Model Aliases",
    description = "Retrieves paginated list of all configured model aliases including user-defined aliases, model aliases, and API provider aliases with filtering and sorting options. Requires any authenticated user (User level permissions or higher).",
    params(
        PaginationSortParams
    ),
    responses(
        (status = 200, description = "Paginated list of model aliases retrieved successfully", body = PaginatedAliasResponse,
         example = json!({
             "data": [
              {
                "source": "user",
                "alias": "llama2:chat",
                "repo": "TheBloke/Llama-2-7B-Chat-GGUF",
                "filename": "llama-2-7b-chat.Q4_K_M.gguf",
                "snapshot": "abc123",
                "request_params": {
                    "temperature": 0.7,
                    "top_p": 0.95
                },
                "context_params": ["--ctx_size", "4096"]
              },
              {
                "source": "model",
                "alias": "TheBloke/Llama-2-7B-Chat-GGUF:Q4_K_M",
                "repo": "TheBloke/Llama-2-7B-Chat-GGUF",
                "filename": "llama-2-7b-chat.Q4_K_M.gguf",
                "snapshot": "abc123"
              },
              {
                "source": "api",
                "id": "openai-gpt4",
                "api_format": "openai",
                "base_url": "https://api.openai.com/v1",
                "models": ["gpt-4", "gpt-3.5-turbo"],
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
             }],
             "total": 3,
             "page": 1,
             "page_size": 10
         })
        ),
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    ),
)]
pub async fn list_aliases_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedAliasResponse>, ApiError> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);

  // Fetch all aliases using unified DataService (User + Model + API)
  let mut aliases = state.app_service().data_service().list_aliases().await?;

  // Sort aliases directly
  sort_aliases(&mut aliases, &sort, &sort_order);

  let total = aliases.len();
  let (start, end) = calculate_pagination(page, page_size, total);
  let paginated_aliases: Vec<Alias> = aliases.into_iter().skip(start).take(end - start).collect();

  // Extract file paths from User and Model variants
  let file_keys: Vec<(String, String, String)> = paginated_aliases
    .iter()
    .filter_map(|alias| match alias {
      Alias::User(u) => Some((u.repo.to_string(), u.filename.clone(), u.snapshot.clone())),
      Alias::Model(m) => Some((m.repo.to_string(), m.filename.clone(), m.snapshot.clone())),
      Alias::Api(_) => None,
    })
    .collect();

  // Batch query metadata for all file paths
  let metadata_map = if !file_keys.is_empty() {
    state
      .app_service()
      .db_service()
      .batch_get_metadata_by_files(&file_keys)
      .await
      .unwrap_or_default()
  } else {
    std::collections::HashMap::new()
  };

  // Convert to AliasResponse and attach metadata
  let data: Vec<AliasResponse> = paginated_aliases
    .into_iter()
    .map(|alias| {
      let response = AliasResponse::from(alias.clone());
      // Try to find metadata for this alias
      let key = match alias {
        Alias::User(u) => Some((u.repo.to_string(), u.filename, u.snapshot)),
        Alias::Model(m) => Some((m.repo.to_string(), m.filename, m.snapshot)),
        Alias::Api(_) => None,
      };
      if let Some(k) = key {
        if let Some(metadata_row) = metadata_map.get(&k) {
          let metadata: objs::ModelMetadata = metadata_row.clone().into();
          return response.with_metadata(Some(metadata));
        }
      }
      response
    })
    .collect();

  let paginated = PaginatedAliasResponse {
    data,
    total,
    page,
    page_size,
  };
  Ok(Json(paginated))
}

/// List available model files in GGUF format from HuggingFace cache
#[utoipa::path(
    get,
    path = ENDPOINT_MODEL_FILES,
    tag = API_TAG_MODELS,
    operation_id = "listModelFiles",
    summary = "List Local Model Files",
    description = "Retrieves paginated list of GGUF model files available in the local HuggingFace cache directory with metadata including repository, filename, snapshot ID, and file size. Requires any authenticated user (User level permissions or higher).",
    params(
        PaginationSortParams
    ),
    responses(
        (status = 200, description = "Local model files retrieved successfully from cache", body = PaginatedLocalModelResponse,
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
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
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

  let page_models: Vec<HubFile> = models.into_iter().skip(start).take(end - start).collect();

  // Batch fetch metadata for all models in this page
  let keys: Vec<(String, String, String)> = page_models
    .iter()
    .map(|m| (m.repo.to_string(), m.filename.clone(), m.snapshot.clone()))
    .collect();

  let metadata_map = state
    .app_service()
    .db_service()
    .batch_get_metadata_by_files(&keys)
    .await
    .map_err(|e| {
      tracing::error!("Failed to batch fetch metadata: {}", e);
      ModelError::MetadataFetchFailed
    })?;

  // Convert to responses with metadata attached
  let data: Vec<LocalModelResponse> = page_models
    .into_iter()
    .map(|model| {
      let key = (
        model.repo.to_string(),
        model.filename.clone(),
        model.snapshot.clone(),
      );
      let metadata = metadata_map.get(&key).map(|row| row.clone().into());
      LocalModelResponse::from(model).with_metadata(metadata)
    })
    .collect();

  let paginated = PaginatedLocalModelResponse {
    data,
    total,
    page,
    page_size,
  };
  Ok(Json(paginated))
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

fn sort_aliases(aliases: &mut [Alias], sort: &str, sort_order: &str) {
  aliases.sort_by(|a, b| {
    let cmp = match sort {
      "alias" | "name" => get_alias_name(a).cmp(get_alias_name(b)),
      "repo" => get_alias_repo(a).cmp(&get_alias_repo(b)),
      "filename" => get_alias_filename(a).cmp(get_alias_filename(b)),
      "source" => get_alias_source(a).cmp(get_alias_source(b)),
      _ => get_alias_name(a).cmp(get_alias_name(b)),
    };
    if sort_order.to_lowercase() == "desc" {
      cmp.reverse()
    } else {
      cmp
    }
  });
}

fn get_alias_name(alias: &Alias) -> &str {
  match alias {
    Alias::User(user_alias) => &user_alias.alias,
    Alias::Model(model_alias) => &model_alias.alias,
    Alias::Api(api_alias) => &api_alias.id,
  }
}

fn get_alias_repo(alias: &Alias) -> String {
  match alias {
    Alias::User(user_alias) => user_alias.repo.to_string(),
    Alias::Model(model_alias) => model_alias.repo.to_string(),
    Alias::Api(_) => "".to_string(), // API aliases don't have repos
  }
}

fn get_alias_filename(alias: &Alias) -> &str {
  match alias {
    Alias::User(user_alias) => &user_alias.filename,
    Alias::Model(model_alias) => &model_alias.filename,
    Alias::Api(_) => "", // API aliases don't have filenames
  }
}

fn get_alias_source(alias: &Alias) -> &str {
  match alias {
    Alias::User(_) => "user",
    Alias::Model(_) => "model",
    Alias::Api(_) => "api",
  }
}

/// Get details for a specific model alias
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS.to_owned() + "/{alias}",
    tag = API_TAG_MODELS,
    operation_id = "getAlias",
    summary = "Get Model Alias Details",
    description = "Retrieves detailed information for a specific model alias. Requires any authenticated user (User level permissions or higher).",
    params(
        ("alias" = String, Path, description = "Alias identifier for the model")
    ),
    responses(
        (status = 200, description = "Model alias details", body = UserAliasResponse,
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
    ),
    security(
        ("bearer_api_token" = ["scope_token_user"]),
        ("bearer_oauth_token" = ["scope_user_user"]),
        ("session_auth" = ["resource_user"])
    )
)]
pub async fn get_user_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(alias): Path<String>,
) -> Result<Json<UserAliasResponse>, ApiError> {
  let user_alias = state
    .app_service()
    .data_service()
    .find_user_alias(&alias)
    .ok_or(DataServiceError::AliasNotFound(alias))?;

  // Query metadata for this alias
  let metadata = state
    .app_service()
    .db_service()
    .get_model_metadata_by_file(
      &user_alias.repo.to_string(),
      &user_alias.filename,
      &user_alias.snapshot,
    )
    .await
    .ok()
    .flatten()
    .map(|row| row.into());

  Ok(Json(
    UserAliasResponse::from(user_alias).with_metadata(metadata),
  ))
}

/// Create Alias
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    operation_id = "createAlias",
    request_body = CreateAliasRequest,
    responses(
      (status = 201, description = "Alias created succesfully", body = UserAliasResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn create_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateAliasRequest>, ApiError>,
) -> Result<(StatusCode, Json<UserAliasResponse>), ApiError> {
  execute_create_alias(
    state.app_service().as_ref(),
    payload.alias.clone(),
    Repo::try_from(payload.repo)?,
    payload.filename,
    payload.snapshot,
    payload.request_params.unwrap_or_default(),
    payload.context_params.unwrap_or_default(),
    false,
  )
  .await?;
  let alias = state
    .app_service()
    .data_service()
    .find_user_alias(&payload.alias)
    .ok_or(DataServiceError::AliasNotFound(payload.alias))?;
  Ok((StatusCode::CREATED, Json(UserAliasResponse::from(alias))))
}

/// Update Alias
#[utoipa::path(
    put,
    path = ENDPOINT_MODELS.to_owned() + "/{id}",
    tag = API_TAG_MODELS,
    params(
        ("id" = String, Path, description = "Alias identifier",
         example = "llama--3")
    ),
    operation_id = "updateAlias",
    request_body = UpdateAliasRequest,
    responses(
      (status = 200, description = "Alias updated succesfully", body = UserAliasResponse),
    ),
    security(
        ("bearer_api_token" = ["scope_token_power_user"]),
        ("bearer_oauth_token" = ["scope_user_power_user"]),
        ("session_auth" = ["resource_power_user"])
    ),
)]
pub async fn update_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  axum::extract::Path(id): axum::extract::Path<String>,
  WithRejection(Json(payload), _): WithRejection<Json<UpdateAliasRequest>, ApiError>,
) -> Result<(StatusCode, Json<UserAliasResponse>), ApiError> {
  execute_create_alias(
    state.app_service().as_ref(),
    id.clone(),
    Repo::try_from(payload.repo)?,
    payload.filename,
    payload.snapshot,
    payload.request_params.unwrap_or_default(),
    payload.context_params.unwrap_or_default(),
    true,
  )
  .await?;
  let alias = state
    .app_service()
    .data_service()
    .find_user_alias(&id)
    .ok_or(DataServiceError::AliasNotFound(id))?;
  Ok((StatusCode::OK, Json(UserAliasResponse::from(alias))))
}

#[allow(clippy::too_many_arguments)]
async fn execute_create_alias(
  service: &dyn AppService,
  alias: String,
  repo: Repo,
  filename: String,
  snapshot: Option<String>,
  oai_request_params: OAIRequestParams,
  context_params: Vec<String>,
  update: bool,
) -> Result<(), CreateAliasError> {
  if service.data_service().find_user_alias(&alias).is_some() {
    if !update {
      return Err(DataServiceError::AliasExists(alias.clone()).into());
    }
    debug!("Updating existing alias: '{}'", alias);
  } else {
    debug!("Creating new alias: '{}'", alias);
  }
  let file_exists = service
    .hub_service()
    .local_file_exists(&repo, &filename, snapshot.clone())?;
  let local_model_file = match file_exists {
    true => {
      debug!(
        "repo: '{}', filename: '{}', already exists in $HF_HOME",
        &repo, &filename
      );
      service
        .hub_service()
        .find_local_file(&repo, &filename, snapshot.clone())?
    }
    false => {
      return Err(CreateAliasError::HubService(
        HubServiceError::FileNotFound {
          filename: filename.clone(),
          repo: repo.to_string(),
          snapshot: snapshot
            .clone()
            .unwrap_or_else(|| SNAPSHOT_MAIN.to_string()),
        },
      ));
    }
  };
  let user_alias = UserAliasBuilder::default()
    .alias(alias)
    .repo(repo)
    .filename(filename)
    .snapshot(local_model_file.snapshot)
    .request_params(oai_request_params)
    .context_params(context_params)
    .build()?;
  service.data_service().save_alias(&user_alias)?;
  debug!(
    "model alias: '{}' saved to $BODHI_HOME/aliases",
    user_alias.alias
  );
  Ok(())
}
