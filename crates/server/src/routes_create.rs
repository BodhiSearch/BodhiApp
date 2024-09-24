use crate::{AliasResponse, HttpError, HttpErrorBuilder, RouterState};
use axum::{
  extract::{rejection::JsonRejection, State},
  response::{IntoResponse, Response},
  routing::{post, put},
  Json, Router,
};
use axum_extra::extract::WithRejection;
use commands::CreateCommand;
use hyper::StatusCode;
use objs::{ChatTemplate, GptContextParams, OAIRequestParams, Repo};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

pub fn create_router() -> Router<Arc<dyn RouterState>> {
  Router::new()
    .route("/models", post(create_alias_handler))
    .route("/models/:id", put(update_alias_handler))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAliasRequest {
  alias: Option<String>,
  repo: Repo,
  filename: String,
  snapshot: Option<String>,
  chat_template: ChatTemplate,
  family: Option<String>,
  request_params: Option<OAIRequestParams>,
  context_params: Option<GptContextParams>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateAliasError {
  #[error("invalid request: {0}")]
  JsonRejection(#[from] JsonRejection),
  #[error("alias not found: {0}")]
  AliasNotFound(String),
  #[error("failed to create/update alias: {0}")]
  CommandError(String),
  #[error("missing alias in request")]
  MissingAlias,
  #[error("alias in request does not match path parameter")]
  AliasMismatch,
}

impl From<CreateAliasError> for HttpError {
  fn from(err: CreateAliasError) -> Self {
    let (r#type, code, msg, status) = match err {
      CreateAliasError::JsonRejection(msg) => (
        "invalid_request_error",
        "invalid_value",
        msg.to_string(),
        StatusCode::BAD_REQUEST,
      ),
      CreateAliasError::AliasNotFound(msg) => {
        ("alias_not_found", "not_found", msg, StatusCode::NOT_FOUND)
      }
      CreateAliasError::CommandError(msg) => (
        "invalid_request_error",
        "command_error",
        msg,
        StatusCode::BAD_REQUEST,
      ),
      CreateAliasError::MissingAlias => (
        "invalid_request_error",
        "missing_alias",
        "alias is required".to_string(),
        StatusCode::BAD_REQUEST,
      ),
      CreateAliasError::AliasMismatch => (
        "invalid_request_error",
        "alias_mismatch",
        "alias in request does not match path parameter".to_string(),
        StatusCode::BAD_REQUEST,
      ),
    };
    HttpErrorBuilder::default()
      .status_code(status)
      .r#type(r#type)
      .code(code)
      .message(&msg)
      .build()
      .unwrap()
  }
}

impl IntoResponse for CreateAliasError {
  fn into_response(self) -> Response {
    let err = HttpError::from(self);
    (err.status_code, Json(err.body)).into_response()
  }
}

impl TryFrom<CreateAliasRequest> for CreateCommand {
  type Error = CreateAliasError;

  fn try_from(value: CreateAliasRequest) -> Result<Self, Self::Error> {
    let alias = value.alias.ok_or_else(|| CreateAliasError::MissingAlias)?;
    let result = CreateCommand {
      alias,
      repo: value.repo,
      filename: value.filename,
      snapshot: value.snapshot,
      chat_template: value.chat_template,
      family: value.family,
      auto_download: false,
      update: false,
      oai_request_params: value.request_params.unwrap_or_default(),
      context_params: value.context_params.unwrap_or_default(),
    };
    Ok(result)
  }
}

pub async fn create_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateAliasRequest>, CreateAliasError>,
) -> Result<(StatusCode, Json<AliasResponse>), CreateAliasError> {
  let command = CreateCommand::try_from(payload)?;
  let alias = command.alias.clone();
  match command.execute(state.app_service()) {
    Ok(()) => {
      let alias = state
        .app_service()
        .data_service()
        .find_alias(&alias)
        .ok_or_else(|| CreateAliasError::AliasNotFound(alias))?;
      Ok((StatusCode::CREATED, Json(AliasResponse::from(alias))))
    }
    Err(err) => Err(CreateAliasError::CommandError(err.to_string())),
  }
}

pub async fn update_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  axum::extract::Path(id): axum::extract::Path<String>,
  WithRejection(Json(mut payload), _): WithRejection<Json<CreateAliasRequest>, CreateAliasError>,
) -> Result<(StatusCode, Json<AliasResponse>), CreateAliasError> {
  if payload.alias.is_some() && payload.alias.as_ref() != Some(&id) {
    return Err(CreateAliasError::AliasMismatch);
  }
  payload.alias = Some(id.clone());
  let mut command = CreateCommand::try_from(payload)?;
  command.update = true;

  match command.execute(state.app_service()) {
    Ok(()) => {
      let alias = state
        .app_service()
        .data_service()
        .find_alias(&id)
        .ok_or_else(|| CreateAliasError::AliasNotFound(id))?;
      Ok((StatusCode::OK, Json(AliasResponse::from(alias))))
    }
    Err(err) => Err(CreateAliasError::CommandError(err.to_string())),
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    create_alias_handler, test_utils::ResponseTestExt, update_alias_handler, AliasResponse,
    AliasResponseBuilder, ErrorBody, MockRouterState,
  };
  use axum::{
    body::Body,
    http::{status::StatusCode, Method, Request},
    routing::{post, put},
    Router,
  };
  use objs::{GptContextParamsBuilder, OAIRequestParamsBuilder};
  use rstest::{fixture, rstest};
  use serde_json::Value;
  use services::test_utils::AppServiceStubBuilder;
  use std::collections::HashMap;
  use std::sync::Arc;
  use tower::ServiceExt;

  #[fixture]
  fn app() -> Router {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .with_hub_service()
      .build()
      .unwrap();
    let service = Arc::new(service);
    let mut router_state = MockRouterState::new();
    router_state
      .expect_app_service()
      .returning(move || service.clone());
    Router::new()
      .route("/api/models", post(create_alias_handler))
      .route("/api/models/:id", put(update_alias_handler))
      .with_state(Arc::new(router_state))
  }

  fn payload() -> Value {
    serde_json::json!({
      "alias": "test:alias",
      "repo": "FakeFactory/fakemodel-gguf",
      "filename": "fakemodel.Q4_0.gguf",
      "chat_template": "llama3",
      "family": "test_family",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": {
        "n_ctx": 2048
      }
    })
  }

  fn expected() -> AliasResponse {
    AliasResponseBuilder::default()
      .alias("test:alias".to_string())
      .repo("FakeFactory/fakemodel-gguf")
      .filename("fakemodel.Q4_0.gguf")
      .family(Some("test_family".to_string()))
      .chat_template("llama3")
      .snapshot("5007652f7a641fe7170e0bad4f63839419bd9213")
      .features(vec!["chat".to_string()])
      .model_params(HashMap::new())
      .request_params(
        OAIRequestParamsBuilder::default()
          .temperature(0.7)
          .build()
          .unwrap(),
      )
      .context_params(
        GptContextParamsBuilder::default()
          .n_ctx(2048)
          .build()
          .unwrap(),
      )
      .build()
      .unwrap()
  }

  fn payload_with_snapshot() -> Value {
    serde_json::json!({
      "alias": "test:alias",
      "repo": "FakeFactory/fakemodel-gguf",
      "filename": "fakemodel.Q4_0.gguf",
      "snapshot": "191239b3e26b2882fb562ffccdd1cf0f65402adb",
      "chat_template": "llama3",
      "family": "test_family",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": {
        "n_ctx": 2048
      }
    })
  }

  fn expected_with_snapshot() -> AliasResponse {
    AliasResponseBuilder::default()
      .alias("test:alias".to_string())
      .repo("FakeFactory/fakemodel-gguf")
      .filename("fakemodel.Q4_0.gguf")
      .snapshot("191239b3e26b2882fb562ffccdd1cf0f65402adb")
      .family(Some("test_family".to_string()))
      .chat_template("llama3")
      .features(vec!["chat".to_string()])
      .model_params(HashMap::new())
      .request_params(
        OAIRequestParamsBuilder::default()
          .temperature(0.7)
          .build()
          .unwrap(),
      )
      .context_params(
        GptContextParamsBuilder::default()
          .n_ctx(2048)
          .build()
          .unwrap(),
      )
      .build()
      .unwrap()
  }

  #[rstest]
  #[case(payload(), expected())]
  #[case(payload_with_snapshot(), expected_with_snapshot())]
  #[tokio::test]
  async fn test_create_alias_handler(
    app: Router,
    #[case] payload: Value,
    #[case] expected: AliasResponse,
  ) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::post("/api/models")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::CREATED);
    let response = response.json::<AliasResponse>().await?;
    assert_eq!(response, expected);
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_create_alias_handler_non_existent_repo(app: Router) -> anyhow::Result<()> {
    let payload = serde_json::json!({
      "alias": "test:newalias",
      "repo": "FakeFactory/not-exists",
      "filename": "fakemodel.Q4_0.gguf",
      "chat_template": "llama3",
      "family": "test_family",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": {
        "n_ctx": 2048
      }
    });

    let response = app
      .oneshot(
        Request::post("/api/models")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    // assert_eq!("", response.text().await?);
    let error_body = response.json::<ErrorBody>().await?;
    assert_eq!(error_body.r#type, "invalid_request_error");
    assert_eq!(error_body.code, Some("command_error".to_string()));
    assert_eq!(
      error_body.message,
      "model file 'fakemodel.Q4_0.gguf' not found in repo 'FakeFactory/not-exists'"
    );

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_alias_handler(app: Router) -> anyhow::Result<()> {
    let payload = serde_json::json!({
      "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
      "filename": "tinyllama-1.1b-chat-v0.3.Q2_K.gguf",
      "chat_template": "TinyLlama/TinyLlama-1.1B-Chat-v1.0",
      "family": "tinyllama",
      "request_params": {
        "temperature": 0.8,
        "max_tokens": 2000
      },
      "context_params": {
        "n_ctx": 4096
      }
    });

    let response = app
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/api/models/tinyllama:instruct")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let updated_alias = response.json::<AliasResponse>().await?;
    assert_eq!(
      AliasResponseBuilder::tinyllama_builder()
        .family(Some("tinyllama".to_string()))
        .request_params(
          OAIRequestParamsBuilder::default()
            .temperature(0.8)
            .max_tokens(2000 as u16)
            .build()
            .unwrap()
        )
        .context_params(
          GptContextParamsBuilder::default()
            .n_ctx(4096)
            .build()
            .unwrap()
        )
        .build()
        .unwrap(),
      updated_alias
    );
    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_update_alias_handler_mismatch(app: Router) -> anyhow::Result<()> {
    let payload = serde_json::json!({
      "alias": "llama3:different",
      "repo": "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF",
      "filename": "Meta-Llama-3-8B-Instruct.Q8_0.gguf",
      "chat_template": "llama3"
    });

    let response = app
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/api/models/llama3:instruct")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error_body = response.json::<ErrorBody>().await?;
    assert_eq!(error_body.code, Some("alias_mismatch".to_string()));
    assert_eq!(
      error_body.message,
      "alias in request does not match path parameter"
    );

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_create_alias_handler_missing_alias(app: Router) -> anyhow::Result<()> {
    let payload = serde_json::json!({
      "repo": "FakeFactory/fakemodel-gguf",
      "filename": "fakemodel.Q4_0.gguf",
      "chat_template": "llama3",
      "family": "test_family",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": {
        "n_ctx": 2048
      }
    });

    let response = app
      .oneshot(
        Request::post("/api/models")
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error_body = response.json::<ErrorBody>().await?;
    assert_eq!(error_body.r#type, "invalid_request_error");
    assert_eq!(error_body.code, Some("missing_alias".to_string()));
    assert_eq!(error_body.message, "alias is required");

    Ok(())
  }

  #[rstest]
  #[case(serde_json::json!({
    "alias": "tinyllama:new",
    "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
    "filename": "tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf",
    "chat_template": "TinyLlama/TinyLlama-1.1B-Chat-v1.0",
    "family": "tinyllama",
    "request_params": {
      "temperature": 0.8,
      "max_tokens": 2000
    },
    "context_params": {
      "n_ctx": 4096
    }
  }), Method::POST, "/api/models")]
  #[case(serde_json::json!({
    "alias": "tinyllama:instruct",
    "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
    "filename": "tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf",
    "chat_template": "TinyLlama/TinyLlama-1.1B-Chat-v1.0",
    "family": "tinyllama",
    "request_params": {
      "temperature": 0.8,
      "max_tokens": 2000
    },
    "context_params": {
      "n_ctx": 4096
    }
  }), Method::PUT, "/api/models/tinyllama:instruct")]
  #[tokio::test]
  async fn test_create_alias_repo_not_downloaded_error(
    app: Router,
    #[case] payload: Value,
    #[case] method: Method,
    #[case] url: String,
  ) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::builder()
          .method(method)
          .uri(url)
          .header("Content-Type", "application/json")
          .body(Body::from(serde_json::to_string(&payload)?))
          .unwrap(),
      )
      .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error_body = response.json::<ErrorBody>().await?;
    assert_eq!(error_body.code, Some("command_error".to_string()));
    assert_eq!(
      error_body.message,
      "model file 'tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf' not found in repo 'TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF'"
    );

    Ok(())
  }
}
