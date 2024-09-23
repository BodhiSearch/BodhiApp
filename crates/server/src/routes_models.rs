use crate::{HttpError, HttpErrorBuilder, RouterState};
use axum::{
  extract::{Query, State},
  routing::get,
  Json, Router,
};
use hyper::StatusCode;
use objs::{Alias, ChatTemplate, ChatTemplateId, GptContextParams, HubFile, OAIRequestParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use strum::IntoEnumIterator;

#[derive(Serialize, Deserialize)]
pub struct PaginationSortParams {
  page: Option<usize>,
  page_size: Option<usize>,
  sort: Option<String>,
  sort_order: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
  data: Vec<T>,
  total: usize,
  page: usize,
  page_size: usize,
}

#[allow(clippy::too_many_arguments)]
#[derive(Serialize, Deserialize, Debug, PartialEq, derive_new::new)]
#[cfg_attr(any(test, feature = "test-utils"), derive(derive_builder::Builder))]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(
    setter(into),
    build_fn(error = objs::BuilderError)))]
pub struct AliasResponse {
  alias: String,
  repo: String,
  filename: String,
  snapshot: String,
  family: Option<String>,
  features: Vec<String>,
  chat_template: String,
  model_params: HashMap<String, Value>,
  request_params: OAIRequestParams,
  context_params: GptContextParams,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LocalModelResponse {
  repo: String,
  filename: String,
  snapshot: String,
  size: Option<u64>,
  model_params: HashMap<String, Value>,
}

impl From<HubFile> for LocalModelResponse {
  fn from(model: HubFile) -> Self {
    LocalModelResponse {
      repo: model.repo.to_string(),
      filename: model.filename,
      snapshot: model.snapshot,
      size: model.size,
      model_params: HashMap::new(),
    }
  }
}

impl From<Alias> for AliasResponse {
  fn from(alias: Alias) -> Self {
    AliasResponse {
      alias: alias.alias,
      family: alias.family,
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      features: alias.features,
      chat_template: alias.chat_template.to_string(),
      model_params: HashMap::new(),
      request_params: alias.request_params,
      context_params: alias.context_params,
    }
  }
}

pub fn models_router() -> Router<Arc<dyn RouterState>> {
  Router::new()
    .route("/models", get(list_local_aliases_handler))
    .route("/models/:id", get(get_alias_handler))
    .route("/modelfiles", get(list_local_modelfiles_handler))
    .route("/chat_templates", get(list_chat_templates_handler))
}

pub async fn list_local_aliases_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedResponse<AliasResponse>>, HttpError> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);

  let mut aliases = state
    .app_service()
    .data_service()
    .list_aliases()
    .map_err(|err| {
      HttpErrorBuilder::default()
        .internal_server(Some(&err.to_string()))
        .build()
        .unwrap()
    })?;

  sort_aliases(&mut aliases, &sort, &sort_order);

  let total = aliases.len();
  let (start, end) = calculate_pagination(page, page_size, total);

  let data: Vec<AliasResponse> = aliases
    .into_iter()
    .skip(start)
    .take(end - start)
    .map(AliasResponse::from)
    .collect();

  Ok(Json(PaginatedResponse {
    data,
    total,
    page,
    page_size,
  }))
}

pub async fn list_local_modelfiles_handler(
  State(state): State<Arc<dyn RouterState>>,
  Query(params): Query<PaginationSortParams>,
) -> Result<Json<PaginatedResponse<LocalModelResponse>>, HttpError> {
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

  Ok(Json(PaginatedResponse {
    data,
    total,
    page,
    page_size,
  }))
}

fn extract_pagination_sort_params(params: PaginationSortParams) -> (usize, usize, String, String) {
  let page = params.page.unwrap_or(1).max(1);
  let page_size = params.page_size.unwrap_or(30).min(100);
  let sort = params.sort.unwrap_or_else(|| "name".to_string());
  let sort_order = params.sort_order.unwrap_or_else(|| "asc".to_string());
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
      "name" => a.alias.cmp(&b.alias),
      "family" => a.family.cmp(&b.family),
      "repo" => a.repo.cmp(&b.repo),
      "filename" => a.filename.cmp(&b.filename),
      _ => a.alias.cmp(&b.alias),
    };
    if sort_order.to_lowercase() == "desc" {
      cmp.reverse()
    } else {
      cmp
    }
  });
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

pub async fn list_chat_templates_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<Vec<ChatTemplate>>, HttpError> {
  let mut responses = Vec::new();
  for chat_template in ChatTemplateId::iter() {
    responses.push(ChatTemplate::Id(chat_template));
  }
  let local_repos = state
    .app_service()
    .hub_service()
    .list_local_tokenizer_configs();
  for repo in local_repos {
    responses.push(ChatTemplate::Repo(repo));
  }
  Ok(Json(responses))
}

pub async fn get_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<AliasResponse>, HttpError> {
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&id)
    .ok_or_else(|| {
      HttpErrorBuilder::default()
        .status_code(StatusCode::NOT_FOUND)
        .r#type("alias_not_found")
        .code("not_found")
        .message(&format!("Alias '{}' not found", id))
        .build()
        .unwrap()
    })?;

  Ok(Json(AliasResponse::from(alias)))
}

#[cfg(test)]
mod tests {
  use crate::{
    get_alias_handler, list_chat_templates_handler, list_local_aliases_handler,
    test_utils::ResponseTestExt, AliasResponse, MockRouterState, PaginatedResponse,
  };
  use axum::{
    body::Body,
    http::{status::StatusCode, Request},
    routing::get,
    Router,
  };
  use objs::{
    ChatTemplate, ChatTemplateId, GptContextParamsBuilder, OAIRequestParamsBuilder, Repo,
  };
  use rstest::{fixture, rstest};
  use serde_json::Value;
  use services::test_utils::AppServiceStubBuilder;
  use std::collections::HashMap;
  use std::sync::Arc;
  use strum::IntoEnumIterator;
  use tower::ServiceExt;

  #[fixture]
  fn app() -> Router {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .with_hub_service()
      .build()
      .unwrap();
    let service = Arc::new(service);
    let mut router_state = MockRouterState::new();
    router_state
      .expect_app_service()
      .returning(move || service.clone());
    Router::new()
      .route("/api/models", get(list_local_aliases_handler))
      .route("/api/models/:id", get(get_alias_handler))
      .route("/api/chat_templates", get(list_chat_templates_handler))
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_local_aliases_handler(app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(Request::get("/api/models").body(Body::empty()).unwrap())
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(response["page"], 1);
    assert_eq!(response["page_size"], 30);
    assert_eq!(response["total"], 3);
    let data = response["data"].as_array().unwrap();
    assert!(!data.is_empty());
    assert_eq!(
      data.first().unwrap()["alias"].as_str().unwrap(),
      "llama3:instruct"
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_local_aliases_page_size(app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::get("/api/models?page=2&page_size=2")
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(response["page"], 2);
    assert_eq!(response["page_size"], 2);
    assert_eq!(response["total"], 3);
    let data = response["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_local_aliases_over_limit_page_size(app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::get("/api/models?page_size=150")
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .json::<Value>()
      .await?;

    assert_eq!(response["page_size"], 100);
    Ok(())
  }

  #[tokio::test]
  async fn test_list_local_aliases_response_structure() -> anyhow::Result<()> {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()?;
    let service = Arc::new(service);
    let mut router_state = MockRouterState::new();
    router_state
      .expect_app_service()
      .returning(move || service.clone());

    let app = Router::new()
      .route("/api/models", get(list_local_aliases_handler))
      .with_state(Arc::new(router_state));

    let response = app
      .oneshot(Request::get("/api/models").body(Body::empty()).unwrap())
      .await?
      .json::<PaginatedResponse<AliasResponse>>()
      .await?;

    assert!(!response.data.is_empty());
    let first_alias = &response.data[0];
    let expected = AliasResponse {
      alias: "llama3:instruct".to_string(),
      family: Some("llama3".to_string()),
      repo: "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string(),
      filename: "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      features: vec!["chat".to_string()],
      chat_template: "llama3".to_string(),
      snapshot: "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      model_params: HashMap::new(),
      request_params: OAIRequestParamsBuilder::default()
        .stop(vec![
          "<|start_header_id|>".to_string(),
          "<|end_header_id|>".to_string(),
          "<|eot_id|>".to_string(),
        ])
        .build()
        .unwrap(),
      context_params: GptContextParamsBuilder::default()
        .n_keep(24)
        .build()
        .unwrap(),
    };

    assert_eq!(first_alias, &expected);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_local_aliases_sorting(app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::get("/api/models?sort=family&sort_order=desc")
          .body(Body::empty())
          .unwrap(),
      )
      .await?
      .json::<PaginatedResponse<AliasResponse>>()
      .await?;

    assert!(!response.data.is_empty());
    let families: Vec<_> = response.data.iter().map(|a| &a.family).collect();
    assert!(families.windows(2).all(|w| w[0] >= w[1]));

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_alias_handler(app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::get("/api/models/llama3:instruct")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let alias_response = response.json::<AliasResponse>().await?;
    let expected = AliasResponse::llama3();
    assert_eq!(expected, alias_response);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_get_alias_handler_non_existent(app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::get("/api/models/non_existent_alias")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let error_body: Value = response.json().await?;
    assert_eq!(error_body["type"], "alias_not_found");
    assert_eq!(error_body["code"], "not_found");

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_chat_templates_handler(app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::get("/api/chat_templates")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let response = response.json::<Vec<ChatTemplate>>().await?;

    assert_eq!(14, response.len());
    for template_id in ChatTemplateId::iter() {
      assert!(response
        .iter()
        .any(|t| t == &ChatTemplate::Id(template_id.clone())));
    }
    let expected_chat_templates = vec![
      "meta-llama/Llama-2-70b-chat-hf",
      "meta-llama/Meta-Llama-3-70B-Instruct",
      "meta-llama/Meta-Llama-3-8B-Instruct",
      "MyFactory/testalias-gguf",
      "TinyLlama/TinyLlama-1.1B-Chat-v1.0",
    ];
    for repo in expected_chat_templates {
      assert!(response
        .iter()
        .any(|t| t == &ChatTemplate::Repo(Repo::try_from(repo).unwrap())));
    }
    Ok(())
  }
}
