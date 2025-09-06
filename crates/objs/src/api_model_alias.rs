use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ApiAlias {
  pub id: String,
  pub provider: String,
  pub base_url: String,
  pub models: Vec<String>,
  pub prefix: Option<String>,
  #[schema(value_type = String, format = "date-time")]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  pub updated_at: DateTime<Utc>,
}

impl ApiAlias {
  pub fn new(
    id: impl Into<String>,
    provider: impl Into<String>,
    base_url: impl Into<String>,
    models: Vec<String>,
    prefix: Option<String>,
    created_at: DateTime<Utc>,
  ) -> Self {
    Self {
      id: id.into(),
      provider: provider.into(),
      base_url: base_url.into(),
      models,
      prefix,
      created_at,
      updated_at: created_at,
    }
  }

  pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
    self.prefix = Some(prefix.into());
    self
  }

  pub fn matchable_models(&self) -> Vec<String> {
    let prefix = self.prefix.as_deref().unwrap_or("");

    self
      .models
      .iter()
      .map(|model| format!("{}{}", prefix, model))
      .collect()
  }
}

impl std::fmt::Display for ApiAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ApiAlias {{ id: {}, provider: {}, prefix: {:?}, models: {:?} }}",
      self.id, self.provider, self.prefix, self.models
    )
  }
}

#[cfg(test)]
mod test {
  use super::ApiAlias;
  use chrono::Utc;
  use rstest::rstest;

  #[rstest]
  #[case(None, vec!["gpt-4".to_string()])]
  #[case(Some("azure".to_string()), vec!["gpt-4".to_string()])]
  fn test_api_model_alias_serialization(
    #[case] prefix: Option<String>,
    #[case] models: Vec<String>,
  ) -> anyhow::Result<()> {
    let alias = ApiAlias::new(
      "test",
      "openai",
      "https://api.openai.com/v1",
      models,
      prefix,
      Utc::now(),
    );

    let serialized = serde_json::to_string(&alias)?;
    let deserialized: ApiAlias = serde_json::from_str(&serialized)?;

    assert_eq!(alias, deserialized);
    Ok(())
  }

  #[test]
  fn test_api_model_alias_with_prefix_builder() {
    let alias = ApiAlias::new(
      "openai",
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string()],
      None,
      Utc::now(),
    )
    .with_prefix("openai");

    assert_eq!(alias.prefix, Some("openai".to_string()));
  }

  #[rstest]
  #[case(vec!["gpt-4".to_string()], None, vec!["gpt-4".to_string()])]
  #[case(vec!["gpt-4".to_string()], Some("azure/".to_string()), vec!["azure/gpt-4".to_string()])]
  #[case(vec!["gpt-4".to_string(), "gpt-3.5".to_string()], Some("openai:".to_string()), vec!["openai:gpt-4".to_string(), "openai:gpt-3.5".to_string()])]
  fn test_matchable_models(
    #[case] models: Vec<String>,
    #[case] prefix: Option<String>,
    #[case] expected: Vec<String>,
  ) {
    let alias = ApiAlias::new("test", "provider", "url", models, prefix, Utc::now());
    let matchable = alias.matchable_models();

    assert_eq!(expected, matchable);
  }
}
