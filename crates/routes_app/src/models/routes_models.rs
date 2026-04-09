use crate::shared::AuthScope;
use crate::{ApiError, BodhiApiError};
use crate::{PaginationSortParams, API_TAG_MODELS, ENDPOINT_MODELS};
use axum::{
  extract::{Path, Query},
  Json,
};
use services::Alias;
use services::{AliasResponse, DataServiceError, PaginatedAliasResponse, UserAliasResponse};

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
pub async fn models_index(
  auth_scope: AuthScope,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedAliasResponse>, ApiError> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);

  // Fetch all aliases using unified DataService (User + Model + API)
  let mut aliases = auth_scope.data().list_aliases().await?;

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
  // Model metadata is not tenant-scoped (shared across tenants), use empty string for tenant_id
  let metadata_map = if !file_keys.is_empty() {
    auth_scope
      .db_service()
      .batch_get_metadata_by_files("", &file_keys)
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
          let metadata: services::ModelMetadata = metadata_row.clone().into();
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

/// Get details for a specific model alias by UUID
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS.to_owned() + "/{id}",
    tag = API_TAG_MODELS,
    operation_id = "getAlias",
    summary = "Get Model Alias Details",
    description = "Retrieves detailed information for a specific model alias by UUID. Requires any authenticated user (User level permissions or higher).",
    params(
        ("id" = String, Path, description = "UUID of the alias")
    ),
    responses(
        (status = 200, description = "Model alias details", body = UserAliasResponse),
        (status = 404, description = "Alias not found", body = BodhiApiError,
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
pub async fn models_show(
  auth_scope: AuthScope,
  Path(id): Path<String>,
) -> Result<Json<UserAliasResponse>, ApiError> {
  let user_alias = auth_scope
    .data()
    .get_user_alias_by_id(&id)
    .await
    .ok_or(DataServiceError::AliasNotFound(id))?;

  // Query metadata for this alias
  let metadata = auth_scope
    .db_service()
    .get_model_metadata_by_file(
      "",
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
