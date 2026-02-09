use crate::{
  copy_alias_handler, create_alias_handler, delete_alias_handler, get_user_alias_handler,
  list_aliases_handler, update_alias_handler, AliasResponse, PaginatedAliasResponse,
  UserAliasResponse,
};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::{status::StatusCode, Method, Request},
  routing::{get, post, put},
  Router,
};
use objs::OAIRequestParamsBuilder;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use serde_json::Value;
use server_core::{
  test_utils::{router_state_stub, RequestTestExt, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::test_utils::{app_service_stub, AppServiceStub};
use std::sync::Arc;
use tower::ServiceExt;

// === Tests from routes_models.rs ===

fn test_router(router_state_stub: DefaultRouterState) -> Router {
  Router::new()
    .route("/api/models", get(list_aliases_handler))
    .route(
      "/api/models/{id}",
      get(get_user_alias_handler).delete(delete_alias_handler),
    )
    .route("/api/models/{id}/copy", post(copy_alias_handler))
    .with_state(Arc::new(router_state_stub))
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_handler(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(Request::get("/api/models").body(Body::empty())?)
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
#[anyhow_trace]
async fn test_list_local_aliases_page_size(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(Request::get("/api/models?page=2&page_size=4").body(Body::empty())?)
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
#[anyhow_trace]
async fn test_list_local_aliases_over_limit_page_size(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(Request::get("/api/models?page_size=150").body(Body::empty())?)
    .await?
    .json::<Value>()
    .await?;

  assert_eq!(100, response["page_size"]);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_response_structure(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(Request::get("/api/models").body(Body::empty())?)
    .await?
    .json::<PaginatedAliasResponse>()
    .await?;

  assert!(!response.data.is_empty());
  // Find a user alias in the discriminated union format
  let user_alias_response = response
    .data
    .iter()
    .find_map(|alias| match alias {
      AliasResponse::User(user_alias) if user_alias.alias == "llama3:instruct" => Some(user_alias),
      _ => None,
    })
    .unwrap();
  // Verify the response has correct fields
  assert_eq!("llama3:instruct", user_alias_response.alias);
  assert_eq!(
    "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
    user_alias_response.repo
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_sorting(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(Request::get("/api/models?sort=repo&sort_order=desc").body(Body::empty())?)
    .await?
    .json::<PaginatedAliasResponse>()
    .await?;

  assert!(!response.data.is_empty());
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_alias_handler(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(Request::get("/api/models/test-llama3-instruct").body(Body::empty())?)
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  let alias_response = response.json::<UserAliasResponse>().await?;
  assert_eq!("test-llama3-instruct", alias_response.id);
  assert_eq!("llama3:instruct", alias_response.alias);
  assert_eq!("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF", alias_response.repo);
  assert_eq!("user", alias_response.source);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_get_alias_handler_non_existent(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(Request::get("/api/models/non_existent_alias").body(Body::empty())?)
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    "data_service_error-alias_not_found",
    response["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

// === Tests from routes_create.rs ===

#[fixture]
#[awt]
async fn app(#[future] app_service_stub: AppServiceStub) -> Router {
  let router_state = DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service_stub),
  );
  Router::new()
    .route("/api/models", post(create_alias_handler))
    .route(
      "/api/models/{id}",
      put(update_alias_handler).delete(delete_alias_handler),
    )
    .route("/api/models/{id}/copy", post(copy_alias_handler))
    .with_state(Arc::new(router_state))
}

fn payload() -> Value {
  serde_json::json!({
    "alias": "testalias:instruct",
    "repo": "MyFactory/testalias-gguf",
    "filename": "testalias.Q8_0.gguf",

    "family": "testalias",
    "request_params": {
      "temperature": 0.7
    },
    "context_params": [
      "--ctx-size 2048"
    ]
  })
}

fn assert_create_alias_response(response: &UserAliasResponse, expected_snapshot: &str) {
  assert!(!response.id.is_empty(), "id should be a UUID");
  assert_eq!("testalias:instruct", response.alias);
  assert_eq!("MyFactory/testalias-gguf", response.repo);
  assert_eq!("testalias.Q8_0.gguf", response.filename);
  assert_eq!(expected_snapshot, response.snapshot);
  assert_eq!("user", response.source);
  assert_eq!(
    OAIRequestParamsBuilder::default()
      .temperature(0.7)
      .build()
      .unwrap(),
    response.request_params,
  );
  assert_eq!(vec!["--ctx-size 2048".to_string()], response.context_params);
}

fn payload_with_snapshot() -> Value {
  serde_json::json!({
    "alias": "testalias:instruct",
    "repo": "MyFactory/testalias-gguf",
    "filename": "testalias.Q8_0.gguf",
    "snapshot": "5007652f7a641fe7170e0bad4f63839419bd9213",

    "family": "testalias",
    "request_params": {
      "temperature": 0.7
    },
    "context_params": [
      "--ctx-size 2048"
    ]
  })
}

#[rstest]
#[case(payload(), "5007652f7a641fe7170e0bad4f63839419bd9213")]
#[case(payload_with_snapshot(), "5007652f7a641fe7170e0bad4f63839419bd9213")]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_alias_handler(
  #[future] app: Router,
  #[case] payload: Value,
  #[case] expected_snapshot: &str,
) -> anyhow::Result<()> {
  let response = app
    .oneshot(Request::post("/api/models").json(&payload)?)
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let response = response.json::<UserAliasResponse>().await?;
  assert_create_alias_response(&response, expected_snapshot);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_alias_handler_non_existent_repo(#[future] app: Router) -> anyhow::Result<()> {
  let payload = serde_json::json!({
    "alias": "test:newalias",
    "repo": "FakeFactory/not-exists",
    "filename": "fakemodel.Q4_0.gguf",

    "family": "test_family",
    "request_params": {
      "temperature": 0.7
    },
    "context_params": [
      "--ctx-size 2048"
    ]
  });

  let response = app
    .oneshot(Request::post("/api/models").json(&payload)?)
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    "hub_service_error-file_not_found",
    response["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_update_alias_handler(#[future] app: Router) -> anyhow::Result<()> {
  let payload = serde_json::json!({
    "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
    "filename": "tinyllama-1.1b-chat-v0.3.Q2_K.gguf",

    "family": "tinyllama",
    "request_params": {
      "temperature": 0.8,
      "max_tokens": 2000
    },
    "context_params": [
      "--ctx-size 4096"
    ]
  });

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::PUT)
        .uri("/api/models/test-tinyllama-instruct")
        .json(&payload)?,
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let updated_alias: UserAliasResponse = response.json::<UserAliasResponse>().await?;
  assert_eq!("test-tinyllama-instruct", updated_alias.id);
  assert_eq!("tinyllama:instruct", updated_alias.alias);
  assert_eq!("TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF", updated_alias.repo);
  assert_eq!("tinyllama-1.1b-chat-v0.3.Q2_K.gguf", updated_alias.filename);
  assert_eq!("user", updated_alias.source);
  assert_eq!(
    OAIRequestParamsBuilder::default()
      .temperature(0.8)
      .max_tokens(2000_u16)
      .build()?,
    updated_alias.request_params,
  );
  assert_eq!(vec!["--ctx-size 4096".to_string()], updated_alias.context_params);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_alias_handler_missing_alias(#[future] app: Router) -> anyhow::Result<()> {
  let payload = serde_json::json!({
    "repo": "FakeFactory/fakemodel-gguf",
    "filename": "fakemodel.Q4_0.gguf",

    "family": "test_family",
    "request_params": {
      "temperature": 0.7
    },
    "context_params": [
      "--ctx-size 2048"
    ]
  });

  let response = app
    .oneshot(Request::post("/api/models").json(&payload)?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    "json_rejection_error",
    response["error"]["code"].as_str().unwrap()
  );
  Ok(())
}

#[rstest]
#[case(serde_json::json!({
  "alias": "tinyllama:new",
  "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
  "filename": "tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf",

  "family": "tinyllama",
  "request_params": {
    "temperature": 0.8,
    "max_tokens": 2000
  },
  "context_params": [
    "--ctx-size 4096"
  ]
}), Method::POST, "/api/models", "hub_service_error-file_not_found")]
#[case(serde_json::json!({
  "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
  "filename": "tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf",

  "family": "tinyllama",
  "request_params": {
    "temperature": 0.8,
    "max_tokens": 2000
  },
  "context_params": [
    "--ctx-size 4096"
  ]
}), Method::PUT, "/api/models/test-tinyllama-instruct", "hub_service_error-file_not_found")]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_alias_repo_not_downloaded_error(
  #[future] app: Router,
  #[case] payload: Value,
  #[case] method: Method,
  #[case] url: String,
  #[case] expected_error_code: &str,
) -> anyhow::Result<()> {
  let response = app
    .oneshot(Request::builder().method(method).uri(url).json(&payload)?)
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    expected_error_code,
    response["error"]["code"].as_str().unwrap()
  );
  Ok(())
}
