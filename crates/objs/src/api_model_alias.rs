use crate::AliasSource;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiModelAlias {
  pub id: String,
  pub source: AliasSource,
  pub provider: String,
  pub base_url: String,
  pub models: Vec<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl ApiModelAlias {
  pub fn new(
    id: impl Into<String>,
    source: AliasSource,
    provider: impl Into<String>,
    base_url: impl Into<String>,
    models: Vec<String>,
    created_at: DateTime<Utc>,
  ) -> Self {
    Self {
      id: id.into(),
      source,
      provider: provider.into(),
      base_url: base_url.into(),
      models,
      created_at,
      updated_at: created_at,
    }
  }
}

impl std::fmt::Display for ApiModelAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ApiModelAlias {{ id: {}, provider: {}, models: {:?} }}",
      self.id, self.provider, self.models
    )
  }
}

#[cfg(test)]
mod test {
  use super::ApiModelAlias;
  use crate::AliasSource;
  use chrono::Utc;

  #[test]
  fn test_api_model_alias_creation() {
    let created_at = Utc::now();
    let alias = ApiModelAlias::new(
      "openai",
      AliasSource::RemoteApi,
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
      created_at,
    );

    let expected = ApiModelAlias {
      id: "openai".to_string(),
      source: AliasSource::RemoteApi,
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
    let alias = ApiModelAlias::new(
      "test-api",
      AliasSource::RemoteApi,
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
    let alias = ApiModelAlias::new(
      "test",
      AliasSource::RemoteApi,
      "openai",
      "https://api.openai.com/v1",
      vec!["gpt-4".to_string()],
      Utc::now(),
    );

    let serialized = serde_json::to_string(&alias)?;
    let deserialized: ApiModelAlias = serde_json::from_str(&serialized)?;

    assert_eq!(alias, deserialized);
    Ok(())
  }
}
