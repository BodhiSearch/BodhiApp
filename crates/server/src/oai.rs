use crate::shared_rw::ContextError;
use crate::{HttpError, HttpErrorBuilder};
use axum::{extract::rejection::JsonRejection, response::IntoResponse};

#[derive(Debug, Clone, thiserror::Error)]
pub enum OpenAIApiError {
  #[error("{0}")]
  ModelNotFound(String),
  #[error("{0}")]
  InternalServer(String),
  #[error("{0}")]
  ContextError(String),
  #[error("{0}")]
  JsonRejection(String),
}

impl From<JsonRejection> for OpenAIApiError {
  fn from(value: JsonRejection) -> Self {
    OpenAIApiError::JsonRejection(value.to_string())
  }
}

impl From<ContextError> for OpenAIApiError {
  fn from(value: ContextError) -> Self {
    OpenAIApiError::ContextError(value.to_string())
  }
}

impl From<OpenAIApiError> for HttpError {
  fn from(value: OpenAIApiError) -> Self {
    match value {
      OpenAIApiError::ModelNotFound(model) => HttpErrorBuilder::default()
        .not_found(&format!("The model '{}' does not exist", model))
        .code("model_not_found")
        .param("model")
        .build()
        .unwrap(),
      OpenAIApiError::ContextError(err) => HttpErrorBuilder::default()
        .internal_server(Some(&err))
        .build()
        .unwrap(),
      OpenAIApiError::InternalServer(err) => HttpErrorBuilder::default()
        .internal_server(Some(&err))
        .build()
        .unwrap(),
      OpenAIApiError::JsonRejection(err) => HttpErrorBuilder::default()
        .bad_request(&err)
        .build()
        .unwrap(),
    }
  }
}

impl IntoResponse for OpenAIApiError {
  fn into_response(self) -> axum::response::Response {
    HttpError::from(self).into_response()
  }
}

#[cfg(test)]
mod tests {
  use crate::{test_utils::ResponseTestExt, OpenAIApiError};
  use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    routing::get,
    Router,
  };
  use rstest::rstest;
  use serde_json::json;
  use tower::ServiceExt;

  async fn error_handler(
    State(error): State<OpenAIApiError>,
  ) -> std::result::Result<(), OpenAIApiError> {
    Err(error)
  }

  fn test_error_router(error: OpenAIApiError) -> Router {
    Router::new()
      .route("/test-error", get(error_handler))
      .with_state(error)
  }

  #[rstest]
  #[case::model_not_found(
        OpenAIApiError::ModelNotFound("gpt-4".to_string()),
        StatusCode::NOT_FOUND,
        json!({
            "message": "The model 'gpt-4' does not exist",
            "type": "invalid_request_error",
            "param": "model",
            "code": "model_not_found"
        })
    )]
  #[case::internal_server(
        OpenAIApiError::InternalServer("Internal error occurred".to_string()),
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({
            "message": "Internal error occurred",
            "type": "internal_server_error",
            "param": null,
            "code": "internal_server_error"
        })
    )]
  #[case::context_error(
        OpenAIApiError::ContextError("Context error".to_string()),
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({
            "message": "Context error",
            "type": "internal_server_error",
            "param": null,
            "code": "internal_server_error"
        })
    )]
  #[case::json_rejection(
        OpenAIApiError::JsonRejection("Invalid JSON".to_string()),
        StatusCode::BAD_REQUEST,
        json!({
            "message": "Invalid JSON",
            "type": "invalid_request_error",
            "param": null,
            "code": "invalid_value"
        })
    )]
  #[tokio::test]
  async fn test_openai_api_error_conversion_and_response(
    #[case] error: OpenAIApiError,
    #[case] expected_status: StatusCode,
    #[case] expected_body: serde_json::Value,
  ) -> anyhow::Result<()> {
    let app = test_error_router(error);

    let request = Request::builder()
      .uri("/test-error")
      .method("GET")
      .body(Body::empty())?;

    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), expected_status);
    let body: serde_json::Value = response.json().await?;
    assert_eq!(body, expected_body);
    Ok(())
  }
}
