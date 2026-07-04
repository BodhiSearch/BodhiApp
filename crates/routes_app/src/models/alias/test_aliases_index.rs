use crate::test_utils::RequestAuthContextExt;
use crate::{models_copy, models_destroy, models_index, models_show};
use anyhow_trace::anyhow_trace;
use axum::{
  body::Body,
  http::Request,
  routing::{get, post},
  Router,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use server_core::test_utils::{router_state_stub, ResponseTestExt};
use services::{AliasResponse, PaginatedAliasResponse};
use services::{
  AuthContext, McpGrant, ModelGrant, ResourceRole, TokenGrants, TokenGrantsV1, TokenScope,
};
use std::sync::Arc;
use tower::ServiceExt;

/// True if every alias (and every inner API model) reports the expected `access`.
fn all_access_is(data: &[Value], expected: bool) -> bool {
  data.iter().all(|alias| {
    let top = alias["access"].as_bool() == Some(expected);
    let models_ok = alias["models"].as_array().map_or(true, |models| {
      models
        .iter()
        .all(|m| m["access"].as_bool() == Some(expected))
    });
    top && models_ok
  })
}

fn test_router(app_service: Arc<dyn services::AppService>) -> Router {
  Router::new()
    .route("/api/models", get(models_index))
    .route("/api/models/{id}", get(models_show).delete(models_destroy))
    .route("/api/models/{id}/copy", post(models_copy))
    .with_state(app_service)
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_handler(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
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
  // Session (Unrestricted) ⇒ every alias and every inner API model is accessible.
  assert!(all_access_is(data, true));
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_aliases_scoped_token_stamps_access_false(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  // models_list on + empty model grant ⇒ every alias is still listed (visibility), but
  // none is inference-granted, so `access:false` is stamped across all variants.
  let token = AuthContext::test_api_token_with_grants(
    "test-user",
    TokenScope::User,
    TokenGrants::V1(TokenGrantsV1 {
      models_list: true,
      models: ModelGrant::Specific { ids: vec![] },
      mcps_list: false,
      mcps: McpGrant::Specific { ids: vec![] },
    }),
  );
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models")
        .body(Body::empty())?
        .with_auth_context(token),
    )
    .await?
    .json::<Value>()
    .await?;
  let data = response["data"].as_array().unwrap();
  assert!(!data.is_empty());
  assert!(all_access_is(data, false));
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_local_aliases_page_size(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models?page=2&page_size=4")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
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
#[anyhow_trace]
async fn test_list_local_aliases_over_limit_page_size(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models?page_size=150")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
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
#[anyhow_trace]
async fn test_list_local_aliases_response_structure(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?
    .json::<PaginatedAliasResponse>()
    .await?;

  assert!(!response.data.is_empty());
  let user_alias_response = response
    .data
    .iter()
    .find_map(|alias| match alias {
      AliasResponse::User(user_alias) if user_alias.alias == "llama3:instruct" => Some(user_alias),
      _ => None,
    })
    .unwrap();
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
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = test_router(router_state_stub)
    .oneshot(
      Request::get("/api/models?sort=repo&sort_order=desc")
        .body(Body::empty())?
        .with_auth_context(AuthContext::test_session(
          "test-user",
          "testuser",
          ResourceRole::User,
        )),
    )
    .await?
    .json::<PaginatedAliasResponse>()
    .await?;

  assert!(!response.data.is_empty());
  Ok(())
}

async fn list_with_query(
  app_service: Arc<dyn services::AppService>,
  query: &str,
) -> anyhow::Result<PaginatedAliasResponse> {
  Ok(
    test_router(app_service)
      .oneshot(
        Request::get(format!("/api/models?{query}"))
          .body(Body::empty())?
          .with_auth_context(AuthContext::test_session(
            "test-user",
            "testuser",
            ResourceRole::User,
          )),
      )
      .await?
      .json::<PaginatedAliasResponse>()
      .await?,
  )
}

fn source_of(alias: &AliasResponse) -> &str {
  match alias {
    AliasResponse::User(_) => "user",
    AliasResponse::Model(_) => "model",
    AliasResponse::Api(_) => "api",
    AliasResponse::ModelRouter(_) => "model_router",
  }
}

/// Local (auto-discovered) Model rows carry a resolved on-disk `size` from the test hub cache.
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_aliases_local_rows_carry_size(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = list_with_query(router_state_stub, "page_size=100").await?;
  // At least one local (Model) row resolves to a real file → Some(size); API rows never have size.
  let mut local_with_size = 0;
  for alias in &response.data {
    match alias {
      AliasResponse::Model(m) => {
        if m.size.is_some() {
          local_with_size += 1;
        }
      }
      AliasResponse::Api(a) => {
        // serde flattens; the api variant simply has no `size` field — nothing to assert beyond type
        let _ = a;
      }
      _ => {}
    }
  }
  assert!(
    local_with_size > 0,
    "expected at least one auto-discovered local row to carry a resolved size"
  );
  Ok(())
}

/// TYPE facet: `type=api_model` returns only API rows; an unfiltered list has more rows.
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_aliases_filter_type_api_model(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let all = list_with_query(router_state_stub.clone(), "page_size=100").await?;
  let api_only = list_with_query(router_state_stub, "page_size=100&type=api_model").await?;
  // total reflects the filtered set (server-side, pre-pagination)
  assert_eq!(api_only.total, api_only.data.len());
  assert!(api_only.total <= all.total);
  assert!(
    api_only.data.iter().all(|a| source_of(a) == "api"),
    "type=api_model must return only api rows"
  );
  Ok(())
}

/// TYPE facet is multi-value: `type=local_file,model_alias` returns only local rows (no api/router).
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_aliases_filter_type_local_multi(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let local = list_with_query(
    router_state_stub,
    "page_size=100&type=local_file,model_alias",
  )
  .await?;
  assert_eq!(local.total, local.data.len());
  assert!(
    local
      .data
      .iter()
      .all(|a| matches!(source_of(a), "model" | "user")),
    "type=local_file,model_alias must return only local (model/user) rows"
  );
  Ok(())
}

/// API-FORMAT facet applies only to API rows; every returned row is an OpenAI-format API alias
/// (or the set is empty if the fixture has none). Non-API rows are filtered out by the facet.
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_aliases_filter_api_format_openai(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = list_with_query(router_state_stub, "page_size=100&api_format=openai").await?;
  assert_eq!(response.total, response.data.len());
  for alias in &response.data {
    match alias {
      AliasResponse::Api(a) => assert_eq!(
        a.api_format,
        services::ApiFormat::OpenAI,
        "api_format=openai must return only openai API rows"
      ),
      other => panic!("api_format facet returned a non-api row: {other:?}"),
    }
  }
  Ok(())
}

/// SIZE facet: a `size_min` larger than every local file hides all local rows, but leaves
/// API/router rows (which have no local file) untouched.
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_aliases_filter_size_min_excludes_local(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  // 1 TB — larger than any test fixture file.
  let huge = list_with_query(router_state_stub, "page_size=100&size_min=1099511627776").await?;
  assert!(
    huge.data.iter().all(|a| !matches!(source_of(a), "model")),
    "size_min above all files must drop sized local rows"
  );
  // API/router rows survive the size facet (no local file to compare).
  Ok(())
}

/// CAPABILITY facet excludes API/router rows and any local row without matching metadata. With no
/// capability metadata seeded in the stub, the result is empty (no false positives).
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_aliases_filter_capability_without_metadata_is_empty(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let response = list_with_query(router_state_stub, "page_size=100&capability=vision").await?;
  assert_eq!(
    0, response.total,
    "capability=vision must return nothing when no rows have vision metadata"
  );
  assert!(response.data.is_empty());
  Ok(())
}

/// SEARCH: case-insensitive substring over alias/repo/filename narrows the list, every returned row
/// matches the query, and `total` reflects the filtered set.
#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_list_aliases_filter_search(
  #[future] router_state_stub: Arc<dyn services::AppService>,
) -> anyhow::Result<()> {
  let all = list_with_query(router_state_stub.clone(), "page_size=100").await?;
  // 'llama' matches the user alias `llama3:instruct` (repo QuantFactory/Meta-Llama-3-8B-...).
  let hits = list_with_query(router_state_stub, "page_size=100&search=LLAMA").await?;
  assert_eq!(hits.total, hits.data.len());
  assert!(hits.total > 0, "expected at least one 'llama' match");
  assert!(
    hits.total < all.total,
    "search must narrow the unfiltered list"
  );
  for alias in &hits.data {
    let hay = match alias {
      AliasResponse::User(u) => format!("{} {} {}", u.alias, u.repo, u.filename),
      AliasResponse::Model(m) => format!("{} {} {}", m.alias, m.repo, m.filename),
      AliasResponse::Api(a) => format!("{} {} {}", a.id, a.name, a.base_url),
      AliasResponse::ModelRouter(r) => r.alias.clone(),
    };
    assert!(
      hay.to_lowercase().contains("llama"),
      "row {hay:?} does not match search 'llama'"
    );
  }
  Ok(())
}
