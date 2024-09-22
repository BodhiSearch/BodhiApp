use crate::{oai::OpenAIApiError, shared_rw::ContextError};
use async_openai::error::OpenAIError;
use axum::{
  extract::rejection::JsonRejection,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use derive_builder::Builder;
use objs::BuilderError;
use objs::ObjError;
use serde::{Deserialize, Serialize};
use services::SecretServiceError;
use services::{db::DbError, AuthServiceError, DataServiceError, HubServiceError};
use std::{io, sync::Arc};
use thiserror::Error;
use tokio::task::JoinError;
use validator::ValidationErrors;

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

impl From<AuthServiceError> for HttpError {
  fn from(value: AuthServiceError) -> Self {
    let msg = match value {
      AuthServiceError::Reqwest(msg) => msg,
      AuthServiceError::AuthServiceApiError(msg) => msg,
    };
    HttpErrorBuilder::default()
      .internal_server(Some(&msg))
      .build()
      .unwrap()
  }
}

impl From<SecretServiceError> for HttpError {
  fn from(err: SecretServiceError) -> Self {
    HttpErrorBuilder::default()
      .internal_server(Some(&err.to_string()))
      .build()
      .unwrap()
  }
}

#[derive(Debug, Error)]
pub enum BodhiError {
  #[error(
    r#"model alias '{0}' not found in pre-configured model aliases.
Run `bodhi list -r` to see list of pre-configured model aliases
"#
  )]
  AliasNotFound(String),
  #[error("model alias '{0}' already exists")]
  AliasExists(String),
  #[error("$HOME directory not found, set home directory using $HOME")]
  HomeDirectory,

  #[error(transparent)]
  Common(#[from] Common),
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  ObjError(#[from] ObjError),
  #[error(transparent)]
  DataService(#[from] DataServiceError),
  #[error(transparent)]
  HubServiceError(#[from] HubServiceError),
  // TODO: replace when async-openai is internal crate
  #[error(transparent)]
  BuildError(#[from] OpenAIError),
  #[error(transparent)]
  OpenAIApiError(#[from] OpenAIApiError),
  #[error(transparent)]
  AxumHttp(#[from] axum::http::Error),
  #[error(transparent)]
  OAuthError(#[from] AuthServiceError),
  #[error(transparent)]
  Db(#[from] DbError),
  #[error(transparent)]
  Join(#[from] JoinError),
}

pub type Result<T> = std::result::Result<T, BodhiError>;

#[derive(Debug, thiserror::Error)]
pub enum Common {
  #[error("io_file: {source}\npath='{path}'")]
  IoFile {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("io_error_dir_create: {source}\npath='{path}'")]
  IoDir {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("io: {0}")]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  SerdeYamlDeserialize(#[from] serde_yaml::Error),
  #[error("serde_yaml_serialize: {source}\nfilename='{filename}'")]
  SerdeYamlSerialize {
    #[source]
    source: serde_yaml::Error,
    filename: String,
  },
  #[error("serde_json_serialize: {source}\nvalue: {value}")]
  SerdeJsonSerialize {
    #[source]
    source: serde_json::Error,
    value: String,
  },
  #[error("serde_json_deserialize: {0}")]
  SerdeJsonDeserialize(#[from] serde_json::Error),
  #[error(transparent)]
  Validation(#[from] ValidationErrors),
  #[error("stderr: {0}")]
  Stdlib(#[from] Arc<dyn std::error::Error + Send + Sync>),
  #[error("sender_err: error sending signal using channel for '{0}'")]
  Sender(String),
}

#[cfg(test)]
mod tests {
  use crate::{
    test_utils::ResponseTestExt, BadRequestError, ErrorBody, HttpError, HttpErrorBuilder,
  };
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Json, Router,
  };
  use axum_extra::extract::WithRejection;
  use rstest::rstest;
  use serde::Deserialize;
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
    async fn error_handler() -> std::result::Result<(), HttpError> {
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
  ) -> std::result::Result<String, BadRequestError> {
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
