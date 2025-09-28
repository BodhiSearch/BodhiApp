use crate::{UserAliasResponse, ENDPOINT_MODELS};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::WithRejection;
use commands::{CreateCommand, CreateCommandError};
use objs::{ApiError, AppError, ErrorType, OAIRequestParams, Repo, API_TAG_MODELS};
use serde::{Deserialize, Serialize};
use server_core::RouterState;
use services::AliasNotFoundError;
use std::sync::Arc;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateAliasRequest {
  alias: String,
  repo: String,
  filename: String,
  snapshot: Option<String>,

  request_params: Option<OAIRequestParams>,
  context_params: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateAliasRequest {
  repo: String,
  filename: String,
  snapshot: Option<String>,

  request_params: Option<OAIRequestParams>,
  context_params: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum CreateAliasError {
  #[error("alias_not_present")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AliasNotPresent,
  #[error(transparent)]
  AliasNotFound(#[from] AliasNotFoundError),
  #[error(transparent)]
  CreateCommand(#[from] CreateCommandError),
  #[error("alias_mismatch")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AliasMismatch { path: String, request: String },
}

/// Create Alias
#[utoipa::path(
    post,
    path = ENDPOINT_MODELS,
    tag = API_TAG_MODELS,
    operation_id = "createAlias",
    request_body = CreateAliasRequest,
    responses(
      (status = 201, description = "Alias created succesfully", body = UserAliasResponse),
    ),
    security(
      ("bearer_auth" = []),
    ),
)]
pub async fn create_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  WithRejection(Json(payload), _): WithRejection<Json<CreateAliasRequest>, ApiError>,
) -> Result<(StatusCode, Json<UserAliasResponse>), ApiError> {
  let command = CreateCommand::new(
    &payload.alias,
    Repo::try_from(payload.repo)?,
    &payload.filename,
    payload.snapshot,
    false,
    false,
    payload.request_params.unwrap_or_default(),
    payload.context_params.unwrap_or_default(),
  );
  command.execute(state.app_service()).await?;
  let alias = state
    .app_service()
    .data_service()
    .find_user_alias(&payload.alias)
    .ok_or(AliasNotFoundError(payload.alias))?;
  Ok((StatusCode::CREATED, Json(UserAliasResponse::from(alias))))
}

/// Update Alias
#[utoipa::path(
    put,
    path = ENDPOINT_MODELS.to_owned() + "/{id}",
    tag = API_TAG_MODELS,
    params(
        ("id" = String, Path, description = "Alias identifier",
         example = "llama--3")
    ),
    operation_id = "updateAlias",
    request_body = UpdateAliasRequest,
    responses(
      (status = 200, description = "Alias updated succesfully", body = UserAliasResponse),
    ),
    security(
      ("bearer_auth" = []),
    ),
)]
pub async fn update_alias_handler(
  State(state): State<Arc<dyn RouterState>>,
  axum::extract::Path(id): axum::extract::Path<String>,
  WithRejection(Json(payload), _): WithRejection<Json<UpdateAliasRequest>, ApiError>,
) -> Result<(StatusCode, Json<UserAliasResponse>), ApiError> {
  let command = CreateCommand::new(
    &id,
    Repo::try_from(payload.repo)?,
    payload.filename,
    payload.snapshot,
    false,
    true,
    payload.request_params.unwrap_or_default(),
    payload.context_params.unwrap_or_default(),
  );
  command.execute(state.app_service()).await?;
  let alias = state
    .app_service()
    .data_service()
    .find_user_alias(&id)
    .ok_or(AliasNotFoundError(id))?;
  Ok((StatusCode::OK, Json(UserAliasResponse::from(alias))))
}

#[cfg(test)]
mod tests {
  use crate::{
    create_alias_handler, update_alias_handler, UserAliasResponse, UserAliasResponseBuilder,
  };
  use axum::{
    body::Body,
    http::{status::StatusCode, Method, Request},
    routing::{post, put},
    Router,
  };
  use objs::{test_utils::setup_l10n, FluentLocalizationService, OAIRequestParamsBuilder};
  use pretty_assertions::assert_eq;
  use rstest::{fixture, rstest};
  use serde_json::{json, Value};
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
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
      .route("/api/models/{id}", put(update_alias_handler))
      .with_state(Arc::new(router_state))
  }

  fn payload() -> Value {
    serde_json::json!({
      "alias": "testalias:instruct",
      "repo": "MyFactory/testalias-gguf",
      "filename": "testalias.Q8_0.gguf",

      "family": "testalias",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": [
        "--ctx-size 2048"
      ]
    })
  }

  fn expected() -> UserAliasResponse {
    UserAliasResponseBuilder::default()
      .alias("testalias:instruct".to_string())
      .repo("MyFactory/testalias-gguf")
      .filename("testalias.Q8_0.gguf")
      .snapshot("5007652f7a641fe7170e0bad4f63839419bd9213")
      .source("user")
      .model_params(HashMap::new())
      .request_params(
        OAIRequestParamsBuilder::default()
          .temperature(0.7)
          .build()
          .unwrap(),
      )
      .context_params(vec!["--ctx-size 2048".to_string()])
      .build()
      .unwrap()
  }

  fn payload_with_snapshot() -> Value {
    serde_json::json!({
      "alias": "testalias:instruct",
      "repo": "MyFactory/testalias-gguf",
      "filename": "testalias.Q8_0.gguf",
      "snapshot": "5007652f7a641fe7170e0bad4f63839419bd9213",

      "family": "testalias",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": [
        "--ctx-size 2048"
      ]
    })
  }

  fn expected_with_snapshot() -> UserAliasResponse {
    UserAliasResponseBuilder::default()
      .alias("testalias:instruct".to_string())
      .repo("MyFactory/testalias-gguf")
      .filename("testalias.Q8_0.gguf")
      .snapshot("5007652f7a641fe7170e0bad4f63839419bd9213")
      .source("user")
      .model_params(HashMap::new())
      .request_params(
        OAIRequestParamsBuilder::default()
          .temperature(0.7)
          .build()
          .unwrap(),
      )
      .context_params(vec!["--ctx-size 2048".to_string()])
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
    #[case] expected: UserAliasResponse,
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
    let response = response.json::<UserAliasResponse>().await?;
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

      "family": "test_family",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": [
        "--ctx-size 2048"
      ]
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

      "family": "tinyllama",
      "request_params": {
        "temperature": 0.8,
        "max_tokens": 2000
      },
      "context_params": [
        "--ctx-size 4096"
      ]
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
    let updated_alias: UserAliasResponse = response.json::<UserAliasResponse>().await?;
    let expected = UserAliasResponseBuilder::tinyllama_builder()
      .request_params(
        OAIRequestParamsBuilder::default()
          .temperature(0.8)
          .max_tokens(2000_u16)
          .build()
          .unwrap(),
      )
      .context_params(vec!["--ctx-size 4096".to_string()])
      .build()
      .unwrap();
    assert_eq!(expected, updated_alias);
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

      "family": "test_family",
      "request_params": {
        "temperature": 0.7
      },
      "context_params": [
        "--ctx-size 2048"
      ]
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
          "code": "json_rejection_error",
          "message": "failed to parse the request body as JSON, error: \u{2068}Failed to deserialize the JSON body into the target type: missing field `alias` at line 1 column 167\u{2069}"
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

    "family": "tinyllama",
    "request_params": {
      "temperature": 0.8,
      "max_tokens": 2000
    },
    "context_params": [
      "--ctx-size 4096"
    ]
  }), Method::POST, "/api/models")]
  #[case(serde_json::json!({
    "alias": "tinyllama:instruct",
    "repo": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF",
    "filename": "tinyllama-1.1b-chat-v0.3.Q4_K_S.gguf",

    "family": "tinyllama",
    "request_params": {
      "temperature": 0.8,
      "max_tokens": 2000
    },
    "context_params": [
      "--ctx-size 4096"
    ]
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
