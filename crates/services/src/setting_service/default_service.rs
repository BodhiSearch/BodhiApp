use super::{Result, *};
use crate::{asref_impl, EnvWrapper};
use objs::{Setting, SettingInfo, SettingMetadata, SettingSource};
use serde_yaml::Value;
use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
  sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct DefaultSettingService {
  env_wrapper: Arc<dyn EnvWrapper>,
  settings_file: PathBuf,
  system_settings: Vec<Setting>,
  settings_lock: RwLock<()>,
  defaults: RwLock<HashMap<String, Value>>,
  listeners: RwLock<Vec<Arc<dyn SettingsChangeListener>>>,
  cmd_lines: RwLock<HashMap<String, Value>>,
}

impl DefaultSettingService {
  pub(crate) fn new(
    env_wrapper: Arc<dyn EnvWrapper>,
    settings_file: PathBuf,
    system_settings: Vec<Setting>,
  ) -> DefaultSettingService {
    Self {
      env_wrapper,
      settings_file,
      system_settings,
      settings_lock: RwLock::new(()),
      defaults: RwLock::new(HashMap::new()),
      listeners: RwLock::new(Vec::new()),
      cmd_lines: RwLock::new(HashMap::new()),
    }
  }

  pub fn new_with_defaults(
    env_wrapper: Arc<dyn EnvWrapper>,
    bodhi_home: Setting,
    mut system_settings: Vec<Setting>,
    file_defaults: HashMap<String, Value>,
    settings_file: PathBuf,
  ) -> Self {
    system_settings.push(bodhi_home.clone());
    let service = Self::new(env_wrapper, settings_file, system_settings);
    service.init_defaults(bodhi_home, file_defaults);
    service
  }

  fn init_defaults(&self, bodhi_home: Setting, file_defaults: HashMap<String, Value>) {
    self.with_defaults_write_lock(|defaults| {
      if bodhi_home.source == SettingSource::Default {
        defaults.insert(BODHI_HOME.to_string(), bodhi_home.value.clone());
        defaults.insert(
          BODHI_LOGS.to_string(),
          Value::String(
            PathBuf::from(bodhi_home.value.as_str().unwrap())
              .join(LOGS_DIR)
              .display()
              .to_string(),
          ),
        );
      } else if let Some(home_dir) = self.home_dir() {
        let default_bodhi_home = home_dir.join(".cache").join("bodhi");
        defaults.insert(
          BODHI_HOME.to_string(),
          Value::String(default_bodhi_home.display().to_string()),
        );
        defaults.insert(
          BODHI_LOGS.to_string(),
          Value::String(default_bodhi_home.join("logs").display().to_string()),
        );
      }
      if let Some(home_dir) = self.home_dir() {
        defaults.insert(
          HF_HOME.to_string(),
          Value::String(
            home_dir
              .join(".cache")
              .join("huggingface")
              .display()
              .to_string(),
          ),
        );
      }
      macro_rules! set_default {
        ($key:expr, $hardcoded_value:expr) => {
          defaults.insert(
            $key.to_string(),
            file_defaults.get($key).cloned().unwrap_or($hardcoded_value),
          );
        };
      }
      set_default!(BODHI_SCHEME, Value::String(DEFAULT_SCHEME.to_string()));
      set_default!(BODHI_HOST, Value::String(DEFAULT_HOST.to_string()));
      set_default!(BODHI_PORT, Value::Number(DEFAULT_PORT.into()));
      set_default!(
        BODHI_LOG_LEVEL,
        Value::String(DEFAULT_LOG_LEVEL.to_string())
      );
      set_default!(BODHI_LOG_STDOUT, Value::Bool(DEFAULT_LOG_STDOUT));
      set_default!(
        BODHI_EXEC_TARGET,
        Value::String(llama_server_proc::BUILD_TARGET.to_string())
      );
      set_default!(
        BODHI_EXEC_VARIANTS,
        Value::String(
          llama_server_proc::BUILD_VARIANTS
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(","),
        )
      );
      set_default!(
        BODHI_EXEC_VARIANT,
        Value::String(llama_server_proc::DEFAULT_VARIANT.to_string())
      );
      set_default!(
        BODHI_EXEC_NAME,
        Value::String(llama_server_proc::EXEC_NAME.to_string())
      );
      set_default!(
        BODHI_LLAMACPP_ARGS,
        Value::String("--jinja --no-webui".to_string())
      );
      set_default!(
        BODHI_KEEP_ALIVE_SECS,
        Value::Number(DEFAULT_KEEP_ALIVE_SECS.into())
      );
      set_default!(
        BODHI_CANONICAL_REDIRECT,
        Value::Bool(DEFAULT_CANONICAL_REDIRECT)
      );
      for (key, value) in file_defaults {
        defaults.insert(key, value);
      }
    });
  }

  pub fn with_settings_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&serde_yaml::Mapping) -> R,
  {
    let _guard = self.settings_lock.read().unwrap();
    if !self.settings_file.exists() {
      return f(&serde_yaml::Mapping::new());
    }
    let contents = fs::read_to_string(&self.settings_file).unwrap_or_else(|_| String::new());
    let settings: serde_yaml::Mapping =
      serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new());
    f(&settings)
  }

  pub fn with_settings_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut serde_yaml::Mapping),
  {
    let _guard = self.settings_lock.write().unwrap();
    let mut settings = if !self.settings_file.exists() {
      serde_yaml::Mapping::new()
    } else {
      let contents = fs::read_to_string(&self.settings_file).unwrap_or_else(|_| String::new());
      serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new())
    };
    f(&mut settings);
    let contents = serde_yaml::to_string(&settings).unwrap();
    fs::write(&self.settings_file, contents).unwrap();
  }

  pub fn with_defaults_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&HashMap<String, Value>) -> R,
  {
    let defaults = self.defaults.read().unwrap();
    f(&defaults)
  }

  pub fn with_defaults_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut HashMap<String, Value>),
  {
    let mut defaults = self.defaults.write().unwrap();
    f(&mut defaults);
  }

  pub fn with_cmd_lines_read_lock<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&HashMap<String, Value>) -> R,
  {
    let cmd_lines = self.cmd_lines.read().unwrap();
    f(&cmd_lines)
  }

  pub fn with_cmd_lines_write_lock<F>(&self, f: F)
  where
    F: FnOnce(&mut HashMap<String, Value>),
  {
    let mut cmd_lines = self.cmd_lines.write().unwrap();
    f(&mut cmd_lines);
  }

  fn notify_listeners(
    &self,
    key: &str,
    prev_value: &Option<Value>,
    prev_source: &SettingSource,
    new_value: &Option<Value>,
    new_source: &SettingSource,
  ) {
    let lock = self.listeners.read().unwrap();
    for listener in lock.iter() {
      listener.on_change(key, prev_value, prev_source, new_value, new_source);
    }
  }
}

impl SettingService for DefaultSettingService {
  fn load(&self, path: &Path) {
    self.env_wrapper.load(path);
  }

  fn home_dir(&self) -> Option<PathBuf> {
    self.env_wrapper.home_dir()
  }

  fn get_env(&self, key: &str) -> Option<String> {
    self.env_wrapper.var(key).ok()
  }

  fn set_setting_with_source(&self, key: &str, value: &Value, source: SettingSource) {
    let (prev_value, prev_source) = self.get_setting_value_with_source(key);
    match source {
      SettingSource::CommandLine => {
        self.with_cmd_lines_write_lock(|cmd_lines| {
          cmd_lines.insert(key.to_string(), value.clone());
        });
      }
      SettingSource::Environment => {
        tracing::error!("SettingSource::Environment is not supported for override");
      }
      SettingSource::SettingsFile => {
        self.with_settings_write_lock(|settings| {
          settings.insert(key.into(), value.clone());
        });
        let (cur_value, cur_source) = self.get_setting_value_with_source(key);
        self.notify_listeners(key, &prev_value, &prev_source, &cur_value, &cur_source);
      }
      SettingSource::Default => {
        self.with_defaults_write_lock(|defaults| {
          defaults.insert(key.to_string(), value.clone());
        });
      }
      SettingSource::System => {
        tracing::error!("SettingSource::System is not supported for override");
      }
    }
  }

  fn delete_setting(&self, key: &str) -> Result<()> {
    let (prev_value, prev_source) = self.get_setting_value_with_source(key);
    self.with_settings_write_lock(|settings| {
      settings.remove(key);
    });
    let (cur_value, cur_source) = self.get_setting_value_with_source(key);
    self.notify_listeners(key, &prev_value, &prev_source, &cur_value, &cur_source);
    Ok(())
  }

  fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource) {
    if let Some(setting) = self.system_settings.iter().find(|s| s.key == key) {
      return (Some(setting.value.clone()), SettingSource::System);
    }

    let metadata = self.get_setting_metadata(key);
    let result = self.with_cmd_lines_read_lock(|cmd_lines| cmd_lines.get(key).cloned());
    if let Some(value) = result {
      return (Some(value), SettingSource::CommandLine);
    }
    if let Ok(value) = self.env_wrapper.var(key) {
      let value = metadata.parse(Value::String(value));
      return (Some(value), SettingSource::Environment);
    }
    let result = self.with_settings_read_lock(|settings| settings.get(key).cloned());
    result
      .map(|value| (Some(metadata.parse(value)), SettingSource::SettingsFile))
      .unwrap_or((self.get_default_value(key), SettingSource::Default))
  }

  fn list(&self) -> Vec<SettingInfo> {
    let mut system_settings = self
      .system_settings
      .iter()
      .map(|s| SettingInfo {
        key: s.key.clone(),
        current_value: s.value.clone(),
        default_value: s.value.clone(),
        source: s.source.clone(),
        metadata: self.get_setting_metadata(&s.key),
      })
      .collect::<Vec<SettingInfo>>();
    let mut app_settings = SETTING_VARS
      .iter()
      .map(|key| {
        let (current_value, source) = self.get_setting_value_with_source(key);
        let metadata = self.get_setting_metadata(key);
        let current_value = current_value.map(|value| metadata.parse(value));

        SettingInfo {
          key: key.to_string(),
          current_value: current_value.unwrap_or(Value::Null),
          default_value: self.get_default_value(key).unwrap_or(Value::Null),
          source,
          metadata,
        }
      })
      .collect::<Vec<SettingInfo>>();

    // Add variant-specific server args settings
    let variants = self.exec_variants();
    for variant in variants {
      let variant_key = format!("BODHI_LLAMACPP_ARGS_{}", variant.to_uppercase());
      let (current_value, source) = self.get_setting_value_with_source(&variant_key);
      let metadata = self.get_setting_metadata(&variant_key);
      let current_value = current_value.map(|value| metadata.parse(value));

      app_settings.push(SettingInfo {
        key: variant_key.clone(),
        current_value: current_value.unwrap_or(Value::Null),
        default_value: self.get_default_value(&variant_key).unwrap_or(Value::Null),
        source,
        metadata,
      });
    }

    system_settings.extend(app_settings);
    system_settings
  }

  fn get_setting_metadata(&self, key: &str) -> SettingMetadata {
    match key {
      BODHI_PORT | BODHI_PUBLIC_PORT => SettingMetadata::Number { min: 1, max: 65535 },
      BODHI_LOG_LEVEL => SettingMetadata::option(
        ["error", "warn", "info", "debug", "trace"]
          .iter()
          .map(|s| s.to_string())
          .collect(),
      ),
      BODHI_LOG_STDOUT => SettingMetadata::Boolean,
      BODHI_EXEC_VARIANT => {
        let variants = self.exec_variants();
        SettingMetadata::option(variants)
      }
      BODHI_EXEC_TARGET => SettingMetadata::String,
      BODHI_EXEC_NAME => SettingMetadata::String,
      BODHI_EXEC_VARIANTS => SettingMetadata::String,
      BODHI_KEEP_ALIVE_SECS => SettingMetadata::Number {
        min: 300,
        max: 86400,
      },
      BODHI_CANONICAL_REDIRECT => SettingMetadata::Boolean,
      BODHI_LLAMACPP_ARGS => SettingMetadata::String,
      key if key.starts_with("BODHI_LLAMACPP_ARGS_") => SettingMetadata::String,
      _ => SettingMetadata::String,
    }
  }

  fn get_default_value(&self, key: &str) -> Option<Value> {
    self.with_defaults_read_lock(|defaults| match key {
      BODHI_HOME => match defaults.get(BODHI_HOME).cloned() {
        Some(value) => Some(value),
        None => {
          let result = self
            .home_dir()
            .map(|home_dir| home_dir.join(".cache").join("bodhi"))
            .map(|path| Value::String(path.display().to_string()));
          result
        }
      },
      BODHI_PUBLIC_HOST => self.get_setting_value(BODHI_HOST),
      BODHI_PUBLIC_SCHEME => self.get_setting_value(BODHI_SCHEME),
      BODHI_PUBLIC_PORT => self.get_setting_value(BODHI_PORT),
      _ => defaults.get(key).cloned(),
    })
  }

  fn add_listener(&self, listener: Arc<dyn SettingsChangeListener>) {
    let mut listeners = self.listeners.write().unwrap();
    if !listeners
      .iter()
      .any(|existing| std::ptr::eq(existing.as_ref(), listener.as_ref()))
    {
      listeners.push(listener);
    }
  }
}

asref_impl!(SettingService, DefaultSettingService);
