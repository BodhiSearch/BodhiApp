use crate::{AliasResponse, LocalModelResponse, PaginatedResponse, PaginationSortParams};
use axum::{
  extract::{Query, State},
  routing::get,
  Json, Router,
};
use objs::{Alias, ApiError, ChatTemplate, ChatTemplateId, HubFile};
use server_core::RouterState;
use services::AliasNotFoundError;
use std::sync::Arc;
use strum::IntoEnumIterator;

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
) -> Result<Json<PaginatedResponse<AliasResponse>>, ApiError> {
  let (page, page_size, sort, sort_order) = extract_pagination_sort_params(params);
  let mut aliases = state.app_service().data_service().list_aliases()?;
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
) -> Result<Json<PaginatedResponse<LocalModelResponse>>, ApiError> {
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
) -> Result<Json<Vec<ChatTemplate>>, ApiError> {
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
  use crate::{
    get_alias_handler, list_chat_templates_handler, list_local_aliases_handler, AliasResponse,
    PaginatedResponse,
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
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{router_state_stub, ResponseTestExt},
    DefaultRouterState,
  };
  use std::collections::HashMap;
  use std::sync::Arc;
  use strum::IntoEnumIterator;
  use tower::ServiceExt;

  fn test_router(router_state_stub: DefaultRouterState) -> Router {
    Router::new()
      .route("/api/models", get(list_local_aliases_handler))
      .route("/api/models/:id", get(get_alias_handler))
      .route("/api/chat_templates", get(list_chat_templates_handler))
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
  #[awt]
  #[tokio::test]
  async fn test_list_local_aliases_page_size(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
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

    assert_eq!(response["page_size"], 100);
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
    let families: Vec<_> = response.data.iter().map(|a| &a.family).collect();
    assert!(families.windows(2).all(|w| w[0] >= w[1]));

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
    assert_eq!(response.status(), StatusCode::OK);
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
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
      .oneshot(
        Request::get("/api/models/non_existent_alias")
          .body(Body::empty())
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let response = response.json::<Value>().await?;
    assert_eq!(
      response,
      json! {{
        "error": {
          "message": "alias '\u{2068}non_existent_alias\u{2069}' not found in $BODHI_HOME/aliases",
          "code": "alias_not_found_error",
          "type": "not_found_error"
        }
      }}
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_list_chat_templates_handler(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let response = test_router(router_state_stub)
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
      assert!(response.iter().any(|t| t == &ChatTemplate::Id(template_id)));
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
