use crate::BootstrapError;
use objs::{AppCommand, LogLevel, Setting};
use serde_yaml::Value;
use services::{
  BootstrapParts, EnvWrapper, BODHI_HOME, BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT,
  DEFAULT_LOG_LEVEL, DEFAULT_LOG_STDOUT, LOGS_DIR,
};
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct BootstrapService {
  // Passthrough state (flows to SettingService via into_parts)
  env_wrapper: Arc<dyn EnvWrapper>,
  settings_file: PathBuf,
  system_settings: Vec<Setting>,
  file_defaults: HashMap<String, Value>,
  app_settings: HashMap<String, String>,
  app_command: AppCommand,

  // Pre-computed bootstrap-critical values
  bodhi_home: PathBuf,
  logs_dir: PathBuf,
  log_level: LogLevel,
  log_stdout: bool,
}

impl BootstrapService {
  pub fn new(
    env_wrapper: Arc<dyn EnvWrapper>,
    system_settings: Vec<Setting>,
    file_defaults: HashMap<String, Value>,
    settings_file: PathBuf,
    app_settings: HashMap<String, String>,
    app_command: AppCommand,
  ) -> std::result::Result<Self, BootstrapError> {
    let bodhi_home_setting = system_settings
      .iter()
      .find(|s| s.key == BODHI_HOME)
      .ok_or(BootstrapError::BodhiHomeNotSet)?;
    let bodhi_home = bodhi_home_setting
      .value
      .as_str()
      .ok_or(BootstrapError::BodhiHomeNotSet)
      .map(PathBuf::from)?;

    // Read settings file once for log resolution
    let settings_map: serde_yaml::Mapping = if settings_file.exists() {
      let contents = fs::read_to_string(&settings_file).unwrap_or_default();
      serde_yaml::from_str(&contents).unwrap_or_default()
    } else {
      serde_yaml::Mapping::new()
    };

    // Resolve logs_dir: env > settings.yaml > convention
    let logs_dir = env_wrapper
      .var(BODHI_LOGS)
      .ok()
      .or_else(|| {
        settings_map
          .get(Value::String(BODHI_LOGS.to_string()))
          .and_then(|v| v.as_str().map(|s| s.to_string()))
      })
      .map(PathBuf::from)
      .unwrap_or_else(|| bodhi_home.join(LOGS_DIR));

    // Resolve log_level: env > settings.yaml > convention ("warn")
    let log_level_str = env_wrapper
      .var(BODHI_LOG_LEVEL)
      .ok()
      .or_else(|| {
        settings_map
          .get(Value::String(BODHI_LOG_LEVEL.to_string()))
          .and_then(|v| v.as_str().map(|s| s.to_string()))
      })
      .unwrap_or_else(|| DEFAULT_LOG_LEVEL.to_string());
    let log_level = LogLevel::try_from(log_level_str.as_str()).unwrap_or(LogLevel::Warn);

    // Resolve log_stdout: env > settings.yaml > convention (false)
    let log_stdout = env_wrapper
      .var(BODHI_LOG_STDOUT)
      .ok()
      .map(|s| s == "true" || s == "1")
      .or_else(|| {
        settings_map
          .get(Value::String(BODHI_LOG_STDOUT.to_string()))
          .map(|v| match v {
            Value::Bool(b) => *b,
            Value::String(s) => s == "true" || s == "1",
            _ => DEFAULT_LOG_STDOUT,
          })
      })
      .unwrap_or(DEFAULT_LOG_STDOUT);

    Ok(Self {
      env_wrapper,
      settings_file,
      system_settings,
      file_defaults,
      app_settings,
      app_command,
      bodhi_home,
      logs_dir,
      log_level,
      log_stdout,
    })
  }

  pub fn into_parts(self) -> BootstrapParts {
    BootstrapParts {
      env_wrapper: self.env_wrapper,
      settings_file: self.settings_file,
      system_settings: self.system_settings,
      file_defaults: self.file_defaults,
      app_settings: self.app_settings,
      app_command: self.app_command,
      bodhi_home: self.bodhi_home,
    }
  }

  // --- Bootstrap-critical typed READ accessors ---

  pub fn bodhi_home(&self) -> PathBuf {
    self.bodhi_home.clone()
  }

  pub fn logs_dir(&self) -> PathBuf {
    self.logs_dir.clone()
  }

  pub fn log_level(&self) -> LogLevel {
    self.log_level.clone()
  }

  pub fn log_stdout(&self) -> bool {
    self.log_stdout
  }
}
