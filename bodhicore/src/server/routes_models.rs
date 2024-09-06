use super::RouterStateFn;
use crate::{
  objs::Alias,
  service::{HttpError, HttpErrorBuilder},
};
use axum::{
  extract::{Query, State},
  routing::get,
  Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
  features: Vec<String>,
  chat_template: String,
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
      features: alias.features,
      chat_template: alias.chat_template.to_string(),
    }
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
  use serde_json::Value;
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
    let expected = ListAliasResponse {
      alias: "llama3:instruct".to_string(),
      family: Some("llama3".to_string()),
      repo: "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string(),
      filename: "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      features: vec!["chat".to_string()],
      chat_template: "llama3".to_string(),
    };

    assert_eq!(first_alias, &expected);
    Ok(())
  }
}
