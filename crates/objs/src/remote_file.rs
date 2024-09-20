use crate::{
  chat_template::ChatTemplate, gpt_params::GptContextParams, oai::OAIRequestParams, repo::Repo,
};
use derive_new::new;
use prettytable::Row;
use serde::Deserialize;

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Deserialize, PartialEq, Clone, PartialOrd, new)]
#[cfg_attr(test, derive(Default))]
pub struct RemoteModel {
  pub alias: String,
  pub family: String,
  pub repo: Repo,
  pub filename: String,
  pub features: Vec<String>,
  pub chat_template: ChatTemplate,
  #[serde(default)]
  pub request_params: OAIRequestParams,
  #[serde(default)]
  pub context_params: GptContextParams,
}

impl From<RemoteModel> for Row {
  fn from(model: RemoteModel) -> Self {
    Row::from(vec![
      &model.alias,
      &model.family,
      &model.repo,
      &model.filename,
      &model.features.join(","),
      &model.chat_template.to_string(),
    ])
  }
}

#[cfg(test)]
mod test {
  use super::RemoteModel;
  use prettytable::{Cell, Row};
  use rstest::rstest;

  #[rstest]
  fn test_list_remote_model_to_row() -> anyhow::Result<()> {
    let model = RemoteModel::llama3();
    let row: Row = model.into();
    let expected = Row::from(vec![
      Cell::new("llama3:instruct"),
      Cell::new("llama3"),
      Cell::new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"),
      Cell::new("Meta-Llama-3-8B-Instruct.Q8_0.gguf"),
      Cell::new("chat"),
      Cell::new("llama3"),
    ]);
    assert_eq!(expected, row);
    Ok(())
  }
}
