use super::{AppCommand, Setting};
use crate::EnvWrapper;
use serde_yaml::Value;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

pub struct BootstrapParts {
  pub env_wrapper: Arc<dyn EnvWrapper>,
  pub settings_file: PathBuf,
  pub system_settings: Vec<Setting>,
  pub file_defaults: HashMap<String, Value>,
  pub app_settings: HashMap<String, String>,
  pub app_command: AppCommand,
  pub bodhi_home: PathBuf,
}
