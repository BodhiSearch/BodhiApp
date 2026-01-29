use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use objs::{AppError, ErrorBody, OpenAIApiError};

/// HTTP error wrapper that converts AppError types into OpenAI-compatible HTTP responses.
///
/// This wrapper provides a standardized way to convert application errors into HTTP responses
/// that follow the OpenAI API error format, ensuring consistency across all API endpoints.
pub struct HttpError<E: AppError>(pub E);

impl<E: AppError> IntoResponse for HttpError<E> {
  fn into_response(self) -> Response {
    let args = self.0.args();
    let openai_error = OpenAIApiError {
      error: ErrorBody {
        message: self.0.to_string(),
        r#type: self.0.error_type(),
        code: Some(self.0.code()),
        param: if args.is_empty() { None } else { Some(args) },
      },
      status: self.0.status(),
    };
    Response::builder()
      .status(openai_error.status)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&openai_error).unwrap()))
      .unwrap()
  }
}
