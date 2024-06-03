use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Display, ops::Deref};
use validator::Validate;

use super::ObjError;

pub static TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";
pub static GGUF_EXTENSION: &str = ".gguf";
pub static REFS: &str = "refs";
pub static REFS_MAIN: &str = "refs/main";
pub static REGEX_REPO: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$").unwrap());

#[derive(Debug, Clone, PartialEq, Validate, Default, PartialOrd, Eq, Ord)]
pub struct Repo {
  #[validate(regex(path = *REGEX_REPO, message = "does not match huggingface repo pattern 'owner/repo'"))]
  value: String,
}

impl TryFrom<String> for Repo {
  type Error = ObjError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    let repo = Repo { value };
    repo.validate()?;
    Ok(repo)
  }
}

impl TryFrom<&str> for Repo {
  type Error = ObjError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let repo = Repo {
      value: String::from(value),
    };
    repo.validate()?;
    Ok(repo)
  }
}

impl Repo {
  pub fn path(&self) -> String {
    hf_hub::Repo::model(self.value.clone()).folder_name()
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

impl Display for Repo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.value.fmt(f)
  }
}

impl<'de> Deserialize<'de> for Repo {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let repo = Repo::try_from(s).map_err(|err| serde::de::Error::custom(err.to_string()))?;
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

#[cfg(test)]
mod test {
  use super::Repo;
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;
  use validator::Validate;

  #[rstest]
  #[case("simple/repo")]
  #[case("QuantFactory/Meta-Llama-3-70B-Instruct-GGUF")]
  #[case("Qua-nt.Fac_tory/Meta.Llama-3_70B-Instruct-GGUF")]
  fn test_repo_valid(#[case] repo: String) -> anyhow::Result<()> {
    Repo::try_from(repo)?.validate()?;
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
    let result = Repo::try_from(repo);
    assert!(result.is_err());
    assert_eq!(
      "value: does not match huggingface repo pattern 'owner/repo'",
      result.unwrap_err().to_string()
    );
    Ok(())
  }
}
