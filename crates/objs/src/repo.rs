use crate::ObjValidationError;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Display, ops::Deref, str::FromStr};
use validator::Validate;

pub static TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";
pub static GGUF: &str = "gguf";
pub static GGUF_EXTENSION: &str = ".gguf";
pub static REGEX_REPO: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$").unwrap());

#[derive(Debug, Clone, PartialEq, Validate, Default, PartialOrd, Eq, Ord, Hash)]
pub struct Repo {
  #[validate(regex(
        path = *REGEX_REPO,
        message = "does not match the huggingface repo pattern 'username/repo'"
    ))]
  value: String,
}

impl TryFrom<String> for Repo {
  type Error = ObjValidationError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    let repo = Repo { value };
    repo.validate()?;
    Ok(repo)
  }
}

impl TryFrom<&str> for Repo {
  type Error = ObjValidationError;

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

impl FromStr for Repo {
  type Err = ObjValidationError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Repo::try_from(s)
  }
}

#[cfg(test)]
mod test {
  use crate::{ObjValidationError, Repo};
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;
  use validator::Validate;

  #[rstest]
  #[case("simple/repo")]
  #[case("QuantFactory/Meta-Llama-3-70B-Instruct-GGUF")]
  #[case("Qua-nt.Fac_tory/Meta.Llama-3_70B-Instruct-GGUF")]
  fn test_repo_valid(#[case] input: String) -> anyhow::Result<()> {
    Repo::try_from(input.clone())?.validate()?;

    let repo: Result<Repo, _> = input.parse();
    assert!(repo.is_ok());
    assert_eq!(repo.unwrap().to_string(), input);
    Ok(())
  }

  #[rstest]
  #[case("simplerepo")]
  #[case("simple\\repo")]
  #[case("$imple/repo")]
  #[case("simp!e/repo")]
  #[case("simple/repo/file")]
  #[anyhow_trace]
  fn test_repo_invalid(#[case] input: String) -> anyhow::Result<()> {
    let result = Repo::try_from(input.clone());
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
      ObjValidationError::ValidationErrors(errors) => {
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
          errors.to_string(),
          "value: does not match the huggingface repo pattern 'username/repo'"
        );
      }
      _ => {
        panic!(
          "Expected ObjValidationError::ValidationErrors, got {:?}",
          err
        );
      }
    }
    let repo: Result<Repo, _> = input.parse();
    assert!(repo.is_err());
    Ok(())
  }
}
