use crate::{ChatTemplate, GptContextParams, OAIRequestParams, Repo};
use serde::Deserialize;

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Deserialize, PartialEq, Clone, PartialOrd, derive_new::new)]
#[cfg_attr(test, derive(Default))]
pub struct RemoteModel {
  pub alias: String,
  pub family: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: Option<String>,
  pub features: Vec<String>,
  pub chat_template: ChatTemplate,
  #[serde(default)]
  pub request_params: OAIRequestParams,
  #[serde(default)]
  pub context_params: GptContextParams,
}
