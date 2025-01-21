use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum EnvType {
  Production,
  #[default]
  Development,
}
impl EnvType {
  pub fn is_production(&self) -> bool {
    self == &EnvType::Production
  }
}

#[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum AppType {
  Native,
  Container,
}

impl AppType {
  pub fn is_native(&self) -> bool {
    self == &AppType::Native
  }
}

#[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum LogLevel {
  Off,
  Error,
  Warn,
  Info,
  Debug,
  Trace,
}

impl From<LogLevel> for tracing::log::LevelFilter {
  fn from(value: LogLevel) -> Self {
    tracing::log::LevelFilter::from_str(&value.to_string())
      .unwrap_or(tracing::log::LevelFilter::Warn)
  }
}

impl From<LogLevel> for tracing::level_filters::LevelFilter {
  fn from(value: LogLevel) -> Self {
    match value {
      LogLevel::Off => tracing::level_filters::LevelFilter::OFF,
      LogLevel::Error => tracing::level_filters::LevelFilter::ERROR,
      LogLevel::Warn => tracing::level_filters::LevelFilter::WARN,
      LogLevel::Info => tracing::level_filters::LevelFilter::INFO,
      LogLevel::Debug => tracing::level_filters::LevelFilter::DEBUG,
      LogLevel::Trace => tracing::level_filters::LevelFilter::TRACE,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SettingSource {
  CommandLine,
  Environment,
  SettingsFile,
  Default,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SettingType {
  String,
  Number,
  Boolean,
  Option,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumberRange {
  pub min: i64,
  pub max: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SettingMetadata {
  String,
  Number { min: i64, max: i64 },
  Boolean,
  Option { options: Vec<String> },
}

impl SettingMetadata {
  pub fn option(options: &[&str]) -> Self {
    Self::Option {
      options: options.iter().map(|s| s.to_string()).collect(),
    }
  }

  pub fn parse(&self, value: serde_yaml::Value) -> serde_yaml::Value {
    use serde_yaml::to_string;

    match (self, value) {
      (SettingMetadata::String, value @ serde_yaml::Value::String(_)) => value,
      (SettingMetadata::Number { .. }, value @ serde_yaml::Value::Number(_)) => value,
      (SettingMetadata::Boolean, value @ serde_yaml::Value::Bool(_)) => value,
      (SettingMetadata::Option { .. }, value @ serde_yaml::Value::String(_)) => value,
      (SettingMetadata::String | SettingMetadata::Option { .. }, value) => to_string(&value)
        .map(|str| str.trim().to_string())
        .map(|str| str.strip_prefix("'").map(|s| s.to_string()).unwrap_or(str))
        .map(|str| str.strip_suffix("'").map(|s| s.to_string()).unwrap_or(str))
        .map(serde_yaml::Value::String)
        .unwrap_or(value),
      (SettingMetadata::Number { .. }, value) => to_string(&value)
        .map(|str| str.trim().to_string())
        .map(|str| str.strip_prefix("'").map(|s| s.to_string()).unwrap_or(str))
        .map(|str| str.strip_suffix("'").map(|s| s.to_string()).unwrap_or(str))
        .map(|str| serde_yaml::Number::from_str(&str))
        .and_then(|res| res.map(serde_yaml::Value::Number))
        .unwrap_or(value),
      (SettingMetadata::Boolean, value) => {
        let result = to_string(&value)
          .map(|str| str.trim().to_string())
          .map(|str| str.strip_prefix("'").map(|s| s.to_string()).unwrap_or(str))
          .map(|str| str.strip_suffix("'").map(|s| s.to_string()).unwrap_or(str))
          .map(|str| str.parse::<bool>());
        if let Ok(Ok(result)) = result {
          serde_yaml::Value::Bool(result)
        } else {
          value
        }
      }
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingInfo {
  pub key: String,
  pub current_value: serde_yaml::Value,
  pub default_value: serde_yaml::Value,
  pub source: SettingSource,
  pub metadata: SettingMetadata,
}

#[cfg(test)]
mod tests {
  use super::SettingMetadata;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_yaml::Number;

  #[rstest]
  // String metadata tests
  #[case::string_to_string(
    SettingMetadata::String,
    serde_yaml::Value::String("test".to_string()),
    serde_yaml::Value::String("test".to_string())
  )]
  #[case::number_to_string(
    SettingMetadata::String,
    serde_yaml::Value::Number(Number::from(123)),
    serde_yaml::Value::String("123".to_string())
  )]
  #[case::boolean_to_string(
    SettingMetadata::String,
    serde_yaml::Value::Bool(true),
    serde_yaml::Value::String("true".to_string())
  )]
  // Number metadata tests
  #[case::number_to_number(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_yaml::Value::Number(Number::from(50)),
    serde_yaml::Value::Number(Number::from(50))
  )]
  #[case::string_to_number(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_yaml::Value::String("42".to_string()),
    serde_yaml::Value::Number(Number::from(42))
  )]
  #[case(
    SettingMetadata::Number { min: -100, max: 100 },
    serde_yaml::Value::String("-42".to_string()),
    serde_yaml::Value::Number(Number::from(-42))
  )]
  // Boolean metadata tests
  #[case(
    SettingMetadata::Boolean,
    serde_yaml::Value::Bool(true),
    serde_yaml::Value::Bool(true)
  )]
  #[case(
    SettingMetadata::Boolean,
    serde_yaml::Value::Bool(false),
    serde_yaml::Value::Bool(false)
  )]
  #[case(
    SettingMetadata::Boolean,
    serde_yaml::Value::String("true".to_string()),
    serde_yaml::Value::Bool(true)
  )]
  #[case(
    SettingMetadata::Boolean,
    serde_yaml::Value::String("false".to_string()),
    serde_yaml::Value::Bool(false)
  )]
  // Option metadata tests
  #[case(
    SettingMetadata::Option { options: vec!["a".to_string(), "b".to_string()] },
    serde_yaml::Value::String("a".to_string()),
    serde_yaml::Value::String("a".to_string())
  )]
  #[case(
    SettingMetadata::Option { options: vec!["a".to_string(), "b".to_string()] },
    serde_yaml::Value::Number(Number::from(123)),
    serde_yaml::Value::String("123".to_string())
  )]
  #[case(
    SettingMetadata::Option { options: vec!["true".to_string(), "false".to_string()] },
    serde_yaml::Value::Bool(true),
    serde_yaml::Value::String("true".to_string())
  )]
  #[case(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_yaml::Value::String("12.34".to_string()),
    serde_yaml::Value::Number(Number::from(12.34))
  )]
  fn test_setting_metadata_parse(
    #[case] metadata: SettingMetadata,
    #[case] input: serde_yaml::Value,
    #[case] expected: serde_yaml::Value,
  ) {
    assert_eq!(expected, metadata.parse(input));
  }

  #[rstest]
  // Invalid number strings
  #[case(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_yaml::Value::String("not_a_number".to_string())
  )]
  #[case(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_yaml::Value::String("".to_string())
  )]
  // Invalid boolean strings
  #[case(
    SettingMetadata::Boolean,
    serde_yaml::Value::String("not_a_bool".to_string())
  )]
  #[case(
    SettingMetadata::Boolean,
    serde_yaml::Value::String("1".to_string())
  )]
  #[case(
    SettingMetadata::Boolean,
    serde_yaml::Value::String("".to_string())
  )]
  fn test_setting_metadata_parse_invalid_values(
    #[case] metadata: SettingMetadata,
    #[case] input: serde_yaml::Value,
  ) {
    // When parsing fails, the original value should be returned
    assert_eq!(input, metadata.parse(input.clone()));
  }
}
