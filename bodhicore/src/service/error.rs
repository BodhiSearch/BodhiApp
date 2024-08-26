use crate::objs::BuilderError;
use axum::{
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

  pub fn invalid_request(&mut self) -> &mut Self {
    self.body.get_or_insert_with(ErrorBody::default).r#type = "invalid_request_error".to_string();
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_utils::ResponseTestExt;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
  };
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
}
