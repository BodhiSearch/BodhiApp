use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// TTL for API model cache (24 hours)
pub const CACHE_TTL_HOURS: i64 = 24;

/// API format/protocol specification
#[derive(
  Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, strum::Display, strum::EnumString,
)]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ApiFormat {
  #[serde(rename = "openai")]
  #[strum(serialize = "openai")]
  #[cfg_attr(any(test, feature = "test-utils"), default)]
  OpenAI,
  #[serde(rename = "placeholder")]
  #[strum(serialize = "placeholder")]
  Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, derive_builder::Builder)]
#[builder(setter(into, strip_option), build_fn(error = crate::BuilderError))]
#[cfg_attr(any(test, feature = "test-utils"), derive(Default))]
pub struct ApiAlias {
  pub id: String,
  pub api_format: ApiFormat,
  pub base_url: String,
  #[builder(default)]
  pub models: Vec<String>,
  #[builder(default)]
  pub prefix: Option<String>,
  #[builder(default)]
  pub forward_all_with_prefix: bool,
  #[builder(default)]
  pub models_cache: Vec<String>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub cache_fetched_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub created_at: DateTime<Utc>,
  #[schema(value_type = String, format = "date-time")]
  #[builder(setter(skip))]
  pub updated_at: DateTime<Utc>,
}

impl ApiAlias {
  pub fn new(
    id: impl Into<String>,
    api_format: ApiFormat,
    base_url: impl Into<String>,
    models: Vec<String>,
    prefix: Option<String>,
    forward_all_with_prefix: bool,
    created_at: DateTime<Utc>,
  ) -> Self {
    // Epoch sentinel for "never fetched"
    let epoch = DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
      .unwrap()
      .with_timezone(&Utc);

    Self {
      id: id.into(),
      api_format,
      base_url: base_url.into(),
      models,
      prefix,
      forward_all_with_prefix,
      models_cache: Vec::new(),
      cache_fetched_at: epoch,
      created_at,
      updated_at: created_at,
    }
  }

  pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
    self.prefix = Some(prefix.into());
    self
  }

  /// Returns the appropriate models list based on the forward_all_with_prefix flag.
  /// - If forward_all_with_prefix=true: returns models_cache (unprefixed cached models)
  /// - If forward_all_with_prefix=false: returns models (user-specified models)
  pub fn get_models(&self) -> &Vec<String> {
    if self.forward_all_with_prefix {
      &self.models_cache
    } else {
      &self.models
    }
  }

  pub fn matchable_models(&self) -> Vec<String> {
    let prefix = self.prefix.as_deref().unwrap_or("");

    let source = if self.forward_all_with_prefix {
      &self.models_cache
    } else {
      &self.models
    };

    source
      .iter()
      .map(|model| format!("{}{}", prefix, model))
      .collect()
  }

  /// Check if this API alias supports a given model name.
  ///
  /// If `forward_all_with_prefix` is true, checks if the model starts with the prefix.
  /// If `forward_all_with_prefix` is false, checks if the model is in matchable_models list.
  pub fn supports_model(&self, model: &str) -> bool {
    if self.forward_all_with_prefix {
      self.prefix.as_ref().map_or(false, |p| model.starts_with(p))
    } else {
      self.matchable_models().contains(&model.to_string())
    }
  }

  /// Check if the cache is stale (older than TTL).
  pub fn is_cache_stale(&self) -> bool {
    Utc::now() - self.cache_fetched_at > Duration::hours(CACHE_TTL_HOURS)
  }

  /// Check if the cache is empty.
  pub fn is_cache_empty(&self) -> bool {
    self.models_cache.is_empty()
  }
}

impl ApiAliasBuilder {
  /// Build an ApiAlias with the provided timestamp for created_at and updated_at.
  ///
  /// This is the primary build method for ApiAlias since timestamps cannot be set through
  /// the builder's field setters (they are marked with `#[builder(setter(skip))]`).
  pub fn build_with_time(&self, timestamp: DateTime<Utc>) -> Result<ApiAlias, crate::BuilderError> {
    // Epoch sentinel for "never fetched"
    let epoch = DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
      .unwrap()
      .with_timezone(&Utc);

    Ok(ApiAlias {
      id: self
        .id
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("id"))?,
      api_format: self
        .api_format
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("api_format"))?,
      base_url: self
        .base_url
        .clone()
        .ok_or_else(|| crate::BuilderError::UninitializedField("base_url"))?,
      models: self.models.clone().unwrap_or_default(),
      prefix: self.prefix.clone().unwrap_or_default(),
      forward_all_with_prefix: self.forward_all_with_prefix.unwrap_or_default(),
      models_cache: self.models_cache.clone().unwrap_or_default(),
      cache_fetched_at: epoch,
      created_at: timestamp,
      updated_at: timestamp,
    })
  }

  /// Create a builder pre-configured with test defaults.
  ///
  /// This convenience method is useful in tests to create builders with sensible defaults
  /// that can be customized as needed.
  #[cfg(any(test, feature = "test-utils"))]
  pub fn test_default() -> Self {
    let mut builder = Self::default();
    builder
      .id("test-id")
      .api_format(ApiFormat::OpenAI)
      .base_url("http://localhost:8080");
    builder
  }
}

impl std::fmt::Display for ApiAlias {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ApiAlias {{ id: {}, api_format: {}, prefix: {:?}, models: {:?} }}",
      self.id, self.api_format, self.prefix, self.models
    )
  }
}

#[cfg(test)]
mod test {
  use super::{ApiAlias, ApiAliasBuilder, ApiFormat};
  use chrono::Utc;
  use rstest::rstest;

  #[rstest]
  #[case(None, vec!["gpt-4".to_string()], false)]
  #[case(Some("azure".to_string()), vec!["gpt-4".to_string()], false)]
  #[case(Some("azure/".to_string()), vec!["gpt-4".to_string()], true)]
  fn test_api_model_alias_serialization(
    #[case] prefix: Option<String>,
    #[case] models: Vec<String>,
    #[case] forward_all: bool,
  ) -> anyhow::Result<()> {
    let mut builder = ApiAliasBuilder::test_default();
    builder
      .id("test")
      .base_url("https://api.openai.com/v1")
      .models(models)
      .forward_all_with_prefix(forward_all);
    if let Some(p) = prefix {
      builder.prefix(p);
    }
    let alias = builder.build_with_time(Utc::now())?;

    let serialized = serde_json::to_string(&alias)?;
    let deserialized: ApiAlias = serde_json::from_str(&serialized)?;

    assert_eq!(alias, deserialized);
    Ok(())
  }

  #[test]
  fn test_api_model_alias_with_prefix_builder() {
    let alias = ApiAliasBuilder::test_default()
      .id("openai")
      .base_url("https://api.openai.com/v1")
      .models(vec!["gpt-4".to_string()])
      .build_with_time(Utc::now())
      .unwrap()
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
    let mut builder = ApiAliasBuilder::test_default();
    builder.id("test").base_url("url").models(models);
    if let Some(p) = prefix {
      builder.prefix(p);
    }
    let alias = builder.build_with_time(Utc::now()).unwrap();
    let matchable = alias.matchable_models();

    assert_eq!(expected, matchable);
  }

  #[rstest]
  #[case(false, Some("azure/".to_string()), vec!["gpt-4".to_string()], "azure/gpt-4", true)]
  #[case(false, Some("azure/".to_string()), vec!["gpt-4".to_string()], "azure/gpt-3.5", false)]
  #[case(true, Some("azure/".to_string()), vec!["gpt-4".to_string()], "azure/gpt-4", true)]
  #[case(true, Some("azure/".to_string()), vec!["gpt-4".to_string()], "azure/gpt-3.5", true)]
  #[case(true, Some("azure/".to_string()), vec!["gpt-4".to_string()], "openai/gpt-4", false)]
  #[case(true, None, vec!["gpt-4".to_string()], "gpt-4", false)]
  fn test_supports_model(
    #[case] forward_all: bool,
    #[case] prefix: Option<String>,
    #[case] models: Vec<String>,
    #[case] model_to_check: &str,
    #[case] expected: bool,
  ) {
    let mut builder = ApiAliasBuilder::test_default();
    builder
      .id("test")
      .base_url("url")
      .models(models)
      .forward_all_with_prefix(forward_all);
    if let Some(p) = prefix {
      builder.prefix(p);
    }
    let alias = builder.build_with_time(Utc::now()).unwrap();
    let result = alias.supports_model(model_to_check);

    assert_eq!(expected, result);
  }

  #[test]
  fn test_api_format_serialization() -> anyhow::Result<()> {
    let format = ApiFormat::OpenAI;
    let serialized = serde_json::to_string(&format)?;
    assert_eq!(serialized, "\"openai\"");

    let deserialized: ApiFormat = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized, ApiFormat::OpenAI);
    Ok(())
  }

  #[test]
  fn test_api_format_display() {
    assert_eq!(ApiFormat::OpenAI.to_string(), "openai");
  }
}
