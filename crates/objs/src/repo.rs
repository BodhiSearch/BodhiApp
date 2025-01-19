use crate::ObjValidationError;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Display, str::FromStr};
use utoipa::ToSchema;
use validator::Validate;

pub static TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";
pub static GGUF: &str = "gguf";
pub static GGUF_EXTENSION: &str = ".gguf";
pub static REGEX_VALID_REPO: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_.-]+$").unwrap());

#[derive(Debug, Clone, PartialEq, Default, PartialOrd, Eq, Ord, Hash, Validate, ToSchema)]
pub struct Repo {
  #[validate(regex(
    path = *REGEX_VALID_REPO,
    message = "repo contains invalid characters"
))]
  user: String,
  #[validate(regex(
    path = *REGEX_VALID_REPO,
    message = "repo contains invalid characters",
  ))]
  name: String,
}

impl Repo {
  pub fn new<T: Into<String>>(user: T, repo_name: T) -> Self {
    Self {
      user: user.into(),
      name: repo_name.into(),
    }
  }

  pub fn path(&self) -> String {
    hf_hub::Repo::model(self.to_string()).folder_name()
  }
}

impl FromStr for Repo {
  type Err = ObjValidationError;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let (user, repo_name) = value
      .split_once('/')
      .ok_or_else(|| ObjValidationError::FilePatternMismatch(value.to_string()))?;
    let repo = Repo::new(user, repo_name);
    repo.validate()?;
    Ok(repo)
  }
}

impl TryFrom<String> for Repo {
  type Error = ObjValidationError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Repo::from_str(&value)
  }
}

impl TryFrom<&str> for Repo {
  type Error = ObjValidationError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Repo::from_str(value)
  }
}

impl Display for Repo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}/{}", self.user, self.name)
  }
}

impl<'de> Deserialize<'de> for Repo {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let repo = Repo::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))?;
    Ok(repo)
  }
}

impl Serialize for Repo {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    if self.user.is_empty() || self.name.is_empty() {
      return serializer.serialize_str("");
    }
    serializer.serialize_str(&self.to_string())
  }
}

#[cfg(test)]
mod test {
  use std::str::FromStr;

  use crate::{ObjValidationError, Repo};
  use anyhow_trace::anyhow_trace;
  use rstest::rstest;

  #[rstest]
  #[case("simple/repo")]
  #[case("QuantFactory/Meta-Llama-3-70B-Instruct-GGUF")]
  #[case("Qua-nt.Fac_tory/Meta.Llama-3_70B-Instruct-GGUF")]
  fn test_repo_valid(#[case] input: String) -> anyhow::Result<()> {
    assert!(Repo::from_str(&input).is_ok());
    let repo: Result<Repo, _> = input.parse();
    assert!(repo.is_ok());
    assert_eq!(repo.unwrap().to_string(), input);
    Ok(())
  }

  #[rstest]
  #[case("simplerepo")]
  #[case("simple\\repo")]
  fn test_invalid_repo_format(#[case] input: String) -> anyhow::Result<()> {
    let result = Repo::try_from(input.clone());
    assert!(
      matches!(result, Err(ObjValidationError::FilePatternMismatch(value)) if value == input)
    );
    Ok(())
  }

  #[anyhow_trace]
  #[rstest]
  #[case("$imple/repo", "user")]
  #[case("simp!e/repo", "user")]
  #[case("simple/r$po", "name")]
  #[case("simple/repo/file", "name")]
  fn test_repo_invalid(#[case] input: String, #[case] field: String) -> anyhow::Result<()> {
    let result = Repo::try_from(input.clone());
    assert!(matches!(
      result,
      Err(ObjValidationError::ValidationErrors(_))
    ));
    let err = result.unwrap_err();
    match err {
      ObjValidationError::ValidationErrors(errors) => {
        assert_eq!(1, errors.errors().len());
        assert_eq!(
          format!("{field}: repo contains invalid characters"),
          errors.to_string()
        );
      }
      _ => {
        panic!(
          "Expected ObjValidationError::ValidationErrors, got {:?}",
          err
        );
      }
    }
    Ok(())
  }
}
