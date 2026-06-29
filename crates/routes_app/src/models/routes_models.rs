use crate::shared::AuthScope;
use crate::BodhiErrorResponse;
use crate::{AliasFilterParams, PaginationSortParams, API_TAG_MODELS, ENDPOINT_MODELS};
use axum::{
  extract::{Path, Query},
  Json,
};
use services::Alias;
use services::{
  AliasResponse, ApiAliasResponse, ApiFormat, DataServiceError, LlmLibertySummary,
  PaginatedAliasResponse, Repo, UserAliasResponse,
};
use std::collections::HashMap;
use std::str::FromStr;

/// (repo, filename, snapshot) — the composite key for resolving a local file's size + metadata.
type FileKey = (String, String, String);

/// List all model aliases as discriminated union (user, model, and API aliases)
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    operation_id = "listAllModels",
    summary = "List All Model Aliases",
    description = "Retrieves paginated list of all configured model aliases including user-defined aliases, model aliases, and API provider aliases with server-side facet filtering (type, api_format, size range, capability) and sorting. Requires any authenticated user (User level permissions or higher).",
    params(
        PaginationSortParams,
        AliasFilterParams
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
  Query(filters): Query<AliasFilterParams>,
) -> Result<Json<PaginatedAliasResponse>, BodhiErrorResponse> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);

  let mut aliases = auth_scope.data().list_aliases().await?;

  // --- Server-side facet filtering (applied before sort + pagination so `total` and the page
  // reflect the filtered set). type + api_format are pure; size + capability need on-disk size /
  // metadata for the whole candidate set, resolved only when those facets are active. ---
  if !filters.is_empty() {
    // type + api_format + search (no I/O)
    let type_tokens = filters.type_tokens();
    let api_format_tokens = filters.api_format_tokens();
    let search = filters.search_query();
    aliases.retain(|alias| {
      (type_tokens.is_empty() || alias_matches_type(alias, &type_tokens))
        && (api_format_tokens.is_empty() || alias_matches_api_format(alias, &api_format_tokens))
        && search
          .as_deref()
          .is_none_or(|q| alias_matches_search(alias, q))
    });

    // size range (resolve on-disk size for the surviving local rows)
    if filters.size_min.is_some() || filters.size_max.is_some() {
      let sizes = resolve_sizes(&auth_scope, &aliases);
      let (min, max) = (filters.size_min.unwrap_or(0), filters.size_max);
      aliases.retain(|alias| match local_file_key(alias) {
        // local rows: keep only when size is known and within [min, max]
        Some(key) => sizes
          .get(&key)
          .copied()
          .flatten()
          .is_some_and(|sz| sz >= min && max.is_none_or(|mx| sz <= mx)),
        // API/router rows have no local file: never hidden by the size facet
        None => true,
      });
    }

    // capability (require metadata for the whole candidate set)
    let capability_tokens = filters.capability_tokens();
    if !capability_tokens.is_empty() {
      let keys = local_file_keys(&aliases);
      let metadata_map = if keys.is_empty() {
        HashMap::new()
      } else {
        auth_scope
          .db_service()
          .batch_get_metadata_by_files("", &keys)
          .await
          .unwrap_or_default()
      };
      aliases.retain(|alias| match local_file_key(alias) {
        Some(key) => metadata_map
          .get(&key)
          .map(|row| {
            let metadata: services::ModelMetadata = row.clone().into();
            capability_tokens
              .iter()
              .all(|cap| capability_is_set(&metadata, cap))
          })
          .unwrap_or(false),
        // API/router rows have no capability metadata: excluded when a capability facet is active
        None => false,
      });
    }
  }

  // Prune to grant-listable models (Unrestricted for session/external-app).
  let policy = auth_scope.access_policy();
  aliases.retain_mut(|alias| alias.retain_listable_models(|id| policy.model_listable(id)));

  sort_aliases(&mut aliases, &sort, &sort_order);

  let total = aliases.len();
  let (start, end) = calculate_pagination(page, page_size, total);
  let paginated_aliases: Vec<Alias> = aliases.into_iter().skip(start).take(end - start).collect();

  let file_keys: Vec<FileKey> = local_file_keys(&paginated_aliases);

  // Model metadata is not tenant-scoped (shared across tenants), use empty string for tenant_id
  let metadata_map = if !file_keys.is_empty() {
    auth_scope
      .db_service()
      .batch_get_metadata_by_files("", &file_keys)
      .await
      .unwrap_or_default()
  } else {
    HashMap::new()
  };

  // Resolve on-disk file size for the local rows on this page (for display).
  let size_map = resolve_sizes(&auth_scope, &paginated_aliases);

  // Pre-fetch llm_liberty summaries for any LlmLibertyOauth API aliases on this page so the
  // chat UI can route requests by `provider` without a follow-up alias-detail call.
  let llm_liberty_summaries: HashMap<String, LlmLibertySummary> = {
    let llm_liberty_ids: Vec<String> = paginated_aliases
      .iter()
      .filter_map(|alias| match alias {
        Alias::Api(a) if a.api_format == ApiFormat::LlmLibertyOauth => Some(a.id.clone()),
        _ => None,
      })
      .collect();
    if llm_liberty_ids.is_empty() {
      HashMap::new()
    } else {
      let tenant_id = auth_scope.require_tenant_id()?;
      let user_id = auth_scope.require_user_id()?;
      let db_service = auth_scope.db_service();
      let mut map = HashMap::new();
      for id in llm_liberty_ids {
        if let Ok(Some(summary)) = db_service
          .get_llm_liberty_summary(tenant_id, user_id, &id)
          .await
        {
          map.insert(id, summary);
        }
      }
      map
    }
  };

  let data: Vec<AliasResponse> = paginated_aliases
    .into_iter()
    .map(|alias| {
      let response = match &alias {
        Alias::Api(a) if a.api_format == ApiFormat::LlmLibertyOauth => {
          let summary = llm_liberty_summaries.get(&a.id).cloned();
          AliasResponse::Api(ApiAliasResponse::from(a.clone()).with_llm_liberty(summary))
        }
        _ => AliasResponse::from(alias.clone()),
      };
      match local_file_key(&alias) {
        Some(key) => {
          let size = size_map.get(&key).copied().flatten();
          let metadata = metadata_map
            .get(&key)
            .map(|row| services::ModelMetadata::from(row.clone()));
          response.with_size(size).with_metadata(metadata)
        }
        None => response,
      }
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
    Alias::ModelRouter(router) => &router.alias,
  }
}

fn get_alias_repo(alias: &Alias) -> String {
  match alias {
    Alias::User(user_alias) => user_alias.repo.to_string(),
    Alias::Model(model_alias) => model_alias.repo.to_string(),
    // API and model-router aliases don't have repos
    Alias::Api(_) | Alias::ModelRouter(_) => "".to_string(),
  }
}

fn get_alias_filename(alias: &Alias) -> &str {
  match alias {
    Alias::User(user_alias) => &user_alias.filename,
    Alias::Model(model_alias) => &model_alias.filename,
    // API and model-router aliases don't have filenames
    Alias::Api(_) | Alias::ModelRouter(_) => "",
  }
}

fn get_alias_source(alias: &Alias) -> &str {
  match alias {
    Alias::User(_) => "user",
    Alias::Model(_) => "model",
    Alias::Api(_) => "api",
    Alias::ModelRouter(_) => "model_router",
  }
}

/// The (repo, filename, snapshot) key for a local alias (User/Model); None for API/router.
fn local_file_key(alias: &Alias) -> Option<FileKey> {
  match alias {
    Alias::User(u) => Some((u.repo.to_string(), u.filename.clone(), u.snapshot.clone())),
    Alias::Model(m) => Some((m.repo.to_string(), m.filename.clone(), m.snapshot.clone())),
    Alias::Api(_) | Alias::ModelRouter(_) => None,
  }
}

fn local_file_keys(aliases: &[Alias]) -> Vec<FileKey> {
  aliases.iter().filter_map(local_file_key).collect()
}

/// Resolve on-disk byte size for every local alias in `aliases`, keyed by file key. Statting is
/// page/candidate-scoped by the caller; a file that can't be resolved maps to `None`.
fn resolve_sizes(auth_scope: &AuthScope, aliases: &[Alias]) -> HashMap<FileKey, Option<u64>> {
  let hub_service = auth_scope.hub_service();
  aliases
    .iter()
    .filter_map(local_file_key)
    .map(|key| {
      let (repo, filename, snapshot) = &key;
      let size = Repo::from_str(repo).ok().and_then(|repo| {
        hub_service
          .find_local_file(&repo, filename, Some(snapshot.clone()))
          .ok()
          .and_then(|hub_file| hub_file.size)
      });
      (key, size)
    })
    .collect()
}

/// Map an alias to its TYPE facet token (`local_file` / `model_alias` / `api_model` / `fallback`)
/// and test membership in the requested set.
fn alias_matches_type(alias: &Alias, tokens: &[String]) -> bool {
  let token = match alias {
    Alias::Model(_) => "local_file",
    Alias::User(_) => "model_alias",
    Alias::Api(_) => "api_model",
    Alias::ModelRouter(_) => "fallback",
  };
  tokens.iter().any(|t| t == token)
}

/// Test whether an API alias' `api_format` falls in the requested API-FORMAT facet set. The facet
/// tokens are UI buckets: `anthropic` covers both anthropic and anthropic_oauth; `responses` =
/// openai_responses; `liberty` = llm_liberty_oauth. Non-API aliases never match.
fn alias_matches_api_format(alias: &Alias, tokens: &[String]) -> bool {
  let Alias::Api(api) = alias else {
    return false;
  };
  let bucket = match api.api_format {
    ApiFormat::OpenAI => "openai",
    ApiFormat::OpenAIResponses => "responses",
    ApiFormat::Anthropic | ApiFormat::AnthropicOAuth => "anthropic",
    ApiFormat::Gemini => "gemini",
    ApiFormat::LlmLibertyOauth => "liberty",
  };
  tokens.iter().any(|t| t == bucket)
}

/// Case-insensitive substring search over a row's identifying fields (`query` is already
/// lowercased). Local rows match on alias / repo / filename; API rows on id / name / base_url;
/// routers on alias. Mirrors the frontend's match set so server + client agree.
fn alias_matches_search(alias: &Alias, query: &str) -> bool {
  let fields: Vec<String> = match alias {
    Alias::User(u) => vec![u.alias.clone(), u.repo.to_string(), u.filename.clone()],
    Alias::Model(m) => vec![m.alias.clone(), m.repo.to_string(), m.filename.clone()],
    Alias::Api(a) => vec![a.id.clone(), a.name.clone(), a.base_url.clone()],
    Alias::ModelRouter(r) => vec![r.alias.clone()],
  };
  fields.iter().any(|f| f.to_lowercase().contains(query))
}

/// Map a CAPABILITY facet token to the corresponding `ModelCapabilities` field. Unknown tokens
/// never match. `tool_use` → tools.function_calling; `reasoning` → thinking; `vision` → vision.
fn capability_is_set(metadata: &services::ModelMetadata, token: &str) -> bool {
  let caps = &metadata.capabilities;
  match token {
    "vision" => caps.vision == Some(true),
    "tool_use" | "tool-use" => caps.tools.function_calling == Some(true),
    "reasoning" => caps.thinking == Some(true),
    _ => false,
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
        (status = 404, description = "Alias not found", body = BodhiErrorResponse,
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
) -> Result<Json<UserAliasResponse>, BodhiErrorResponse> {
  let user_alias = auth_scope
    .data()
    .get_user_alias_by_id(&id)
    .await
    .ok_or(DataServiceError::AliasNotFound(id))?;

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
