use crate::{AppError, ErrorType};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Default, strum::EnumString, strum::Display, serde::Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display, serde::Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, derive_new::new)]
pub struct Setting {
  #[new(into)]
  pub key: String,
  pub value: serde_yaml::Value,
  pub source: SettingSource,
  pub metadata: SettingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum SettingSource {
  System,
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

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SettingsMetadataError {
  #[error("invalid_value_type")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidValueType(SettingMetadata, serde_json::Value),
  #[error("invalid_value")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidValue(serde_json::Value),
  #[error("null_value")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  NullValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, PartialOrd, strum::Display)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SettingMetadata {
  String,
  Number { min: i64, max: i64 },
  Boolean,
  Option { options: Vec<String> },
}

impl SettingMetadata {
  pub fn option(options: Vec<String>) -> Self {
    Self::Option { options }
  }

  pub fn parse(&self, value: serde_yaml::Value) -> serde_yaml::Value {
    use serde_yaml::to_string;

    match (self, value) {
      (SettingMetadata::String, value @ serde_yaml::Value::String(_)) => value,
      (SettingMetadata::Number { .. }, value @ serde_yaml::Value::Number(_)) => value,
      (SettingMetadata::Boolean, value @ serde_yaml::Value::Bool(_)) => value,
      (SettingMetadata::Boolean, serde_yaml::Value::String(value)) => value
        .parse::<bool>()
        .ok()
        .map(serde_yaml::Value::Bool)
        .unwrap_or_else(|| serde_yaml::Value::String(value)),
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

  pub fn convert(
    self: &SettingMetadata,
    value: serde_json::Value,
  ) -> Result<serde_yaml::Value, SettingsMetadataError> {
    let orig = value.clone();

    if let serde_json::Value::Null = value {
      return Err(SettingsMetadataError::NullValue);
    }

    match self {
      SettingMetadata::String => match value {
        serde_json::Value::String(s) => Ok(serde_yaml::Value::String(s)),
        _ => Ok(serde_yaml::Value::String(
          value.as_str().unwrap_or(&value.to_string()).to_string(),
        )),
      },

      SettingMetadata::Option { options } => {
        let option = match value {
          serde_json::Value::String(s) => s,
          _ => value.as_str().unwrap_or(&value.to_string()).to_string(),
        };

        if options.contains(&option) {
          Ok(serde_yaml::Value::String(option))
        } else {
          Err(SettingsMetadataError::InvalidValue(orig))
        }
      }

      metadata @ SettingMetadata::Number { min, max } => {
        let number = match value {
          serde_json::Value::Number(n) => serde_yaml::Number::from_str(&n.to_string()),
          serde_json::Value::String(s) => serde_yaml::Number::from_str(&s),
          _ => serde_yaml::Number::from_str(&value.to_string()),
        }
        .map_err(|_| SettingsMetadataError::InvalidValueType(metadata.clone(), orig.clone()))?;

        let num_val = number
          .as_f64()
          .ok_or_else(|| SettingsMetadataError::InvalidValueType(metadata.clone(), orig.clone()))?;

        if num_val >= *min as f64 && num_val <= *max as f64 {
          Ok(serde_yaml::Value::Number(number))
        } else {
          Err(SettingsMetadataError::InvalidValue(orig))
        }
      }

      metadata @ SettingMetadata::Boolean => match value {
        serde_json::Value::Bool(b) => Ok(serde_yaml::Value::Bool(b)),
        serde_json::Value::String(s) => s
          .parse::<bool>()
          .map(serde_yaml::Value::Bool)
          .map_err(|_| SettingsMetadataError::InvalidValueType(metadata.clone(), orig)),
        _ => value
          .to_string()
          .parse::<bool>()
          .map(serde_yaml::Value::Bool)
          .map_err(|_| SettingsMetadataError::InvalidValueType(metadata.clone(), orig)),
      },
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, PartialOrd)]
pub struct SettingInfo {
  pub key: String,
  pub current_value: serde_yaml::Value,
  pub default_value: serde_yaml::Value,
  pub source: SettingSource,
  pub metadata: SettingMetadata,
}

impl SettingInfo {
  pub fn new_system_setting<T, U>(key: T, default_value: U) -> Self
  where
    T: Display,
    U: Display,
  {
    Self {
      key: key.to_string(),
      current_value: serde_yaml::Value::String(default_value.to_string()),
      default_value: serde_yaml::Value::String(default_value.to_string()),
      source: SettingSource::Default,
      metadata: SettingMetadata::String,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::SettingMetadata;
  use crate::{test_utils::setup_l10n, ApiError, FluentLocalizationService, OpenAIApiError};
  // use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_yaml::Number;
  use std::sync::Arc;

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

  #[rstest]
  // String metadata tests
  #[case::convert_string_to_string(
    SettingMetadata::String,
    serde_json::json!("test"),
    serde_yaml::Value::String("test".to_string())
  )]
  #[case::convert_number_to_string(
    SettingMetadata::String,
    serde_json::json!(42),
    serde_yaml::Value::String("42".to_string())
  )]
  #[case::convert_option_to_string(
    SettingMetadata::Option { options: vec!["a".to_string(), "b".to_string()] },
    serde_json::json!("a"),
    serde_yaml::Value::String("a".to_string())
  )]
  // Number metadata tests
  #[case::convert_number_to_number(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_json::json!(42),
    serde_yaml::Value::Number(serde_yaml::Number::from(42))
  )]
  #[case::convert_string_to_number(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_json::json!("42"),
    serde_yaml::Value::Number(serde_yaml::Number::from(42))
  )]
  #[case::convert_string_to_number(
    SettingMetadata::Number { min: -100, max: 100 },
    serde_json::json!(-42),
    serde_yaml::Value::Number(serde_yaml::Number::from(-42))
  )]
  // Boolean metadata tests
  #[case::convert_boolean_to_boolean(
    SettingMetadata::Boolean,
    serde_json::json!(true),
    serde_yaml::Value::Bool(true)
  )]
  #[case::convert_string_to_boolean(
    SettingMetadata::Boolean,
    serde_json::json!("true"),
    serde_yaml::Value::Bool(true)
  )]
  fn test_setting_metadata_convert_success(
    #[case] metadata: SettingMetadata,
    #[case] input: serde_json::Value,
    #[case] expected: serde_yaml::Value,
  ) {
    let result = metadata.convert(input);
    assert!(result.is_ok());
    assert_eq!(expected, result.unwrap());
  }

  #[rstest]
  // Invalid string for boolean
  #[case::invalid_string_for_boolean(
    SettingMetadata::Boolean,
    serde_json::json!("not_a_bool"),
    "cannot parse \"not_a_bool\" as Boolean"
  )]
  // Invalid type combinations
  #[case(
    SettingMetadata::Boolean,
    serde_json::json!(42),
    "cannot parse 42 as Boolean"
  )]
  #[case(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_json::json!(true),
    "cannot parse true as Number"
  )]
  // Number range validation
  #[case::number_range_validation_lower(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_json::json!(-1),
    "passed value is not a valid value: -1"
  )]
  #[case::number_range_validation_upper(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_json::json!(101),
    "passed value is not a valid value: 101"
  )]
  #[case::number_range_validation_string(
    SettingMetadata::Number { min: 0, max: 100 },
    serde_json::json!("101"),
    "passed value is not a valid value: \"101\""
  )]
  // Option validation
  #[case(
    SettingMetadata::Option { options: vec!["a".to_string(), "b".to_string()] },
    serde_json::json!("c"),
    "passed value is not a valid value: \"c\""
  )]
  // Null test
  #[case(
    SettingMetadata::String,
    serde_json::json!(null),
    "value is null"
  )]
  fn test_setting_metadata_convert_error(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    #[case] metadata: SettingMetadata,
    #[case] input: serde_json::Value,
    #[case] expected_error: &str,
  ) {
    let app_error = metadata.convert(input).unwrap_err();
    let message = OpenAIApiError::from(ApiError::from(app_error))
      .error
      .message
      .replace("\u{2068}", "")
      .replace("\u{2069}", "");
    assert_eq!(expected_error, message.trim());
  }
}
