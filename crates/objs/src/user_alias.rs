use crate::{is_default, OAIRequestParams, Repo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
  Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, strum::Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum AliasSource {
  #[default]
  User,
  Model,
  #[serde(rename = "api")]
  #[strum(serialize = "api")]
  Api,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, derive_builder::Builder)]
#[builder(setter(into, strip_option), build_fn(error = crate::BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct UserAlias {
  #[builder(setter(skip))]
  pub id: String,
  pub alias: String,
  #[schema(value_type = String, format = "string")]
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
  #[serde(default, skip_serializing_if = "is_default")]
  #[builder(default)]
  pub request_params: OAIRequestParams,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  #[builder(default)]
  pub context_params: Vec<String>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub updated_at: DateTime<Utc>,
}

impl UserAliasBuilder {
  pub fn build_with_time(&self, now: DateTime<Utc>) -> Result<UserAlias, crate::BuilderError> {
    Ok(UserAlias {
      id: uuid::Uuid::new_v4().to_string(),
      alias: self
        .alias
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("alias"))?,
      repo: self
        .repo
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("repo"))?,
      filename: self
        .filename
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("filename"))?,
      snapshot: self
        .snapshot
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("snapshot"))?,
      request_params: self.request_params.clone().unwrap_or_default(),
      context_params: self.context_params.clone().unwrap_or_default(),
      created_at: now,
      updated_at: now,
    })
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn build_test(&self) -> Result<UserAlias, crate::BuilderError> {
    use chrono::TimeZone;
    let fixed_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    Ok(UserAlias {
      id: format!("test-{}", self.alias.clone().unwrap_or_default()),
      alias: self
        .alias
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("alias"))?,
      repo: self
        .repo
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("repo"))?,
      filename: self
        .filename
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("filename"))?,
      snapshot: self
        .snapshot
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("snapshot"))?,
      request_params: self.request_params.clone().unwrap_or_default(),
      context_params: self.context_params.clone().unwrap_or_default(),
      created_at: fixed_time,
      updated_at: fixed_time,
    })
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn build_with_id(&self, id: &str, now: DateTime<Utc>) -> UserAlias {
    UserAlias {
      id: id.to_string(),
      alias: self.alias.clone().unwrap_or_default(),
      repo: self.repo.clone().unwrap_or_default(),
      filename: self.filename.clone().unwrap_or_default(),
      snapshot: self.snapshot.clone().unwrap_or_default(),
      request_params: self.request_params.clone().unwrap_or_default(),
      context_params: self.context_params.clone().unwrap_or_default(),
      created_at: now,
      updated_at: now,
    }
  }
}

impl std::fmt::Display for UserAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Alias {{ alias: {}, repo: {}, filename: {}, snapshot: {} }}",
      self.alias, self.repo, self.filename, self.snapshot
    )
  }
}

#[cfg(test)]
mod test {
  use crate::test_utils::AliasBuilder;
  use crate::{Repo, UserAlias};

  #[test]
  fn test_alias_display() {
    let alias = UserAlias {
      alias: "test:alias".to_string(),
      repo: Repo::try_from("test/repo").unwrap(),
      filename: "test.gguf".to_string(),
      snapshot: "main".to_string(),
      ..Default::default()
    };
    assert_eq!(
      format!("{}", alias),
      "Alias { alias: test:alias, repo: test/repo, filename: test.gguf, snapshot: main }"
    );
  }

  #[test]
  fn test_alias_derive_builder() {
    let alias = AliasBuilder::tinyllama().build_test().unwrap();
    assert_eq!("tinyllama:instruct", alias.alias);
    assert_eq!("test-tinyllama:instruct", alias.id);
  }
}
