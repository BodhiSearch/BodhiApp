use super::error::SettingsMetadataError;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, derive_new::new)]
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
  Database,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
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

#[derive(Debug, Clone)]
pub enum AppCommand {
  Serve {
    host: Option<String>,
    port: Option<u16>,
  },
  Default,
}

#[cfg(test)]
#[path = "test_setting_objs.rs"]
mod test_setting_objs;
