use crate::models::error::ModelRouteError;
use crate::models::models_api_schemas::{LocalModelResponse, PaginatedLocalModelResponse};
use crate::shared::AuthScope;
use crate::BodhiErrorResponse;
use crate::{PaginationSortParams, API_TAG_MODELS_FILES, ENDPOINT_MODELS_FILES};
use axum::{extract::Query, Json};
use services::HubFile;

/// List available model files in GGUF format from HuggingFace cache
#[utoipa::path(
    get,
    path = ENDPOINT_MODELS_FILES,
    tag = API_TAG_MODELS_FILES,
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
pub async fn modelfiles_index(
  auth_scope: AuthScope,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedLocalModelResponse>, BodhiErrorResponse> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);
  let mut models = auth_scope.hub_service().list_local_models();
  sort_models(&mut models, &sort, &sort_order);
  let total = models.len();
  let (start, end) = calculate_pagination(page, page_size, total);

  let page_models: Vec<HubFile> = models.into_iter().skip(start).take(end - start).collect();

  // Batch fetch metadata for all models in this page
  let keys: Vec<(String, String, String)> = page_models
    .iter()
    .map(|m| (m.repo.to_string(), m.filename.clone(), m.snapshot.clone()))
    .collect();

  let metadata_map = auth_scope
    .db_service()
    .batch_get_metadata_by_files("", &keys)
    .await
    .map_err(|e| {
      tracing::error!("Failed to batch fetch metadata: {}", e);
      ModelRouteError::MetadataFetchFailed
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
