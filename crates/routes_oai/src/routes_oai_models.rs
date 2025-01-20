use crate::ENDPOINT_OAI_MODELS;
use async_openai::types::{ListModelResponse, Model};
use axum::{
  extract::{Path, State},
  Json,
};
use objs::{Alias, ApiError, OpenAIApiError};
use server_core::RouterState;
use services::AliasNotFoundError;
use std::sync::Arc;
use utoipa::openapi::ObjectBuilder;

pub struct ListModelResponseWrapper;

impl utoipa::PartialSchema for ListModelResponseWrapper {
  fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{ArrayBuilder, SchemaType, Type};

    ObjectBuilder::new()
      .property(
        "object",
        ObjectBuilder::new().schema_type(SchemaType::new(Type::String)),
      )
      .required("object")
      .property(
        "data",
        ArrayBuilder::new().items(
          ObjectBuilder::new()
            .property(
              "id",
              ObjectBuilder::new()
                .schema_type(SchemaType::new(Type::String))
                .description(Some(
                  "The model identifier, which can be referenced in the API endpoints.",
                )),
            )
            .required("id")
            .property(
              "object",
              ObjectBuilder::new()
                .schema_type(SchemaType::new(Type::String))
                .description(Some("The object type, which is always \"model\".")),
            )
            .required("object")
            .property(
              "created",
              ObjectBuilder::new()
                .schema_type(SchemaType::new(Type::Integer))
                .format(Some(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                  utoipa::openapi::schema::KnownFormat::Int32,
                )))
                .description(Some(
                  "The Unix timestamp (in seconds) when the model was created.",
                ))
                .minimum(Some(0f64)),
            )
            .required("created")
            .property(
              "owned_by",
              ObjectBuilder::new()
                .schema_type(SchemaType::new(Type::String))
                .description(Some("The organization that owns the model.")),
            )
            .required("owned_by")
            .description(Some(
              "Describes an OpenAI model offering that can be used with the API.",
            )),
        ),
      )
      .required("data")
      .into()
  }
}

impl utoipa::ToSchema for ListModelResponseWrapper {}

/// List available models
#[utoipa::path(
    get,
    path = ENDPOINT_OAI_MODELS,
    tag = "openai",
    operation_id = "listModels",
    responses(
        (status = 200, description = "List of available models", 
         body = ListModelResponseWrapper,
         example = json!({
             "object": "list",
             "data": [
                 {
                     "id": "llama2:chat",
                     "object": "model",
                     "created": 1677610602,
                     "owned_by": "bodhi"
                 },
                 {
                     "id": "mistral:instruct",
                     "object": "model",
                     "created": 1677610602,
                     "owned_by": "bodhi"
                 }
             ]
         })),
        (status = 401, description = "Invalid authentication", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Invalid authentication token",
                 "type": "invalid_request_error",
                 "code": "invalid_api_key"
             }
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
      ("bearer_auth" = []),
    ),
)]
pub async fn oai_models_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<Json<ListModelResponse>, ApiError> {
  let models = state
    .app_service()
    .data_service()
    .list_aliases()?
    .into_iter()
    .map(|alias| to_oai_model(state.clone(), alias))
    .collect::<Vec<_>>();
  Ok(Json(ListModelResponse {
    object: "list".to_string(),
    data: models,
  }))
}

pub async fn oai_model_handler(
  State(state): State<Arc<dyn RouterState>>,
  Path(id): Path<String>,
) -> Result<Json<Model>, ApiError> {
  let alias = state
    .app_service()
    .data_service()
    .find_alias(&id)
    .ok_or(AliasNotFoundError(id))?;
  let model = to_oai_model(state, alias);
  Ok(Json(model))
}

fn to_oai_model(state: Arc<dyn RouterState>, alias: Alias) -> Model {
  let bodhi_home = &state.app_service().env_service().bodhi_home();
  let path = bodhi_home.join("aliases").join(alias.config_filename());
  let created = state.app_service().time_service().created_at(&path);
  Model {
    id: alias.alias,
    object: "model".to_string(),
    created,
    owned_by: "system".to_string(),
  }
}

#[cfg(test)]
mod tests {
  use super::{oai_model_handler, oai_models_handler};
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
  };
  use objs::{test_utils::setup_l10n, FluentLocalizationService};
  use pretty_assertions::assert_eq;
  use rstest::{fixture, rstest};
  use serde_json::{json, Value};
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContext};
  use services::test_utils::AppServiceStubBuilder;
  use std::sync::Arc;
  use tower::ServiceExt;

  #[fixture]
  async fn app() -> Router {
    let service = AppServiceStubBuilder::default()
      .with_data_service()
      .build()
      .unwrap();
    let router_state =
      DefaultRouterState::new(Arc::new(MockSharedContext::default()), Arc::new(service));
    Router::new()
      .route("/v1/models", axum::routing::get(oai_models_handler))
      .route("/v1/models/:id", axum::routing::get(oai_model_handler))
      .with_state(Arc::new(router_state))
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_oai_models_handler(#[future] app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(Request::builder().uri("/v1/models").body(Body::empty())?)
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "object": "list",
        "data": [
          {
            "id": "llama3:instruct",
            "object": "model",
            "created": 0,
            "owned_by": "system"
          },
          {
            "id": "testalias-exists:instruct",
            "object": "model",
            "created": 0,
            "owned_by": "system"
          },
          {
            "id": "tinyllama:instruct",
            "object": "model",
            "created": 0,
            "owned_by": "system"
          },
          {
            "id": "FakeFactory/fakemodel-gguf:Q4_0",
            "object": "model",
            "created": 0,
            "owned_by": "system"
          },
          {
            "id": "TheBloke/Llama-2-7B-Chat-GGUF:Q8_0",
            "object": "model",
            "created": 0,
            "owned_by": "system"
          },
          {
            "id": "TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF:Q2_K",
            "object": "model",
            "created": 0,
            "owned_by": "system"
          },
        ]
      }},
      response
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_oai_model_handler(#[future] app: Router) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::builder()
          .uri("/v1/models/llama3:instruct")
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::OK, response.status());
    let response = response.json::<Value>().await?;
    assert_eq!(
      json! {{
        "id": "llama3:instruct",
        "object": "model",
        "created": 0,
        "owned_by": "system",
      }},
      response
    );
    Ok(())
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_oai_model_handler_not_found(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[future] app: Router,
  ) -> anyhow::Result<()> {
    let response = app
      .oneshot(
        Request::builder()
          .uri("/v1/models/non_existent_model")
          .body(Body::empty())?,
      )
      .await?;

    assert_eq!(StatusCode::NOT_FOUND, response.status());
    Ok(())
  }
}
