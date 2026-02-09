#[cfg(test)]
mod test {
  use crate::routes_ollama::{ollama_model_show_handler, ollama_models_handler};
  use anyhow_trace::anyhow_trace;
  use axum::{
    body::Body,
    http::Request,
    routing::{get, post},
    Router,
  };
  use rstest::rstest;
  use serde_json::{json, Value};
  use server_core::{
    test_utils::{router_state_stub, RequestTestExt, ResponseTestExt},
    DefaultRouterState,
  };
  use std::sync::Arc;
  use tower::ServiceExt;
  use validator::ValidateLength;

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_ollama_routes_models_list(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let app = Router::new()
      .route("/api/tags", get(ollama_models_handler))
      .with_state(Arc::new(router_state_stub));
    let response = app
      .oneshot(Request::get("/api/tags").body(Body::empty()).unwrap())
      .await?
      .json::<Value>()
      .await?;
    // Since llama.cpp now handles chat templates, we include all GGUF files
    assert!(response["models"].as_array().length().unwrap() >= 6);
    let llama3 = response["models"]
      .as_array()
      .unwrap()
      .iter()
      .find(|item| item["model"] == "llama3:instruct")
      .unwrap();
    assert_eq!("5007652f7a641fe7170e0bad4f63839419bd9213", llama3["digest"]);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_ollama_model_show(
    #[future] router_state_stub: DefaultRouterState,
  ) -> anyhow::Result<()> {
    let app = Router::new()
      .route("/api/show", post(ollama_model_show_handler))
      .with_state(Arc::new(router_state_stub));
    let response = app
      .oneshot(Request::post("/api/show").json(json! {{"name": "llama3:instruct"}})?)
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(
      json! {
      {
        "families": null,
        "family": "unknown",
        "format": "gguf",
        "parameter_size": "",
        "parent_model": null,
        "quantization_level": ""
      }},
      response["details"]
    );
    assert_eq!("", response["template"]); // Chat template removed since llama.cpp now handles this
    assert_eq!(
      r#"- --n-keep 24
stop:
- <|start_header_id|>
- <|end_header_id|>
- <|eot_id|>
"#,
      response["parameters"].as_str().unwrap()
    );
    Ok(())
  }

  // Auth tier: User - These Ollama-compatible endpoints are accessible to all authenticated users
  // All roles (User, PowerUser, Manager, Admin) can access these endpoints

  #[rstest]
  #[case::list_models("GET", "/api/tags")]
  #[case::show_model("POST", "/api/show")]
  #[case::chat("POST", "/api/chat")]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_ollama_endpoints_reject_unauthenticated(
    #[case] method: &str,
    #[case] path: &str,
  ) -> anyhow::Result<()> {
    use crate::test_utils::{build_test_router, unauth_request};
    use axum::http::StatusCode;
    use tower::ServiceExt;

    let (router, _, _temp) = build_test_router().await?;
    let response = router.oneshot(unauth_request(method, path)).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_ollama_endpoints_allow_all_roles(
    #[values("resource_user", "resource_power_user", "resource_manager", "resource_admin")]
    role: &str,
    #[values(("GET", "/api/tags"))] endpoint: (&str, &str),
  ) -> anyhow::Result<()> {
    use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
    use axum::http::StatusCode;
    use tower::ServiceExt;

    let (router, app_service, _temp) = build_test_router().await?;
    let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
    let (method, path) = endpoint;
    let response = router.oneshot(session_request(method, path, &cookie)).await?;
    assert_eq!(
      StatusCode::OK,
      response.status(),
      "{role} should be allowed to {method} {path}"
    );
    Ok(())
  }
}
