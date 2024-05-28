use derive_new::new;
use once_cell::sync::Lazy;
use prettytable::{Cell, Row};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
  fmt::Display,
  ops::Deref, path::PathBuf,
};
use strum::{AsRefStr, Display, EnumIter};
use validator::Validate;

pub static REGEX_REPO: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$").unwrap());
pub static TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";

#[derive(
  clap::ValueEnum, Clone, Debug, Serialize, Deserialize, PartialEq, EnumIter, AsRefStr, Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ChatTemplateId {
  Llama3,
  Llama2,
  Llama2Legacy,
  Phi3,
  Gemma,
  Deepseek,
  CommandR,
  Openchat,
}

impl PartialOrd for ChatTemplateId {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.as_ref().partial_cmp(other.as_ref())
  }
}

#[derive(Debug, Clone, PartialEq, Validate, Default, PartialOrd, Eq, Ord)]
pub struct Repo {
  #[validate(regex(path = *REGEX_REPO, message = "does not match huggingface repo pattern 'owner/repo'"))]
  pub value: String,
}

impl Repo {
  pub fn try_new(value: String) -> crate::service::Result<Self> {
    let repo = Repo { value };
    repo.validate()?;
    Ok(repo)
  }

  pub fn path(&self) -> String {
    format!("models--{}--{}", self.owner(), self.name())
  }

  pub fn split(&self) -> (String, String) {
    match self.value.split_once('/') {
      Some((owner, repo)) => (owner.to_string(), repo.to_string()),
      None => unreachable!(
        "should not be able to create Repo with invalid repo format, value is '{}'",
        self.value
      ),
    }
  }

  pub fn owner(&self) -> String {
    let (owner, _) = self.split();
    owner
  }

  pub fn name(&self) -> String {
    let (_, name) = self.split();
    name
  }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
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

// #[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, new)]
// pub struct LocalModel {
//   pub repo: Repo,
//   pub files: Vec<LocalModelFile>,
// }

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, new)]
pub struct LocalModelFile {
  pub hf_cache: PathBuf,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  pub size: Option<u64>,
}

impl LocalModelFile {
  pub fn path(&self) -> PathBuf {
    let mut path = self.hf_cache.clone();
    path.push(self.repo.path());
    path.push("snapshots");
    path.push(&self.snapshot);
    path.push(&self.filename);
    path
  }
}

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Deserialize, PartialEq, Clone, PartialOrd, new)]
#[cfg_attr(test, derive(Default))]
pub struct RemoteModel {
  pub(super) alias: String,
  pub(super) family: String,
  pub(super) repo: Repo,
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
#[cfg_attr(test, derive(Default))]
pub struct Alias {
  pub alias: String,
  pub family: Option<String>,
  pub repo: Repo,
  pub filename: String,
  pub snapshot: Option<String>,
  pub features: Vec<String>,
  pub chat_template: ChatTemplate,
}

impl From<RemoteModel> for Alias {
  fn from(value: RemoteModel) -> Self {
    Alias::new(
      value.alias,
      Some(value.family),
      value.repo,
      value.filename,
      None,
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
      Cell::new(&value.repo.to_string()),
      Cell::new(&value.filename),
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
  use super::{ChatTemplate, ChatTemplateId, RemoteModel, Repo};
  use anyhow_trace::anyhow_trace;
  use prettytable::{Cell, Row};
  use rstest::rstest;
  use validator::Validate;

  #[rstest]
  #[case("simple/repo")]
  #[case("QuantFactory/Meta-Llama-3-70B-Instruct-GGUF")]
  #[case("Qua-nt.Fac_tory/Meta.Llama-3_70B-Instruct-GGUF")]
  fn test_repo_valid(#[case] repo: String) -> anyhow::Result<()> {
    Repo::try_new(repo)?.validate()?;
    Ok(())
  }

  #[rstest]
  #[case("simplerepo")]
  #[case("simple\\repo")]
  #[case("$imple/repo")]
  #[case("simp!e/repo")]
  #[case("simple/repo/file")]
  #[anyhow_trace]
  fn test_repo_invalid(#[case] repo: String) -> anyhow::Result<()> {
    let result = Repo::try_new(repo);
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
      Repo::try_new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string())?,
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

  #[rstest]
  fn test_chat_template_id_partial_ord() -> anyhow::Result<()> {
    assert!(ChatTemplateId::Llama3.gt(&ChatTemplateId::Llama2));
    assert!(ChatTemplateId::Openchat.gt(&ChatTemplateId::CommandR));
    Ok(())
  }
}
