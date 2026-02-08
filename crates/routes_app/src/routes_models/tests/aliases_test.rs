use crate::{
  create_alias_handler, get_user_alias_handler, list_aliases_handler, update_alias_handler,
  AliasResponse, PaginatedAliasResponse, UserAliasResponse, UserAliasResponseBuilder,
};
use axum::{
  body::Body,
  http::{status::StatusCode, Method, Request},
  routing::{get, post, put},
  Router,
};
use objs::OAIRequestParamsBuilder;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use serde_json::{json, Value};
use server_core::{
  test_utils::{router_state_stub, ResponseTestExt},
  DefaultRouterState, MockSharedContext,
};
use services::test_utils::{app_service_stub, AppServiceStub};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;

// === Tests from routes_models.rs ===

fn test_router(router_state_stub: DefaultRouterState) -> Router {
  Router::new()
    .route("/api/models", get(list_aliases_handler))
    .route("/api/models/{id}", get(get_user_alias_handler))
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
async fn test_list_local_aliases_page_size(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models?page=2&page_size=4")
        .body(Body::empty())
        .unwrap(),
    )
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

  assert_eq!(100, response["page_size"]);
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
async fn test_list_local_aliases_sorting(
  #[future] router_state_stub: DefaultRouterState,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models?sort=repo&sort_order=desc")
        .body(Body::empty())
        .unwrap(),
    )
    .await?
    .json::<PaginatedAliasResponse>()
    .await?;

  assert!(!response.data.is_empty());
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
  assert_eq!(StatusCode::OK, response.status());
  let alias_response = response.json::<UserAliasResponse>().await?;
  let expected = UserAliasResponse::llama3();
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
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    json! {{
      "error": {
        "message": "Model configuration 'non_existent_alias' not found.",
        "code": "alias_not_found_error",
        "type": "not_found_error",
        "param": {
          "var_0": "non_existent_alias"
        }
      }
    }},
    response
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
    .route("/api/models/{id}", put(update_alias_handler))
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

fn expected() -> UserAliasResponse {
  UserAliasResponseBuilder::default()
    .alias("testalias:instruct".to_string())
    .repo("MyFactory/testalias-gguf")
    .filename("testalias.Q8_0.gguf")
    .snapshot("5007652f7a641fe7170e0bad4f63839419bd9213")
    .source("user")
    .model_params(HashMap::new())
    .request_params(
      OAIRequestParamsBuilder::default()
        .temperature(0.7)
        .build()
        .unwrap(),
    )
    .context_params(vec!["--ctx-size 2048".to_string()])
    .build()
    .unwrap()
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

fn expected_with_snapshot() -> UserAliasResponse {
  UserAliasResponseBuilder::default()
    .alias("testalias:instruct".to_string())
    .repo("MyFactory/testalias-gguf")
    .filename("testalias.Q8_0.gguf")
    .snapshot("5007652f7a641fe7170e0bad4f63839419bd9213")
    .source("user")
    .model_params(HashMap::new())
    .request_params(
      OAIRequestParamsBuilder::default()
        .temperature(0.7)
        .build()
        .unwrap(),
    )
    .context_params(vec!["--ctx-size 2048".to_string()])
    .build()
    .unwrap()
}

#[rstest]
#[case(payload(), expected())]
#[case(payload_with_snapshot(), expected_with_snapshot())]
#[tokio::test]
#[awt]
async fn test_create_alias_handler(
  #[future] app: Router,
  #[case] payload: Value,
  #[case] expected: UserAliasResponse,
) -> anyhow::Result<()> {
  let response = app
    .oneshot(
      Request::post("/api/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&payload)?))
        .unwrap(),
    )
    .await?;
  assert_eq!(StatusCode::CREATED, response.status());
  let response = response.json::<UserAliasResponse>().await?;
  assert_eq!(expected, response);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
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
    .oneshot(
      Request::post("/api/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&payload)?))
        .unwrap(),
    )
    .await?;
  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    json! {{
      "error": {
        "type": "not_found_error",
        "code": "hub_file_not_found_error",
        "message": "File 'fakemodel.Q4_0.gguf' not found in repository 'FakeFactory/not-exists'.",
        "param": {
          "filename": "fakemodel.Q4_0.gguf",
          "repo": "FakeFactory/not-exists",
          "snapshot": "main"
        }
      }
    }},
    response
  );
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
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
        .uri("/api/models/tinyllama:instruct")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&payload)?))
        .unwrap(),
    )
    .await?;

  assert_eq!(StatusCode::OK, response.status());
  let updated_alias: UserAliasResponse = response.json::<UserAliasResponse>().await?;
  let expected = UserAliasResponseBuilder::tinyllama_builder()
    .request_params(
      OAIRequestParamsBuilder::default()
        .temperature(0.8)
        .max_tokens(2000_u16)
        .build()
        .unwrap(),
    )
    .context_params(vec!["--ctx-size 4096".to_string()])
    .build()
    .unwrap();
  assert_eq!(expected, updated_alias);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
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
    .oneshot(
      Request::post("/api/models")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&payload)?))
        .unwrap(),
    )
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    json! {{
      "error": {
        "type": "invalid_request_error",
        "code": "json_rejection_error",
        "message": "Invalid JSON in request: Failed to deserialize the JSON body into the target type: missing field `alias` at line 1 column 167.",
        "param": {
          "source": "Failed to deserialize the JSON body into the target type: missing field `alias` at line 1 column 167"
        }
      }
    }},
    response
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
}), Method::POST, "/api/models")]
#[case(serde_json::json!({
  "alias": "tinyllama:instruct",
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
}), Method::PUT, "/api/models/tinyllama:instruct")]
#[awt]
#[tokio::test]
async fn test_create_alias_repo_not_downloaded_error(
  #[future] app: Router,
  #[case] payload: Value,
  #[case] method: Method,
  #[case] url: String,
) -> anyhow::Result<()> {
  let response = app
    .oneshot(
      Request::builder()
        .method(method)
        .uri(url)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&payload)?))
        .unwrap(),
    )
    .await?;

  assert_eq!(StatusCode::NOT_FOUND, response.status());
  let response = response.json::<Value>().await?;
  assert_eq!(
    json! {{
      "error": {
        "type": "not_found_error",
        "code": "hub_file_not_found_error",
        "message": "File 'tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf' not found in repository 'TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF'.",
        "param": {
          "filename": "tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf",
          "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
          "snapshot": "main"
        }
      }
    }},
    response
  );
  Ok(())
}
