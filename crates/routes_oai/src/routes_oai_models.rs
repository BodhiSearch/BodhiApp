use async_openai::types::{ListModelResponse, Model};
use axum::{
  extract::{Path, State},
  Json,
};
use objs::{Alias, ApiError};
use server_core::RouterState;
use services::AliasNotFoundError;
use std::sync::Arc;

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
  use rstest::{fixture, rstest};
  use serde_json::{json, Value};
  use server_core::{test_utils::ResponseTestExt, DefaultRouterState, MockSharedContextRw};
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
      DefaultRouterState::new(Arc::new(MockSharedContextRw::default()), Arc::new(service));
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

    assert_eq!(response.status(), StatusCode::OK);
    let response = response.json::<Value>().await?;
    assert_eq!(
      response,
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
          }
        ]
      }}
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

    assert_eq!(response.status(), StatusCode::OK);
    let response = response.json::<Value>().await?;
    assert_eq!(
      response,
      json! {{
        "id": "llama3:instruct",
        "object": "model",
        "created": 0,
        "owned_by": "system",
      }}
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

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    Ok(())
  }
}
