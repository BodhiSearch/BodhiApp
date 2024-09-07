use super::RouterStateFn;
use crate::{
  objs::{Alias, GptContextParams, OAIRequestParams},
  service::{HttpError, HttpErrorBuilder},
};
use axum::{
  extract::{Query, State},
  routing::get,
  Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::{collections::HashMap, sync::Arc};

pub fn models_router() -> Router<Arc<dyn RouterStateFn>> {
  Router::new().route("/models", get(list_local_aliases_handler))
}

#[derive(Deserialize)]
pub struct ListQueryParams {
  page: Option<usize>,
  page_size: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ListAliasResponse {
  alias: String,
  family: Option<String>,
  repo: String,
  filename: String,
  snapshot: String,
  features: Vec<String>,
  chat_template: String,
  model_params: HashMap<String, Value>,
  request_params: HashMap<String, Value>,
  context_params: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ListAliasesResponse {
  data: Vec<ListAliasResponse>,
  total: usize,
  page: usize,
  page_size: usize,
}

impl From<Alias> for ListAliasResponse {
  fn from(alias: Alias) -> Self {
    ListAliasResponse {
      alias: alias.alias,
      family: alias.family,
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      features: alias.features,
      chat_template: alias.chat_template.to_string(),
      model_params: HashMap::new(),
      request_params: alias.request_params.into(),
      context_params: alias.context_params.into(),
    }
  }
}

impl From<OAIRequestParams> for HashMap<String, Value> {
  fn from(value: OAIRequestParams) -> Self {
    let mut map = HashMap::new();
    if let Some(frequency_penalty) = value.frequency_penalty {
      map.insert(
        "frequency_penalty".to_string(),
        Value::Number(Number::from_f64(frequency_penalty.into()).unwrap()),
      );
    }
    if let Some(max_tokens) = value.max_tokens {
      map.insert(
        "max_tokens".to_string(),
        Value::Number(Number::from(max_tokens)),
      );
    }
    if let Some(presence_penalty) = value.presence_penalty {
      map.insert(
        "presence_penalty".to_string(),
        Value::Number(Number::from_f64(presence_penalty.into()).unwrap()),
      );
    }
    if let Some(seed) = value.seed {
      map.insert("seed".to_string(), Value::Number(Number::from(seed)));
    }
    if let Some(temperature) = value.temperature {
      map.insert(
        "temperature".to_string(),
        Value::Number(Number::from_f64(temperature.into()).unwrap()),
      );
    }
    if let Some(top_p) = value.top_p {
      map.insert(
        "top_p".to_string(),
        Value::Number(Number::from_f64(top_p.into()).unwrap()),
      );
    }
    map.insert(
      "stop".to_string(),
      Value::Array(value.stop.into_iter().map(Value::String).collect()),
    );
    map
  }
}

impl From<GptContextParams> for HashMap<String, Value> {
  fn from(value: GptContextParams) -> Self {
    let mut map = HashMap::new();
    if let Some(n_seed) = value.n_seed {
      map.insert("n_seed".to_string(), Value::Number(Number::from(n_seed)));
    }
    if let Some(n_threads) = value.n_threads {
      map.insert(
        "n_threads".to_string(),
        Value::Number(Number::from(n_threads)),
      );
    }
    if let Some(n_ctx) = value.n_ctx {
      map.insert("n_ctx".to_string(), Value::Number(Number::from(n_ctx)));
    }
    if let Some(n_parallel) = value.n_parallel {
      map.insert(
        "n_parallel".to_string(),
        Value::Number(Number::from(n_parallel)),
      );
    }
    if let Some(n_predict) = value.n_predict {
      map.insert(
        "n_predict".to_string(),
        Value::Number(Number::from(n_predict)),
      );
    }
    if let Some(n_keep) = value.n_keep {
      map.insert("n_keep".to_string(), Value::Number(Number::from(n_keep)));
    }
    map
  }
}

pub async fn list_local_aliases_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
  Query(params): Query<ListQueryParams>,
) -> Result<Json<ListAliasesResponse>, HttpError> {
  let page = params.page.unwrap_or(1).max(1);
  let page_size = params.page_size.unwrap_or(30).min(100);

  let aliases = state
    .app_service()
    .data_service()
    .list_aliases()
    .map_err(|err| {
      HttpErrorBuilder::default()
        .internal_server(Some(&err.to_string()))
        .build()
        .unwrap()
    })?;

  let total = aliases.len();
  let start = (page - 1) * page_size;
  let end = (start + page_size).min(total);

  let data: Vec<ListAliasResponse> = aliases
    .into_iter()
    .skip(start)
    .take(end - start)
    .map(ListAliasResponse::from)
    .collect();

  Ok(Json(ListAliasesResponse {
    data,
    total,
    page,
    page_size,
  }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_utils::{AppServiceStubBuilder, MockRouterState, ResponseTestExt};
  use axum::{body::Body, http::Request, routing::get, Router};
  use rstest::{fixture, rstest};
  use serde_json::{Number, Value};
  use std::sync::Arc;
  use tower::ServiceExt;

  #[fixture]
  fn app() -> Router {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()
      .unwrap();
    let service = Arc::new(service);
    let mut router_state = MockRouterState::new();
    router_state
      .expect_app_service()
      .returning(move || service.clone());

    let app = Router::new()
      .route("/api/aliases", get(list_local_aliases_handler))
      .with_state(Arc::new(router_state));
    app
  }

  #[rstest]
  #[tokio::test]
  async fn test_list_local_aliases_handler(app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(Request::get("/api/aliases").body(Body::empty()).unwrap())
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
        Request::get("/api/aliases?page=2&page_size=2")
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
        Request::get("/api/aliases?page_size=150")
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
      .route("/api/aliases", get(list_local_aliases_handler))
      .with_state(Arc::new(router_state));

    let response = app
      .oneshot(Request::get("/api/aliases").body(Body::empty()).unwrap())
      .await?
      .json::<ListAliasesResponse>()
      .await?;

    assert!(!response.data.is_empty());
    let first_alias = &response.data[0];
    let request_params = maplit::hashmap! {
      "stop".to_string() => Value::Array(vec![
        Value::String("<|start_header_id|>".to_string()),
        Value::String("<|end_header_id|>".to_string()),
        Value::String("<|eot_id|>".to_string()),
      ]),
    };
    let context_params = maplit::hashmap! {
      "n_keep".to_string() => Value::Number(Number::from(24)),
    };
    let expected = ListAliasResponse {
      alias: "llama3:instruct".to_string(),
      family: Some("llama3".to_string()),
      repo: "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string(),
      filename: "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      features: vec!["chat".to_string()],
      chat_template: "llama3".to_string(),
      snapshot: "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      model_params: HashMap::new(),
      request_params,
      context_params,
    };

    assert_eq!(first_alias, &expected);
    Ok(())
  }
}
