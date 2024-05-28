use std::{fmt::Display, ops::Deref};

use crate::cli::ChatTemplateId;
use derive_new::new;
use once_cell::sync::Lazy;
use prettytable::{Cell, Row};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use validator::Validate;

pub static REGEX_REPO: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$").unwrap());
pub static TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";

#[derive(Debug, Clone, PartialEq, Validate, Default, new)]
pub struct Repo {
  #[validate(regex(path = *REGEX_REPO, message = "does not match huggingface repo pattern 'owner/repo'"))]
  pub value: String,
}

impl<'de> Deserialize<'de> for Repo {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let repo = Repo { value: s };
    repo
      .validate()
      .map_err(|e| serde::de::Error::custom(e.to_string()))?;
    Ok(repo)
  }
}

impl Serialize for Repo {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.value)
  }
}

impl Deref for Repo {
  type Target = String;

  fn deref(&self) -> &Self::Target {
    &self.value
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ChatTemplate {
  Id(ChatTemplateId),
  Repo(Repo),
}

impl Display for ChatTemplate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ChatTemplate::Id(id) => write!(f, "{}", id),
      ChatTemplate::Repo(repo) => write!(f, "{}", repo.value),
    }
  }
}

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Deserialize, PartialEq, Clone, new)]
pub struct RemoteModel {
  pub(super) alias: String,
  pub(super) family: String,
  pub(super) repo: String,
  pub(super) filename: String,
  pub(super) features: Vec<String>,
  pub(super) chat_template: ChatTemplate,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, new)]
pub struct Alias {
  pub alias: String,
  pub family: Option<String>,
  pub repo: Option<Repo>,
  pub filename: Option<String>,
  pub features: Vec<String>,
  pub chat_template: ChatTemplate,
}

impl From<RemoteModel> for Alias {
  fn from(value: RemoteModel) -> Self {
    Alias::new(
      value.alias,
      Some(value.family),
      Some(Repo::new(value.repo)),
      Some(value.filename),
      value.features,
      value.chat_template,
    )
  }
}

impl From<Alias> for Row {
  fn from(value: Alias) -> Self {
    Row::from(vec![
      Cell::new(&value.alias),
      Cell::new(&value.family.unwrap_or_default()),
      Cell::new(&value.repo.unwrap_or_default()),
      Cell::new(&value.filename.unwrap_or_default()),
      Cell::new(&value.features.join(",")),
      Cell::new(&value.chat_template.to_string()),
    ])
  }
}

pub fn default_features() -> Vec<String> {
  vec!["chat".to_string()]
}

#[cfg(test)]
mod test {
  use super::{RemoteModel, Repo};
  use crate::{cli::ChatTemplateId, objs::ChatTemplate};
  use prettytable::{Cell, Row};
  use rstest::rstest;
  use validator::Validate;

  #[rstest]
  #[case("simple/repo")]
  #[case("QuantFactory/Meta-Llama-3-70B-Instruct-GGUF")]
  #[case("Qua-nt.Fac_tory/Meta.Llama-3_70B-Instruct-GGUF")]
  fn test_repo_valid(#[case] repo: String) -> anyhow::Result<()> {
    Repo::new(repo).validate()?;
    Ok(())
  }

  #[rstest]
  #[case("simplerepo")]
  #[case("simple\\repo")]
  #[case("$imple/repo")]
  #[case("simp!e/repo")]
  #[case("simple/repo/file")]
  fn test_repo_invalid(#[case] repo: String) -> anyhow::Result<()> {
    let result = Repo::new(repo).validate();
    assert!(result.is_err());
    assert_eq!(
      "value: does not match huggingface repo pattern 'owner/repo'",
      result.unwrap_err().to_string()
    );
    Ok(())
  }

  #[test]
  fn test_list_remote_model_to_row() -> anyhow::Result<()> {
    let model = RemoteModel::new(
      "llama3:instruct".to_string(),
      "llama3".to_string(),
      "QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string(),
      "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      vec!["chat".to_string()],
      ChatTemplate::Id(ChatTemplateId::Llama3),
    );
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
