use crate::objs::BuilderError;
use axum::{
  extract::rejection::JsonRejection,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

mod status_code {
  use axum::http::StatusCode;
  use serde::{self, Deserialize, Deserializer, Serializer};

  pub fn serialize<S>(status_code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_u16(status_code.as_u16())
  }

  pub fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
  where
    D: Deserializer<'de>,
  {
    let code = u16::deserialize(deserializer)?;
    StatusCode::from_u16(code).map_err(serde::de::Error::custom)
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Builder, Default)]
#[builder(setter(strip_option))]
pub struct ErrorBody {
  #[builder(default)]
  pub message: String,
  #[builder(default)]
  pub r#type: String,
  #[builder(default)]
  pub param: Option<String>,
  #[builder(default)]
  pub code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Builder)]
#[builder(setter(strip_option), build_fn(error = BuilderError))]
pub struct HttpError {
  #[serde(with = "status_code")]
  pub status_code: StatusCode,
  #[builder(default)]
  pub body: ErrorBody,
}

impl HttpErrorBuilder {
  pub fn message(&mut self, msg: &str) -> &mut Self {
    self.body.get_or_insert_with(ErrorBody::default).message = msg.to_string();
    self
  }

  pub fn r#type(&mut self, r#type: &str) -> &mut Self {
    self.body.get_or_insert_with(ErrorBody::default).r#type = r#type.to_string();
    self
  }

  pub fn param(&mut self, param: &str) -> &mut Self {
    self.body.get_or_insert_with(ErrorBody::default).param = Some(param.to_string());
    self
  }

  pub fn code(&mut self, code: &str) -> &mut Self {
    self.body.get_or_insert_with(ErrorBody::default).code = Some(code.to_string());
    self
  }

  pub fn bad_request(&mut self, msg: &str) -> &mut Self {
    self.status_code = Some(StatusCode::BAD_REQUEST);
    self.body.get_or_insert_with(ErrorBody::default).r#type = "invalid_request_error".to_string();
    self.body.get_or_insert_with(ErrorBody::default).code = Some("invalid_value".to_string());
    self.body.get_or_insert_with(ErrorBody::default).message = msg.to_string();
    self
  }

  pub fn unauthorized(&mut self, msg: &str, code: Option<&str>) -> &mut Self {
    self.status_code = Some(StatusCode::UNAUTHORIZED);
    self.body.get_or_insert_with(ErrorBody::default).r#type = "invalid_request_error".to_string();
    self.body.get_or_insert_with(ErrorBody::default).message = msg.to_string();
    if let Some(code) = code {
      self.body.get_or_insert_with(ErrorBody::default).code = Some(code.to_string());
    }
    self
  }

  pub fn forbidden(&mut self, msg: &str) -> &mut Self {
    self.status_code = Some(StatusCode::FORBIDDEN);
    self.body.get_or_insert_with(ErrorBody::default).message = msg.to_string();
    self.body.get_or_insert_with(ErrorBody::default).r#type = "invalid_request_error".to_string();
    self
  }

  pub fn not_found(&mut self, msg: &str) -> &mut Self {
    self.status_code = Some(StatusCode::NOT_FOUND);
    self.body.get_or_insert_with(ErrorBody::default).message = msg.to_string();
    self.body.get_or_insert_with(ErrorBody::default).r#type = "invalid_request_error".to_string();
    self
  }

  pub fn internal_server(&mut self, msg: Option<&str>) -> &mut Self {
    self.status_code = Some(StatusCode::INTERNAL_SERVER_ERROR);
    self.body.get_or_insert_with(ErrorBody::default).r#type = "internal_server_error".to_string();
    self.body.get_or_insert_with(ErrorBody::default).code =
      Some("internal_server_error".to_string());
    if let Some(msg) = msg {
      self.body.get_or_insert_with(ErrorBody::default).message = msg.to_string();
    }
    self
  }
}

impl IntoResponse for HttpError {
  fn into_response(self) -> Response {
    (self.status_code, Json(self.body)).into_response()
  }
}

pub struct BadRequestError(String);

impl From<JsonRejection> for BadRequestError {
  fn from(value: JsonRejection) -> Self {
    match value {
      JsonRejection::JsonDataError(e) => BadRequestError(format!("JSONDataError: {e}")),
      JsonRejection::JsonSyntaxError(e) => BadRequestError(format!("JSONSyntaxError: {e}")),
      JsonRejection::MissingJsonContentType(e) => {
        BadRequestError(format!("MissingContentType: {e}"))
      }
      JsonRejection::BytesRejection(e) => BadRequestError(format!("BytesRejection: {e}")),
      err => BadRequestError(format!("{err:?}")),
    }
  }
}

impl IntoResponse for BadRequestError {
  fn into_response(self) -> Response {
    (
      StatusCode::BAD_REQUEST,
      HttpErrorBuilder::default()
        .bad_request(&format!(
          "We could not parse the JSON body of your request: {}",
          self.0
        ))
        .build()
        .unwrap(),
    )
      .into_response()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_utils::ResponseTestExt;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
  };
  use axum_extra::extract::WithRejection;
  use rstest::rstest;
  use serde_json::Value;
  use tower::ServiceExt;

  #[rstest]
  fn test_http_error_builder() -> anyhow::Result<()> {
    let error = HttpErrorBuilder::default()
      .status_code(StatusCode::BAD_REQUEST)
      .message("Invalid input")
      .r#type("validation_error")
      .param("username")
      .code("invalid_characters")
      .build()?;

    let expected = HttpError {
      status_code: StatusCode::BAD_REQUEST,
      body: ErrorBody {
        message: "Invalid input".to_string(),
        r#type: "validation_error".to_string(),
        param: Some("username".to_string()),
        code: Some("invalid_characters".to_string()),
      },
    };
    assert_eq!(expected, error);
    Ok(())
  }

  #[rstest]
  fn test_http_error_builder_partial() -> anyhow::Result<()> {
    let error = HttpErrorBuilder::default()
      .status_code(StatusCode::NOT_FOUND)
      .message("Resource not found")
      .r#type("not_found_error")
      .build()?;
    let expected = HttpError {
      status_code: StatusCode::NOT_FOUND,
      body: ErrorBody {
        message: "Resource not found".to_string(),
        r#type: "not_found_error".to_string(),
        param: None,
        code: None,
      },
    };
    assert_eq!(expected, error);
    Ok(())
  }

  #[rstest]
  fn test_http_error_builder_default_body() -> anyhow::Result<()> {
    let error = HttpErrorBuilder::default()
      .status_code(StatusCode::INTERNAL_SERVER_ERROR)
      .build()?;
    let expected = HttpError {
      status_code: StatusCode::INTERNAL_SERVER_ERROR,
      body: ErrorBody::default(),
    };
    assert_eq!(expected, error);
    Ok(())
  }

  #[rstest]
  fn test_http_error_builder_missing_status_code() -> anyhow::Result<()> {
    let result = HttpErrorBuilder::default().build();
    assert!(result.is_err());
    Ok(())
  }

  #[rstest]
  fn test_http_error_serialization() -> anyhow::Result<()> {
    let error = HttpErrorBuilder::default()
      .status_code(StatusCode::BAD_REQUEST)
      .message("Invalid input")
      .r#type("validation_error")
      .build()?;
    let serialized = serde_json::to_string(&error)?;
    let deserialized: HttpError = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized, error);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_http_error_into_response() -> anyhow::Result<()> {
    async fn error_handler() -> Result<(), HttpError> {
      let err = HttpErrorBuilder::default()
        .status_code(StatusCode::BAD_REQUEST)
        .message("Invalid input")
        .r#type("validation_error")
        .param("username")
        .code("invalid_characters")
        .build()
        .unwrap();
      Err(err)
    }
    let app = Router::new().route("/test", get(error_handler));
    let request = Request::builder().uri("/test").body(Body::empty())?;
    let response = app.oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body: Value = response.json().await?;
    let expected_body = serde_json::json!({
        "message": "Invalid input",
        "type": "validation_error",
        "param": "username",
        "code": "invalid_characters"
    });
    assert_eq!(expected_body, body);
    Ok(())
  }

  #[derive(Deserialize)]
  struct TestRequest {
    field1: String,
    field2: i32,
  }

  async fn test_handler(
    WithRejection(Json(payload), _): WithRejection<Json<TestRequest>, BadRequestError>,
  ) -> Result<String, BadRequestError> {
    Ok(format!("Received: {}, {}", payload.field1, payload.field2))
  }

  fn create_test_app() -> Router {
    Router::new().route("/test", post(test_handler))
  }

  #[rstest]
  #[case::json_data_error(
    r#"{"field1": "test", "field2": "not_a_number"}"#,
    Some("application/json"),
    "JSONDataError: Failed to deserialize the JSON body into the target type"
  )]
  #[case::json_syntax_error(
    r#"{"field1": "test", "field2": 42,}"#,
    Some("application/json"),
    "JSONSyntaxError: Failed to parse the request body as JSON"
  )]
  #[case::missing_content_type(
    r#"{"field1": "test", "field2": 42}"#,
    None,
    "MissingContentType: Expected request with `Content-Type: application/json`"
  )]
  #[case::empty_body(
    "",
    Some("application/json"),
    "JSONSyntaxError: Failed to parse the request body as JSON"
  )]
  #[tokio::test]
  async fn test_json_errors(
    #[case] body: &str,
    #[case] content_type: Option<&str>,
    #[case] expected_error_message: &str,
  ) -> anyhow::Result<()> {
    let app = create_test_app();
    let mut builder = Request::builder().method("POST").uri("/test");
    if let Some(ct) = content_type {
      builder = builder.header("content-type", ct);
    }

    let response = app
      .oneshot(builder.body(Body::from(body.to_string())).unwrap())
      .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error: ErrorBody = response.json().await.unwrap();
    assert_eq!(
      ErrorBody {
        message: format!(
          "We could not parse the JSON body of your request: {expected_error_message}"
        ),
        r#type: "invalid_request_error".to_string(),
        param: None,
        code: Some("invalid_value".to_string())
      },
      error
    );
    Ok(())
  }
}