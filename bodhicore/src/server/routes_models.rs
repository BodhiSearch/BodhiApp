use super::RouterStateFn;
use crate::{
  server::{HttpError, HttpErrorBuilder},
  CreateCommand,
};
use axum::extract::rejection::JsonRejection;
use axum::response::{IntoResponse, Response};
use axum::{
  extract::{Query, State},
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::WithRejection;
use hyper::StatusCode;
use objs::{
  Alias, ChatTemplate, ChatTemplateId, GptContextParams, HubFile, OAIRequestParams, Repo,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use strum::IntoEnumIterator;
use validator::Validate;

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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AliasResponse {
  alias: String,
  family: Option<String>,
  repo: String,
  filename: String,
  snapshot: String,
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

pub fn models_router() -> Router<Arc<dyn RouterStateFn>> {
  Router::new()
    .route("/models", get(list_local_aliases_handler))
    .route("/models", post(create_alias_handler))
    .route("/models/:id", get(get_alias_handler))
    .route("/modelfiles", get(list_local_modelfiles_handler))
    .route("/chat_templates", get(list_chat_templates_handler))
}

pub async fn list_local_aliases_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
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
  State(state): State<Arc<dyn RouterStateFn>>,
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAliasRequest {
  alias: String,
  repo: Repo,
  filename: String,
  chat_template: ChatTemplate,
  family: Option<String>,
  request_params: Option<OAIRequestParams>,
  context_params: Option<GptContextParams>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateAliasError {
  #[error("invalid request: {0}")]
  JsonRejection(#[from] JsonRejection),
  #[error("alias not found: {0}")]
  AliasNotFound(String),
  #[error("failed to create alias: {0}")]
  CommandError(String),
}

impl From<CreateAliasError> for HttpError {
  fn from(err: CreateAliasError) -> Self {
    let (r#type, code, msg, status) = match err {
      CreateAliasError::JsonRejection(msg) => (
        "invalid_request_error",
        "invalid_value",
        msg.to_string(),
        StatusCode::BAD_REQUEST,
      ),
      CreateAliasError::AliasNotFound(msg) => {
        ("alias_not_found", "not_found", msg, StatusCode::NOT_FOUND)
      }
      CreateAliasError::CommandError(msg) => (
        "invalid_request_error",
        "command_error",
        msg,
        StatusCode::BAD_REQUEST,
      ),
    };
    HttpErrorBuilder::default()
      .status_code(status)
      .r#type(r#type)
      .code(code)
      .message(&msg)
      .build()
      .unwrap()
  }
}

impl IntoResponse for CreateAliasError {
  fn into_response(self) -> Response {
    let err = HttpError::from(self);
    (err.status_code, Json(err.body)).into_response()
  }
}

impl TryFrom<CreateAliasRequest> for CreateCommand {
  type Error = CreateAliasError;

  fn try_from(value: CreateAliasRequest) -> Result<Self, Self::Error> {
    let result = CreateCommand {
      alias: value.alias,
      repo: value.repo,
      filename: value.filename,
      chat_template: value.chat_template,
      family: value.family,
      force: false,
      auto_download: false,
      oai_request_params: value.request_params.unwrap_or_default(),
      context_params: value.context_params.unwrap_or_default(),
    };
    Ok(result)
  }
}

pub async fn create_alias_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateAliasRequest>, CreateAliasError>,
) -> Result<(StatusCode, Json<AliasResponse>), CreateAliasError> {
  let command = CreateCommand::try_from(payload)?;
  let alias = command.alias.clone();
  match command.execute(state.app_service()) {
    Ok(()) => {
      let alias = state
        .app_service()
        .data_service()
        .find_alias(&alias)
        .ok_or_else(|| CreateAliasError::AliasNotFound(alias))?;
      Ok((StatusCode::CREATED, Json(AliasResponse::from(alias))))
    }
    Err(err) => Err(CreateAliasError::CommandError(err.to_string())),
  }
}

pub async fn list_chat_templates_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
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
  State(state): State<Arc<dyn RouterStateFn>>,
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
  use super::*;
  use crate::test_utils::{MockRouterState, ResponseTestExt};
  use axum::{body::Body, http::Request, routing::get, Router};
  use objs::{GptContextParamsBuilder, OAIRequestParamsBuilder};
  use rstest::{fixture, rstest};
  use serde_json::Value;
  use services::test_utils::AppServiceStubBuilder;
  use std::sync::Arc;
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
      .route("/api/models", post(create_alias_handler))
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
  async fn test_create_alias_handler(app: Router) -> anyhow::Result<()> {
    let payload = serde_json::json!({
      "alias": "test:alias",
      "repo": "FakeFactory/fakemodel-gguf",
      "filename": "fakemodel.Q4_0.gguf",
      "chat_template": "llama3",
      "family": "test_family",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": {
        "n_ctx": 2048
      }
    });

    let response = app
      .oneshot(
        Request::post("/api/models")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::CREATED);
    let response = response.json::<AliasResponse>().await?;
    assert_eq!(
      response,
      AliasResponse {
        alias: "test:alias".to_string(),
        family: Some("test_family".to_string()),
        repo: "FakeFactory/fakemodel-gguf".to_string(),
        filename: "fakemodel.Q4_0.gguf".to_string(),
        chat_template: "llama3".to_string(),
        snapshot: "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
        features: vec!["chat".to_string()],
        model_params: HashMap::new(),
        request_params: OAIRequestParamsBuilder::default()
          .temperature(0.7)
          .build()
          .unwrap(),
        context_params: GptContextParamsBuilder::default()
          .n_ctx(2048)
          .build()
          .unwrap(),
      }
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_create_alias_handler_non_existent_repo(app: Router) -> anyhow::Result<()> {
    let payload = serde_json::json!({
      "alias": "test:newalias",
      "repo": "FakeFactory/not-exists",
      "filename": "fakemodel.Q4_0.gguf",
      "chat_template": "llama3",
      "family": "test_family",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": {
        "n_ctx": 2048
      }
    });

    let response = app
      .oneshot(
        Request::post("/api/models")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    // assert_eq!("", response.text().await?);
    let error_body: Value = response.json().await?;
    assert_eq!(error_body["type"], "invalid_request_error");
    assert_eq!(error_body["code"], "command_error");
    assert!(error_body["message"]
      .as_str()
      .unwrap()
      .contains("file 'fakemodel.Q4_0.gguf' not found in $HF_HOME repo 'FakeFactory/not-exists'"));

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

    assert_eq!(13, response.len());
    for template_id in ChatTemplateId::iter() {
      assert!(response
        .iter()
        .any(|t| t == &ChatTemplate::Id(template_id.clone())));
    }
    let expected = vec![
      "meta-llama/Llama-2-70b-chat-hf",
      "meta-llama/Meta-Llama-3-70B-Instruct",
      "meta-llama/Meta-Llama-3-8B-Instruct",
      "MyFactory/testalias-gguf",
    ];
    for repo in expected {
      assert!(response
        .iter()
        .any(|t| t == &ChatTemplate::Repo(Repo::try_from(repo).unwrap())));
    }
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
    assert_eq!(alias_response.alias, "llama3:instruct");
    assert_eq!(
      alias_response.repo,
      "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"
    );
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
}
