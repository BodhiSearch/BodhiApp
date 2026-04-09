use crate::standalone_inference::proxy_to_remote;
use anyhow_trace::anyhow_trace;
use axum::http::Method;
use rstest::rstest;
use serde_json::{json, Value};
use services::inference::LlmEndpoint;
use services::test_utils::fixed_dt;
use services::{ApiAlias, ApiFormat, MockAiApiService};
use std::sync::Arc;

fn make_test_api_alias() -> ApiAlias {
  ApiAlias::new(
    "test-api",
    ApiFormat::OpenAI,
    "https://api.example.com/v1",
    vec!["gpt-4".to_string()],
    None,
    false,
    fixed_dt(),
  )
}

fn ok_response() -> axum::response::Response {
  axum::response::Response::builder()
    .status(200)
    .body(axum::body::Body::empty())
    .unwrap()
}

#[rstest]
#[case::get_endpoint(
  LlmEndpoint::ResponsesGet("resp-123".to_string()),
  Method::GET,
  Value::Null,
  None,
)]
#[case::delete_endpoint(
  LlmEndpoint::ResponsesDelete("resp-123".to_string()),
  Method::DELETE,
  Value::Null,
  None,
)]
#[case::post_endpoint(
  LlmEndpoint::Responses,
  Method::POST,
  json!({"model": "gpt-4", "input": "hello"}),
  None,
)]
#[case::post_null_body(
  LlmEndpoint::ResponsesCancel("resp-123".to_string()),
  Method::POST,
  Value::Null,
  None,
)]
#[case::get_with_query_params(
  LlmEndpoint::ResponsesGet("resp-456".to_string()),
  Method::GET,
  Value::Null,
  Some(vec![("after".to_string(), "ts_123".to_string())]),
)]
#[anyhow_trace]
#[tokio::test]
async fn test_proxy_to_remote_method_dispatch(
  #[case] endpoint: LlmEndpoint,
  #[case] expected_method: Method,
  #[case] request: Value,
  #[case] query_params: Option<Vec<(String, String)>>,
) -> anyhow::Result<()> {
  let expected_method_cl = expected_method.clone();
  let expect_body = expected_method == Method::POST && request != Value::Null;
  let request_cl = request.clone();
  let query_params_cl = query_params.clone();

  let mut mock_ai = MockAiApiService::new();
  mock_ai
    .expect_forward_request_with_method()
    .withf(move |method, _path, _alias, _key, body, params| {
      let method_ok = *method == expected_method_cl;
      let body_ok = if expect_body {
        body.as_ref() == Some(&request_cl)
      } else {
        body.is_none()
      };
      let params_ok = *params == query_params_cl;
      method_ok && body_ok && params_ok
    })
    .times(1)
    .return_once(|_, _, _, _, _, _| Ok(ok_response()));

  let ai_service: Arc<dyn services::AiApiService> = Arc::new(mock_ai);
  let api_alias = make_test_api_alias();

  let result = proxy_to_remote(
    &ai_service,
    endpoint,
    request,
    &api_alias,
    Some("test-key".to_string()),
    query_params,
  )
  .await;

  assert!(result.is_ok());
  Ok(())
}
