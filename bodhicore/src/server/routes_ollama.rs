use super::RouterStateFn;
use crate::objs::{Alias, GGUF};
use axum::{extract::State, Json};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize, Serializer};
use std::{fs, sync::Arc, time::UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub(crate) struct ModelsResponse {
  models: Vec<Model>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct Model {
  model: String,
  #[serde(serialize_with = "serialize_datetime")]
  modified_at: u32,
  size: i64,
  digest: String,
  details: ModelDetails,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct ModelDetails {
  parent_model: Option<String>,
  format: String,
  family: String,
  families: Option<Vec<String>>,
  parameter_size: String,
  quantization_level: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OllamaError {
  error: String,
}

pub(crate) async fn ollama_models_handler(
  State(state): State<Arc<dyn RouterStateFn>>,
) -> Result<Json<ModelsResponse>, Json<OllamaError>> {
  let models = state
    .app_service()
    .data_service()
    .list_aliases()
    .map_err(|err| {
      Json(OllamaError {
        error: err.to_string(),
      })
    })?
    .into_iter()
    .map(|alias| to_ollama_model(state.clone(), alias))
    .collect::<Vec<_>>();
  Ok(Json(ModelsResponse { models }))
}

fn to_ollama_model(state: Arc<dyn RouterStateFn>, alias: Alias) -> Model {
  let bodhi_home = &state.app_service().env_service().bodhi_home();
  let path = bodhi_home.join("aliases").join(alias.config_filename());
  let created = fs::metadata(path)
    .map_err(|e| e.to_string())
    .and_then(|m| m.created().map_err(|e| e.to_string()))
    .and_then(|t| t.duration_since(UNIX_EPOCH).map_err(|e| e.to_string()))
    .unwrap_or_default()
    .as_secs() as u32;
  Model {
    model: alias.alias,
    modified_at: created,
    size: 0,
    digest: alias.snapshot,
    details: ModelDetails {
      parent_model: None,
      format: GGUF.to_string(),
      family: alias.family.unwrap_or_else(|| "unknown".to_string()),
      families: None,
      // TODO: have alias contain parameter size and quantizaiton level
      parameter_size: "".to_string(),
      quantization_level: "".to_string(),
    },
  }
}

fn serialize_datetime<S>(timestamp: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let datetime = Utc
    .timestamp_opt(*timestamp as i64, 0)
    .single()
    .unwrap_or_default();
  let formatted = datetime.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
  serializer.serialize_str(&formatted)
}

#[cfg(test)]
mod test {
  use super::ollama_models_handler;
  use crate::{
    test_utils::app_service_stub,
    test_utils::{AppServiceTuple, MockRouterState, ResponseTestExt},
  };
  use axum::{body::Body, http::Request, routing::get, Router};
  use rstest::rstest;
  use serde_json::Value;
  use std::sync::Arc;
  use tower::ServiceExt;
  use validator::ValidateLength;

  #[rstest]
  #[tokio::test]
  async fn test_ollama_routes_models_list(app_service_stub: AppServiceTuple) -> anyhow::Result<()> {
    let AppServiceTuple(_bodhi_home, _hf_home, _, _, service) = app_service_stub;
    let service = Arc::new(service);
    let mut router_state = MockRouterState::new();
    router_state
      .expect_app_service()
      .returning(move || service.clone());
    let app = Router::new()
      .route("/api/tags", get(ollama_models_handler))
      .with_state(Arc::new(router_state));
    let response = app
      .oneshot(Request::get("/api/tags").body(Body::empty()).unwrap())
      .await?
      .json::<Value>()
      .await?;
    assert_eq!(3, response["models"].as_array().length().unwrap());
    let llama3 = response["models"]
      .as_array()
      .unwrap()
      .iter()
      .find(|item| item["model"] == "llama3:instruct")
      .unwrap();
    assert_eq!(llama3["digest"], "5007652f7a641fe7170e0bad4f63839419bd9213");
    Ok(())
  }
}
