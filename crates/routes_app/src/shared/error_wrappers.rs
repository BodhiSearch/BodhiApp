use axum::{
  body::Body,
  extract::rejection::JsonRejection,
  response::{IntoResponse, Response},
};
use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Invalid JSON in request: {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest)]
pub struct JsonRejectionError {
  #[from]
  source: JsonRejection,
}

impl IntoResponse for JsonRejectionError {
  fn into_response(self) -> Response {
    let args = self.args();
    let param = if args.is_empty() { None } else { Some(args) };
    let mut error_obj = serde_json::json!({
      "message": self.to_string(),
      "type": self.error_type(),
      "code": self.code(),
    });
    if let Some(p) = param {
      error_obj["param"] = serde_json::to_value(p).unwrap_or(serde_json::Value::Null);
    }
    let body = serde_json::json!({ "error": error_obj });
    let body_str = serde_json::to_string(&body).unwrap_or_else(|e| format!("{:?}", e));
    Response::builder()
      .status(self.status())
      .header("Content-Type", "application/json")
      .body(Body::from(body_str))
      .unwrap()
  }
}

#[cfg(test)]
mod tests {
  use crate::JsonRejectionError;
  use axum::{
    body::{to_bytes, Body},
    http::StatusCode,
    response::Response,
    routing::get,
    Json, Router,
  };
  use axum_extra::extract::WithRejection;
  use rstest::rstest;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use tower::ServiceExt;

  async fn parse<T: serde::de::DeserializeOwned>(response: Response<Body>) -> T {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
  }

  #[rstest]
  #[tokio::test]
  async fn test_json_rejection_error() {
    #[derive(Debug, Serialize, Deserialize)]
    struct Input {
      source: String,
    }

    async fn with_json_rejection(
      WithRejection(Json(value), _): WithRejection<Json<Input>, JsonRejectionError>,
    ) -> Response {
      Response::builder()
        .status(418)
        .body(Body::from(format!(
          "{{\"message\": \"ok - {}\"}}",
          value.source
        )))
        .unwrap()
    }

    let router = Router::new().route("/", get(with_json_rejection));
    let response = router
      .oneshot(
        axum::http::Request::builder()
          .uri("/")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = parse::<Value>(response).await;
    assert_eq!(
      json! {{
        "error": {
          "message": "Invalid JSON in request: Expected request with `Content-Type: application/json`.",
          "type": "invalid_request_error",
          "code": "json_rejection_error",
          "param": {"source": "Expected request with `Content-Type: application/json`"}
        }
      }},
      response
    );
  }
}
