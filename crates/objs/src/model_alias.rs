use crate::Repo;
use derive_new::new;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
  Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, derive_builder::Builder, new,
)]
#[builder(
  setter(into, strip_option),
  build_fn(error = crate::BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct ModelAlias {
  pub alias: String,
  #[schema(value_type = String, format = "string")]
  pub repo: Repo,
  pub filename: String,
  pub snapshot: String,
}

impl ModelAlias {
  pub fn config_filename(&self) -> String {
    let filename = self.alias.replace(':', "--");
    crate::to_safe_filename(&filename)
  }
}

impl std::fmt::Display for ModelAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ModelAlias {{ alias: {}, repo: {}, filename: {}, snapshot: {} }}",
      self.alias, self.repo, self.filename, self.snapshot
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::{ModelAlias, ModelAliasBuilder, Repo};
  use std::str::FromStr;

  #[test]
  fn test_model_alias_builder() {
    let alias = ModelAliasBuilder::default()
      .alias("llama3:instruct")
      .repo(Repo::from_str("test/llama3").unwrap())
      .filename("llama3.gguf")
      .snapshot("main")
      .build()
      .unwrap();

    assert_eq!(alias.alias, "llama3:instruct");
    assert_eq!(alias.repo.to_string(), "test/llama3");
    assert_eq!(alias.filename, "llama3.gguf");
    assert_eq!(alias.snapshot, "main");
  }

  #[test]
  fn test_model_alias_display() {
    let alias = ModelAlias::new(
      "test:model".to_string(),
      Repo::from_str("owner/repo").unwrap(),
      "model.gguf".to_string(),
      "main".to_string(),
    );

    let display = format!("{}", alias);
    assert_eq!(
      display,
      "ModelAlias { alias: test:model, repo: owner/repo, filename: model.gguf, snapshot: main }"
    );
  }

  #[test]
  fn test_model_alias_config_filename() {
    let alias = ModelAlias::new(
      "test:model".to_string(),
      Repo::from_str("owner/repo").unwrap(),
      "model.gguf".to_string(),
      "main".to_string(),
    );

    let config_filename = alias.config_filename();
    assert_eq!(config_filename, "test--model");
  }
}
