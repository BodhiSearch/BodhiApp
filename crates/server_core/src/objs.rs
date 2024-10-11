use objs::{Alias, GptContextParams, HubFile, OAIRequestParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct PaginationSortParams {
  pub page: Option<usize>,
  pub page_size: Option<usize>,
  pub sort: Option<String>,
  pub sort_order: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
  pub data: Vec<T>,
  pub total: usize,
  pub page: usize,
  pub page_size: usize,
}

#[allow(clippy::too_many_arguments)]
#[derive(Serialize, Deserialize, Debug, PartialEq, derive_new::new)]
#[cfg_attr(any(test, feature = "test-utils"), derive(derive_builder::Builder))]
#[cfg_attr(
  any(test, feature = "test-utils"),
  builder(
    setter(into),
    build_fn(error = objs::BuilderError)))]
pub struct AliasResponse {
  pub alias: String,
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  pub family: Option<String>,
  pub features: Vec<String>,
  pub chat_template: String,
  pub model_params: HashMap<String, Value>,
  pub request_params: OAIRequestParams,
  pub context_params: GptContextParams,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LocalModelResponse {
  pub repo: String,
  pub filename: String,
  pub snapshot: String,
  pub size: Option<u64>,
  pub model_params: HashMap<String, Value>,
}

impl From<HubFile> for LocalModelResponse {
  fn from(model: HubFile) -> Self {
    LocalModelResponse {
      repo: model.repo.to_string(),
      filename: model.filename,
      snapshot: model.snapshot,
      size: model.size,
      model_params: HashMap::new(),
    }
  }
}

impl From<Alias> for AliasResponse {
  fn from(alias: Alias) -> Self {
    AliasResponse {
      alias: alias.alias,
      family: alias.family,
      repo: alias.repo.to_string(),
      filename: alias.filename,
      snapshot: alias.snapshot,
      features: alias.features,
      chat_template: alias.chat_template.to_string(),
      model_params: HashMap::new(),
      request_params: alias.request_params,
      context_params: alias.context_params,
    }
  }
}
