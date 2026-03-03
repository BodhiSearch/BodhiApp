use crate::ValidatedJson;
use axum::{
  body::{to_bytes, Body},
  http::{Request, StatusCode},
  response::Response,
  routing::post,
  Router,
};
use rstest::rstest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::ServiceExt;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
struct TestInput {
  #[validate(length(min = 1))]
  name: String,
}

fn json_request(body: &str) -> Request<Body> {
  Request::builder()
    .method("POST")
    .uri("/")
    .header("Content-Type", "application/json")
    .body(Body::from(body.to_string()))
    .unwrap()
}

async fn parse_response(response: Response) -> Value {
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  serde_json::from_slice(&bytes).unwrap()
}

async fn test_handler(ValidatedJson(input): ValidatedJson<TestInput>) -> String {
  format!("ok: {}", input.name)
}

fn app() -> Router {
  Router::new().route("/", post(test_handler))
}

#[rstest]
#[tokio::test]
async fn test_valid_input_passes_through() {
  let response = app()
    .oneshot(json_request(r#"{"name": "alice"}"#))
    .await
    .unwrap();
  assert_eq!(StatusCode::OK, response.status());
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body = String::from_utf8(bytes.to_vec()).unwrap();
  assert_eq!("ok: alice", body);
}

#[rstest]
#[tokio::test]
async fn test_invalid_field_returns_validation_error() {
  let response = app()
    .oneshot(json_request(r#"{"name": ""}"#))
    .await
    .unwrap();
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = parse_response(response).await;
  let error = &body["error"];
  assert_eq!("Validation failed", error["message"]);
  assert_eq!("invalid_request_error", error["type"]);
  assert_eq!("validation_error", error["code"]);
  assert!(
    error["param"]["name"].is_string(),
    "expected param.name to be a string, got: {}",
    error["param"]
  );
}

#[rstest]
#[tokio::test]
async fn test_malformed_json_returns_json_rejection_error() {
  let response = app().oneshot(json_request("not json")).await.unwrap();
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = parse_response(response).await;
  let error = &body["error"];
  assert_eq!("json_rejection_error", error["code"]);
  assert_eq!("invalid_request_error", error["type"]);
  assert!(error["message"]
    .as_str()
    .unwrap()
    .contains("Invalid JSON in request"));
}

#[rstest]
#[tokio::test]
async fn test_empty_body_returns_json_rejection_error() {
  let response = app().oneshot(json_request("")).await.unwrap();
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = parse_response(response).await;
  let error = &body["error"];
  assert_eq!("json_rejection_error", error["code"]);
}

#[rstest]
#[tokio::test]
async fn test_missing_required_field_returns_json_rejection_error() {
  let response = app().oneshot(json_request("{}")).await.unwrap();
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = parse_response(response).await;
  let error = &body["error"];
  assert_eq!("json_rejection_error", error["code"]);
}
