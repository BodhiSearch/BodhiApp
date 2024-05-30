use crate::{service::DataServiceError, utils::to_safe_filename};
use derive_new::new;
use once_cell::sync::Lazy;
use prettytable::{Cell, Row};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Display, fs, ops::Deref, path::PathBuf};
use strum::{AsRefStr, Display, EnumIter};
use validator::Validate;

pub static REGEX_REPO: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$").unwrap());
pub static TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";
pub static GGUF_EXTENSION: &str = ".gguf";
pub static REGEX_HF_REPO_FILE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^(?P<hf_cache>.+)/models--(?P<username>[^/]+)--(?P<repo_name>[^/]+)/snapshots/(?P<snapshot>[^/]+)/(?P<filename>.*)$").unwrap()
});

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
  value: String,
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
    let repo = Repo::try_new(s).map_err(|err| serde::de::Error::custom(err.to_string()))?;
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

impl TryFrom<PathBuf> for LocalModelFile {
  type Error = DataServiceError;

  fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
    let path = value.to_string_lossy().into_owned();
    let caps = REGEX_HF_REPO_FILE.captures(&path).ok_or(anyhow::anyhow!(
      "'{path}' does not match huggingface hub cache filepath pattern"
    ))?;
    let size = match fs::metadata(&value) {
      Ok(metadata) => Some(metadata.len()),
      Err(_) => None,
    };
    let repo = Repo::try_new(format!("{}/{}", &caps["username"], &caps["repo_name"]))?;
    Ok(LocalModelFile {
      hf_cache: PathBuf::from(caps["hf_cache"].to_string()),
      repo,
      filename: caps["filename"].to_string(),
      snapshot: caps["snapshot"].to_string(),
      size,
    })
  }
}

impl From<LocalModelFile> for Row {
  fn from(model: LocalModelFile) -> Self {
    let LocalModelFile {
      repo,
      filename,
      snapshot,
      size,
      ..
    } = model;
    let human_size = size
      .map(|size| format!("{:.2} GB", size as f64 / 2_f64.powf(30.0)))
      .unwrap_or_else(|| String::from("Unknown"));
    Row::from(vec![
      Cell::new(&filename),
      Cell::new(&repo),
      Cell::new(&snapshot[..8]),
      Cell::new(&human_size),
    ])
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

impl Alias {
  pub fn config_filename(&self) -> String {
    let filename = self.alias.replace(':', "--");
    let filename = to_safe_filename(&filename);
    format!("{}.yaml", filename)
  }
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
  use std::path::PathBuf;

  use crate::{
    service::HubService,
    test_utils::{hub_service, temp_hf_home, HubServiceTuple},
  };

  use super::{Alias, ChatTemplate, ChatTemplateId, LocalModelFile, RemoteModel, Repo};
  use anyhow_trace::anyhow_trace;
  use prettytable::{Cell, Row};
  use rstest::rstest;
  use tempfile::TempDir;
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

  #[test]
  fn test_local_model_to_row() -> anyhow::Result<()> {
    let model = LocalModelFile::new(
      PathBuf::from("."),
      Repo::try_new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF".to_string())?,
      "Meta-Llama-3-8B-Instruct.Q8_0.gguf".to_string(),
      "1234567890".to_string(),
      Some(1024 * 1024 * 1024 * 10),
    );
    let row = model.into();
    let expected = Row::from(vec![
      Cell::new("Meta-Llama-3-8B-Instruct.Q8_0.gguf"),
      Cell::new("QuantFactory/Meta-Llama-3-8B-Instruct-GGUF"),
      Cell::new("12345678"),
      Cell::new("10.00 GB"),
    ]);
    assert_eq!(expected, row);
    Ok(())
  }

  #[rstest]
  fn test_local_model_file_from_pathbuf(temp_hf_home: TempDir) -> anyhow::Result<()> {
    let hf_cache = temp_hf_home.path().join("huggingface").join("hub");
    let filepath = hf_cache
      .clone()
      .join("models--MyFactory--testalias-neverdownload-gguf")
      .join("snapshots")
      .join("5007652f7a641fe7170e0bad4f63839419bd9213")
      .join("testalias-neverdownload.Q8_0.gguf");
    let local_model: LocalModelFile = filepath.try_into()?;
    let expected = LocalModelFile::new(
      hf_cache,
      Repo::try_new("MyFactory/testalias-neverdownload-gguf".to_string())?,
      "testalias-neverdownload.Q8_0.gguf".to_string(),
      "5007652f7a641fe7170e0bad4f63839419bd9213".to_string(),
      Some(21),
    );
    assert_eq!(expected, local_model);
    Ok(())
  }

  #[rstest]
  #[case("llama3:instruct", "llama3--instruct.yaml")]
  #[case("llama3/instruct", "llama3--instruct.yaml")]
  fn test_alias_config_filename(
    #[case] input: String,
    #[case] expected: String,
  ) -> anyhow::Result<()> {
    let alias = Alias {
      alias: input,
      ..Default::default()
    };
    assert_eq!(expected, alias.config_filename());
    Ok(())
  }
}
