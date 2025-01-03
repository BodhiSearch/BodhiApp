use crate::{ChatTemplateType, GptContextParams, OAIRequestParams, Repo};
use serde::Deserialize;

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Deserialize, PartialEq, Clone, PartialOrd, derive_new::new)]
#[cfg_attr(test, derive(Default))]
pub struct RemoteModel {
  pub alias: String,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: Option<String>,
  pub chat_template: ChatTemplateType,
  #[serde(default)]
  pub request_params: OAIRequestParams,
  #[serde(default)]
  pub context_params: GptContextParams,
}
