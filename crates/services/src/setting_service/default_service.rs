use super::{
  BootstrapParts, Result, SettingService, SettingServiceError, SettingsChangeListener,
  BODHI_CANONICAL_REDIRECT, BODHI_DEPLOYMENT, BODHI_EXEC_NAME, BODHI_EXEC_TARGET, BODHI_EXEC_VARIANT,
  BODHI_EXEC_VARIANTS, BODHI_HOME, BODHI_HOST, BODHI_KEEP_ALIVE_SECS, BODHI_LLAMACPP_ARGS,
  BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT, BODHI_PORT, BODHI_PUBLIC_HOST, BODHI_PUBLIC_PORT,
  BODHI_PUBLIC_SCHEME, BODHI_SCHEME, BODHI_SESSION_DB_URL, DEFAULT_CANONICAL_REDIRECT, DEFAULT_HOST,
  DEFAULT_KEEP_ALIVE_SECS, DEFAULT_LOG_LEVEL, DEFAULT_LOG_STDOUT, DEFAULT_PORT, DEFAULT_SCHEME,
  HF_HOME, LOGS_DIR, SESSION_DB, SETTING_VARS,
};
use crate::db::{DbSetting, SettingsRepository};
use crate::{asref_impl, EnvWrapper};
use objs::{AppCommand, Setting, SettingInfo, SettingMetadata, SettingSource};
use serde_yaml::Value;
use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
  sync::{Arc, RwLock},
};

pub struct DefaultSettingService {
  env_wrapper: Arc<dyn EnvWrapper>,
  system_settings: Vec<Setting>,
  cmd_lines: RwLock<HashMap<String, Value>>,
  settings_file_values: RwLock<HashMap<String, Value>>,
  defaults: RwLock<HashMap<String, Value>>,
  listeners: RwLock<Vec<Arc<dyn SettingsChangeListener>>>,
  db_service: Arc<dyn SettingsRepository>,
}

impl std::fmt::Debug for DefaultSettingService {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DefaultSettingService")
      .field("system_settings", &self.system_settings)
      .finish_non_exhaustive()
  }
}

impl DefaultSettingService {
  fn setting_metadata_static(key: &str) -> SettingMetadata {
    match key {
      BODHI_PORT | BODHI_PUBLIC_PORT => SettingMetadata::Number { min: 1, max: 65535 },
      BODHI_LOG_LEVEL => SettingMetadata::option(
        ["error", "warn", "info", "debug", "trace"]
          .iter()
          .map(|s| s.to_string())
          .collect(),
      ),
      BODHI_LOG_STDOUT => SettingMetadata::Boolean,
      BODHI_EXEC_VARIANT => SettingMetadata::String,
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

  pub fn from_parts(parts: BootstrapParts, db_service: Arc<dyn SettingsRepository>) -> Self {
    // 1. Load .env from bodhi_home/.env (mutates process env)
    let env_file = parts.bodhi_home.join(".env");
    if env_file.exists() {
      parts.env_wrapper.load(&env_file);
    }

    // 2. Load settings.yaml once into memory
    let mut settings_file_values = load_settings_yaml(&parts.settings_file);

    // 3. Overlay NAPI app_settings onto settings_file_values (app_settings wins)
    for (key, value_str) in &parts.app_settings {
      let metadata = Self::setting_metadata_static(key);
      let parsed = metadata.parse(Value::String(value_str.clone()));
      settings_file_values.insert(key.clone(), parsed);
    }

    // 4. Extract cmd_lines from AppCommand
    let mut cmd_lines = HashMap::new();
    if let AppCommand::Serve { ref host, ref port } = parts.app_command {
      if let Some(h) = host {
        cmd_lines.insert(BODHI_HOST.to_string(), Value::String(h.clone()));
      }
      if let Some(p) = port {
        cmd_lines.insert(BODHI_PORT.to_string(), Value::Number((*p).into()));
      }
    }

    // 5. Build all defaults from file_defaults + hardcoded
    let defaults = build_all_defaults(parts.env_wrapper.as_ref(), &parts.file_defaults, &parts.bodhi_home);

    Self {
      env_wrapper: parts.env_wrapper,
      system_settings: parts.system_settings,
      cmd_lines: RwLock::new(cmd_lines),
      settings_file_values: RwLock::new(settings_file_values),
      defaults: RwLock::new(defaults),
      listeners: RwLock::new(Vec::new()),
      db_service,
    }
  }

  fn is_valid_db_key(&self, key: &str) -> bool {
    SETTING_VARS.contains(&key) || key.starts_with("BODHI_LLAMACPP_ARGS_")
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

fn load_settings_yaml(path: &PathBuf) -> HashMap<String, Value> {
  if !path.exists() {
    return HashMap::new();
  }
  let contents = fs::read_to_string(path).unwrap_or_default();
  let mapping: serde_yaml::Mapping =
    serde_yaml::from_str(&contents).unwrap_or_else(|_| serde_yaml::Mapping::new());
  mapping
    .into_iter()
    .filter_map(|(k, v)| k.as_str().map(|s| (s.to_string(), v)))
    .collect()
}

fn build_all_defaults(
  env_wrapper: &dyn EnvWrapper,
  file_defaults: &HashMap<String, Value>,
  bodhi_home: &Path,
) -> HashMap<String, Value> {
  let mut defaults = HashMap::new();

  // Start with file_defaults as the base
  for (key, value) in file_defaults {
    defaults.insert(key.clone(), value.clone());
  }

  macro_rules! ensure_default {
    ($key:expr, $value:expr) => {
      defaults.entry($key.to_string()).or_insert($value);
    };
  }

  // BODHI_HOME and BODHI_LOGS are set from system_settings, not here
  ensure_default!(
    BODHI_LOG_LEVEL,
    Value::String(DEFAULT_LOG_LEVEL.to_string())
  );
  ensure_default!(BODHI_LOG_STDOUT, Value::Bool(DEFAULT_LOG_STDOUT));
  ensure_default!(BODHI_SCHEME, Value::String(DEFAULT_SCHEME.to_string()));
  ensure_default!(BODHI_HOST, Value::String(DEFAULT_HOST.to_string()));
  ensure_default!(BODHI_PORT, Value::Number(DEFAULT_PORT.into()));
  ensure_default!(
    BODHI_EXEC_TARGET,
    Value::String(llama_server_proc::BUILD_TARGET.to_string())
  );
  ensure_default!(
    BODHI_EXEC_VARIANTS,
    Value::String(
      llama_server_proc::BUILD_VARIANTS
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(","),
    )
  );
  ensure_default!(
    BODHI_EXEC_VARIANT,
    Value::String(llama_server_proc::DEFAULT_VARIANT.to_string())
  );
  ensure_default!(
    BODHI_EXEC_NAME,
    Value::String(llama_server_proc::EXEC_NAME.to_string())
  );
  ensure_default!(
    BODHI_LLAMACPP_ARGS,
    Value::String("--jinja --no-webui".to_string())
  );
  ensure_default!(
    BODHI_KEEP_ALIVE_SECS,
    Value::Number(DEFAULT_KEEP_ALIVE_SECS.into())
  );
  ensure_default!(
    BODHI_CANONICAL_REDIRECT,
    Value::Bool(DEFAULT_CANONICAL_REDIRECT)
  );
  if let Some(home_dir) = env_wrapper.home_dir() {
    defaults.entry(HF_HOME.to_string()).or_insert_with(|| {
      Value::String(
        home_dir
          .join(".cache")
          .join("huggingface")
          .display()
          .to_string(),
      )
    });
  }

  ensure_default!(
    BODHI_SESSION_DB_URL,
    Value::String(format!("sqlite:{}", bodhi_home.join(SESSION_DB).display()))
  );
  ensure_default!(BODHI_DEPLOYMENT, Value::String("standalone".to_string()));

  defaults
}

#[async_trait::async_trait]
impl SettingService for DefaultSettingService {
  async fn load(&self, path: &Path) {
    self.env_wrapper.load(path);
  }

  async fn home_dir(&self) -> Option<PathBuf> {
    self.env_wrapper.home_dir()
  }

  async fn get_env(&self, key: &str) -> Option<String> {
    self.env_wrapper.var(key).ok()
  }

  async fn set_setting_with_source(
    &self,
    key: &str,
    value: &Value,
    source: SettingSource,
  ) -> Result<()> {
    let (prev_value, prev_source) = self.get_setting_value_with_source(key).await;
    match source {
      SettingSource::CommandLine => {
        self.with_cmd_lines_write_lock(|cmd_lines| {
          cmd_lines.insert(key.to_string(), value.clone());
        });
        Ok(())
      }
      SettingSource::Environment | SettingSource::SettingsFile | SettingSource::System => {
        Err(SettingServiceError::InvalidSource)
      }
      SettingSource::Database => {
        if !self.is_valid_db_key(key) {
          tracing::error!("key '{}' is not a valid database setting key", key);
          return Err(SettingServiceError::InvalidKey(key.to_string()));
        }
        let value_str = match value {
          Value::String(s) => s.clone(),
          Value::Number(n) => n.to_string(),
          Value::Bool(b) => b.to_string(),
          _ => serde_yaml::to_string(value)
            .unwrap_or_default()
            .trim()
            .to_string(),
        };
        let metadata = self.get_setting_metadata(key).await;
        let value_type = match metadata {
          SettingMetadata::String => "string",
          SettingMetadata::Number { .. } => "number",
          SettingMetadata::Boolean => "boolean",
          SettingMetadata::Option { .. } => "option",
        };
        let db_setting = DbSetting {
          key: key.to_string(),
          value: value_str,
          value_type: value_type.to_string(),
          created_at: 0,
          updated_at: 0,
        };
        self.db_service.upsert_setting(&db_setting).await?;
        let (cur_value, cur_source) = self.get_setting_value_with_source(key).await;
        self.notify_listeners(key, &prev_value, &prev_source, &cur_value, &cur_source);
        Ok(())
      }
      SettingSource::Default => {
        self.with_defaults_write_lock(|defaults| {
          defaults.insert(key.to_string(), value.clone());
        });
        Ok(())
      }
    }
  }

  async fn delete_setting(&self, key: &str) -> Result<()> {
    let (prev_value, prev_source) = self.get_setting_value_with_source(key).await;
    self.db_service.delete_setting(key).await?;
    let (cur_value, cur_source) = self.get_setting_value_with_source(key).await;
    self.notify_listeners(key, &prev_value, &prev_source, &cur_value, &cur_source);
    Ok(())
  }

  async fn get_setting_value_with_source(&self, key: &str) -> (Option<Value>, SettingSource) {
    if let Some(setting) = self.system_settings.iter().find(|s| s.key == key) {
      return (Some(setting.value.clone()), SettingSource::System);
    }

    let metadata = self.get_setting_metadata(key).await;
    let result = self.with_cmd_lines_read_lock(|cmd_lines| cmd_lines.get(key).cloned());
    if let Some(value) = result {
      return (Some(value), SettingSource::CommandLine);
    }
    if let Ok(value) = self.env_wrapper.var(key) {
      let value = metadata.parse(Value::String(value));
      return (Some(value), SettingSource::Environment);
    }
    if let Ok(Some(db_setting)) = self.db_service.get_setting(key).await {
      let value = metadata.parse(Value::String(db_setting.value));
      return (Some(value), SettingSource::Database);
    }
    let result = self.settings_file_values.read().unwrap().get(key).cloned();
    if let Some(value) = result {
      return (Some(metadata.parse(value)), SettingSource::SettingsFile);
    }
    (self.get_default_value(key).await, SettingSource::Default)
  }

  async fn list(&self) -> Vec<SettingInfo> {
    let mut system_settings = Vec::new();
    for s in &self.system_settings {
      system_settings.push(SettingInfo {
        key: s.key.clone(),
        current_value: s.value.clone(),
        default_value: s.value.clone(),
        source: s.source.clone(),
        metadata: self.get_setting_metadata(&s.key).await,
      });
    }
    let mut app_settings = Vec::new();
    for key in SETTING_VARS {
      let (current_value, source) = self.get_setting_value_with_source(key).await;
      let metadata = self.get_setting_metadata(key).await;
      let current_value = current_value.map(|value| metadata.parse(value));

      app_settings.push(SettingInfo {
        key: key.to_string(),
        current_value: current_value.unwrap_or(Value::Null),
        default_value: self.get_default_value(key).await.unwrap_or(Value::Null),
        source,
        metadata,
      });
    }

    let variants = self.exec_variants().await;
    for variant in variants {
      let variant_key = format!("BODHI_LLAMACPP_ARGS_{}", variant.to_uppercase());
      let (current_value, source) = self.get_setting_value_with_source(&variant_key).await;
      let metadata = self.get_setting_metadata(&variant_key).await;
      let current_value = current_value.map(|value| metadata.parse(value));

      app_settings.push(SettingInfo {
        key: variant_key.clone(),
        current_value: current_value.unwrap_or(Value::Null),
        default_value: self
          .get_default_value(&variant_key)
          .await
          .unwrap_or(Value::Null),
        source,
        metadata,
      });
    }

    system_settings.extend(app_settings);
    system_settings
  }

  async fn get_setting_metadata(&self, key: &str) -> SettingMetadata {
    if key == BODHI_EXEC_VARIANT {
      let variants = self.exec_variants().await;
      return SettingMetadata::option(variants);
    }
    Self::setting_metadata_static(key)
  }

  async fn get_default_value(&self, key: &str) -> Option<Value> {
    match key {
      BODHI_PUBLIC_HOST => return self.get_setting_value(BODHI_HOST).await,
      BODHI_PUBLIC_SCHEME => return self.get_setting_value(BODHI_SCHEME).await,
      BODHI_PUBLIC_PORT => return self.get_setting_value(BODHI_PORT).await,
      _ => {}
    }
    self.with_defaults_read_lock(|defaults| match key {
      BODHI_HOME => match defaults.get(BODHI_HOME).cloned() {
        Some(value) => Some(value),
        None => self
          .env_wrapper
          .home_dir()
          .map(|home_dir| home_dir.join(".cache").join("bodhi"))
          .map(|path| Value::String(path.display().to_string())),
      },
      BODHI_LOGS => match defaults.get(BODHI_LOGS).cloned() {
        Some(value) => Some(value),
        None => {
          let bodhi_home = defaults.get(BODHI_HOME).cloned().or_else(|| {
            self
              .env_wrapper
              .home_dir()
              .map(|home_dir| home_dir.join(".cache").join("bodhi"))
              .map(|path| Value::String(path.display().to_string()))
          });
          bodhi_home.map(|bh| {
            let home = PathBuf::from(bh.as_str().unwrap_or_default());
            Value::String(home.join(LOGS_DIR).display().to_string())
          })
        }
      },
      _ => defaults.get(key).cloned(),
    })
  }

  /// Deduplication is based on Arc pointer equality. Separately allocated Arc
  /// instances wrapping equivalent implementations will not be deduplicated.
  async fn add_listener(&self, listener: Arc<dyn SettingsChangeListener>) {
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
