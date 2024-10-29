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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub library_path: Option<String>,
}

impl Settings {
  pub fn app_default() -> Self {
    let library_path = format!(
      "{}/{}/{}",
      llamacpp_sys::BUILD_TARGET,
      llamacpp_sys::DEFAULT_VARIANT,
      llamacpp_sys::LIBRARY_NAME
    );
    Self {
      library_path: Some(library_path),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_settings_serialize_empty() {
    let settings = Settings { library_path: None };
    let yaml = serde_yaml::to_string(&settings).unwrap();
    assert_eq!("{}\n", yaml);
  }

  #[test]
  fn test_serialize_from_empty() {
    let settings: Settings = serde_yaml::from_str("").unwrap();
    assert_eq!(Settings { library_path: None }, settings);
  }
}
