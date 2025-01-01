use axum::http::StatusCode;
use axum::{
  extract::State,
  routing::{post, put},
  Json, Router,
};
use axum_extra::extract::WithRejection;
use commands::{CreateCommand, CreateCommandError};
use objs::{
  ApiError, AppError, ChatTemplateType, ErrorType, GptContextParams, OAIRequestParams, Repo,
};
use serde::{Deserialize, Serialize};
use server_core::{AliasResponse, RouterState};
use services::AliasNotFoundError;
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
  chat_template: ChatTemplateType,
  family: Option<String>,
  request_params: Option<OAIRequestParams>,
  context_params: Option<GptContextParams>,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum CreateAliasError {
  #[error("alias_not_present")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  AliasNotPresent,
  #[error(transparent)]
  AliasNotFound(#[from] AliasNotFoundError),
  #[error(transparent)]
  CreateCommand(#[from] CreateCommandError),
  #[error("alias_mismatch")]
  #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
  AliasMismatch { path: String, request: String },
}

impl TryFrom<CreateAliasRequest> for CreateCommand {
  type Error = CreateAliasError;

  fn try_from(value: CreateAliasRequest) -> Result<Self, Self::Error> {
    let alias = value.alias.ok_or(CreateAliasError::AliasNotPresent)?;
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
  WithRejection(Json(payload), _): WithRejection<Json<CreateAliasRequest>, ApiError>,
) -> Result<(StatusCode, Json<AliasResponse>), ApiError> {
  let command = CreateCommand::try_from(payload)?;
  let alias = command.alias.clone();
  command.execute(state.app_service())?;
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&alias)
    .ok_or(AliasNotFoundError(alias))?;
  Ok((StatusCode::CREATED, Json(AliasResponse::from(alias))))
}

pub async fn update_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  axum::extract::Path(id): axum::extract::Path<String>,
  WithRejection(Json(mut payload), _): WithRejection<Json<CreateAliasRequest>, ApiError>,
) -> Result<(StatusCode, Json<AliasResponse>), ApiError> {
  if payload.alias.is_some() && payload.alias.as_ref() != Some(&id) {
    return Err(CreateAliasError::AliasMismatch {
      path: id.to_string(),
      request: payload.alias.unwrap(),
    })?;
  }
  payload.alias = Some(id.clone());
  let mut command = CreateCommand::try_from(payload)?;
  command.update = true;
  command.execute(state.app_service())?;
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&id)
    .ok_or(AliasNotFoundError(id))?;
  Ok((StatusCode::OK, Json(AliasResponse::from(alias))))
}

#[cfg(test)]
mod tests {
  use crate::{create_alias_handler, update_alias_handler};
  use axum::{
    body::Body,
    http::{status::StatusCode, Method, Request},
    routing::{post, put},
    Router,
  };
  use objs::{
    test_utils::setup_l10n, FluentLocalizationService, GptContextParamsBuilder,
    OAIRequestParamsBuilder,
  };
  use pretty_assertions::assert_eq;
  use rstest::{fixture, rstest};
  use serde_json::{json, Value};
  use server_core::{
    test_utils::ResponseTestExt, AliasResponse, AliasResponseBuilder, DefaultRouterState,
    MockSharedContext,
  };
  use services::test_utils::{app_service_stub, AppServiceStub};
  use std::collections::HashMap;
  use std::sync::Arc;
  use tower::ServiceExt;

  #[fixture]
  #[awt]
  async fn app(#[future] app_service_stub: AppServiceStub) -> Router {
    let router_state = DefaultRouterState::new(
      Arc::new(MockSharedContext::default()),
      Arc::new(app_service_stub),
    );
    Router::new()
      .route("/api/models", post(create_alias_handler))
      .route("/api/models/:id", put(update_alias_handler))
      .with_state(Arc::new(router_state))
  }

  fn payload() -> Value {
    serde_json::json!({
      "alias": "testalias:instruct",
      "repo": "MyFactory/testalias-gguf",
      "filename": "testalias.Q8_0.gguf",
      "chat_template": "llama3",
      "family": "testalias",
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
      .alias("testalias:instruct".to_string())
      .repo("MyFactory/testalias-gguf")
      .filename("testalias.Q8_0.gguf")
      .chat_template("llama3")
      .family(Some("testalias".to_string()))
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
      "alias": "testalias:instruct",
      "repo": "MyFactory/testalias-gguf",
      "filename": "testalias.Q8_0.gguf",
      "snapshot": "5007652f7a641fe7170e0bad4f63839419bd9213",
      "chat_template": "llama3",
      "family": "testalias",
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
      .alias("testalias:instruct".to_string())
      .repo("MyFactory/testalias-gguf")
      .filename("testalias.Q8_0.gguf")
      .snapshot("5007652f7a641fe7170e0bad4f63839419bd9213")
      .family(Some("testalias".to_string()))
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
  #[awt]
  async fn test_create_alias_handler(
    #[future] app: Router,
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
    assert_eq!(StatusCode::CREATED, response.status());
    let response = response.json::<AliasResponse>().await?;
    assert_eq!(expected, response);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_alias_handler_non_existent_repo(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[future] app: Router,
  ) -> anyhow::Result<()> {
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
    assert_eq!(StatusCode::NOT_FOUND, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "type": "not_found_error",
          "code": "hub_file_not_found_error",
          "message": "file '\u{2068}fakemodel.Q4_0.gguf\u{2069}' not found in huggingface repo '\u{2068}FakeFactory/not-exists\u{2069}', snapshot '\u{2068}main\u{2069}'"
        }
      }},
      response
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_alias_handler(#[future] app: Router) -> anyhow::Result<()> {
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

    assert_eq!(StatusCode::OK, response.status());
    let updated_alias: AliasResponse = response.json::<AliasResponse>().await?;
    let expected = AliasResponseBuilder::tinyllama_builder()
      .family(Some("tinyllama".to_string()))
      .request_params(
        OAIRequestParamsBuilder::default()
          .temperature(0.8)
          .max_tokens(2000_u16)
          .build()
          .unwrap(),
      )
      .context_params(
        GptContextParamsBuilder::default()
          .n_ctx(4096)
          .build()
          .unwrap(),
      )
      .build()
      .unwrap();
    assert_eq!(expected, updated_alias);
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_update_alias_handler_mismatch(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[future] app: Router,
  ) -> anyhow::Result<()> {
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

    assert_eq!(StatusCode::BAD_REQUEST, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "type": "invalid_request_error",
          "code": "create_alias_error-alias_mismatch",
          "message": "alias in path '\u{2068}llama3:instruct\u{2069}' does not match alias in request '\u{2068}llama3:different\u{2069}'"
        }
      }},
      response
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_create_alias_handler_missing_alias(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[future] app: Router,
  ) -> anyhow::Result<()> {
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

    assert_eq!(StatusCode::BAD_REQUEST, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "type": "invalid_request_error",
          "code": "create_alias_error-alias_not_present",
          "message": "alias is not present in request"
        }
      }},
      response
    );
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
  #[awt]
  #[tokio::test]
  async fn test_create_alias_repo_not_downloaded_error(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[future] app: Router,
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

    assert_eq!(StatusCode::NOT_FOUND, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "error": {
          "type": "not_found_error",
          "code": "hub_file_not_found_error",
          "message": "file '\u{2068}tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf\u{2069}' not found in huggingface repo '\u{2068}TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF\u{2069}', snapshot '\u{2068}main\u{2069}'"
        }
      }},
      response
    );
    Ok(())
  }
}
