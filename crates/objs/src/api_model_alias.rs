use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiAlias {
  pub id: String,
  pub provider: String,
  pub base_url: String,
  pub models: Vec<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl ApiAlias {
  pub fn new(
    id: impl Into<String>,
    provider: impl Into<String>,
    base_url: impl Into<String>,
    models: Vec<String>,
    created_at: DateTime<Utc>,
  ) -> Self {
    Self {
      id: id.into(),
      provider: provider.into(),
      base_url: base_url.into(),
      models,
      created_at,
      updated_at: created_at,
    }
  }
}

impl std::fmt::Display for ApiAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ApiAlias {{ id: {}, provider: {}, models: {:?} }}",
      self.id, self.provider, self.models
    )
  }
}

#[cfg(test)]
mod test {
  use super::ApiAlias;
  use chrono::Utc;

  #[test]
  fn test_api_model_alias_creation() {
    let created_at = Utc::now();
    let alias = ApiAlias::new(
      "openai",
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      created_at,
    );

    let expected = ApiAlias {
      id: "openai".to_string(),
      provider: "openai".to_string(),
      base_url: "https://api.openai.com/v1".to_string(),
      models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      created_at,
      updated_at: created_at,
    };

    assert_eq!(alias, expected);
  }

  #[test]
  fn test_api_model_alias_display() {
    let alias = ApiAlias::new(
      "test-api",
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string()],
      Utc::now(),
    );

    let display = format!("{}", alias);
    assert!(display.contains("test-api"));
    assert!(display.contains("openai"));
    assert!(display.contains("gpt-4"));
  }

  #[test]
  fn test_api_model_alias_serialization() -> anyhow::Result<()> {
    let alias = ApiAlias::new(
      "test",
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string()],
      Utc::now(),
    );

    let serialized = serde_json::to_string(&alias)?;
    let deserialized: ApiAlias = serde_json::from_str(&serialized)?;

    assert_eq!(alias, deserialized);
    Ok(())
  }
}
