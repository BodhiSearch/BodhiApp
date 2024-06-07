use super::{is_default, ChatTemplate, GptContextParams, OAIRequestParams, Repo};
use crate::utils::to_safe_filename;
use derive_new::new;
use prettytable::{Cell, Row};
use serde::{Deserialize, Serialize};

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, new)]
#[cfg_attr(test, derive(Default, derive_builder::Builder))]
pub struct Alias {
  pub alias: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub family: Option<String>,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  pub features: Vec<String>,
  pub chat_template: ChatTemplate,
  #[serde(default, skip_serializing_if = "is_default")]
  pub request_params: OAIRequestParams,
  #[serde(default, skip_serializing_if = "is_default")]
  pub context_params: GptContextParams,
}

impl Alias {
  pub fn config_filename(&self) -> String {
    let filename = self.alias.replace(':', "--");
    let filename = to_safe_filename(&filename);
    format!("{}.yaml", filename)
  }
}

impl From<Alias> for Row {
  fn from(value: Alias) -> Self {
    Row::from(vec![
      Cell::new(&value.alias),
      Cell::new(&value.family.unwrap_or_default()),
      Cell::new(&value.repo.to_string()),
      Cell::new(&value.filename),
      Cell::new(&value.features.join(",")),
      Cell::new(&value.chat_template.to_string()),
    ])
  }
}

// TODO: hard coding for time being
pub fn default_features() -> Vec<String> {
  vec!["chat".to_string()]
}

#[cfg(test)]
mod test {
  use super::Alias;
  use rstest::rstest;

  #[rstest]
  #[case("llama3:instruct", "llama3--instruct.yaml")]
  #[case("llama3/instruct", "llama3--instruct.yaml")]
  fn test_alias_config_filename(#[case] input: String, #[case] expected: String) {
    let alias = Alias {
      alias: input,
      ..Default::default()
    };
    assert_eq!(expected, alias.config_filename());
  }

  #[rstest]
  #[case(
    Alias::default(),
    r#"alias: ''
repo: ''
filename: ''
snapshot: ''
features: []
chat_template: llama3
"#
  )]
  fn test_alias_serialize(#[case] alias: Alias, #[case] expected: String) -> anyhow::Result<()> {
    let actual = serde_yaml::to_string(&alias)?;
    assert_eq!(expected, actual);
    Ok(())
  }
}
