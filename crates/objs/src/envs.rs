use std::str::FromStr;

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
