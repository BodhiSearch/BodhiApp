use crate::app_options::AppOptions;
use crate::AppDirsBuilderError;
use objs::{EnvType, Setting, SettingMetadata, SettingSource};
use serde_yaml::Value;
use services::{
  DefaultSettingService, EnvWrapper, SettingService, BODHI_APP_TYPE, BODHI_AUTH_REALM,
  BODHI_AUTH_URL, BODHI_COMMIT_SHA, BODHI_ENV_TYPE, BODHI_HOME, BODHI_VERSION, HF_HOME,
  SETTINGS_YAML,
};
use std::{
  collections::HashMap,
  env,
  fs::{self, File},
  path::PathBuf,
  sync::Arc,
};
use tracing::{info, warn};

const DEFAULTS_YAML: &str = "defaults.yaml";

/// Primary entry point for setting up all application directories and configuration.
/// This function orchestrates the complete initialization process.
pub fn setup_app_dirs(options: &AppOptions) -> Result<DefaultSettingService, AppDirsBuilderError> {
  let file_defaults = load_defaults_yaml();
  let (bodhi_home, source) = create_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  )?;
  let setting_service = setup_settings(options, bodhi_home, source, file_defaults)?;
  setup_bodhi_subdirs(&setting_service)?;
  setup_hf_home(&setting_service)?;
  setup_logs_dir(&setting_service)?;
  Ok(setting_service)
}

/// Creates the main Bodhi home directory if it doesn't exist.
/// Returns the path and source (environment or default).
fn create_bodhi_home(
  env_wrapper: Arc<dyn EnvWrapper>,
  env_type: &EnvType,
  file_defaults: &HashMap<String, Value>,
) -> Result<(PathBuf, SettingSource), AppDirsBuilderError> {
  let (bodhi_home, source) = find_bodhi_home(env_wrapper, env_type, file_defaults)?;
  if !bodhi_home.exists() {
    fs::create_dir_all(&bodhi_home).map_err(|err| AppDirsBuilderError::DirCreate {
      source: err,
      path: format!("$BODHI_HOME={}", &bodhi_home.display()),
    })?;
  }
  Ok((bodhi_home, source))
}

/// Sets up the settings service with system defaults and loads environment variables.
fn setup_settings(
  options: &AppOptions,
  bodhi_home: PathBuf,
  source: SettingSource,
  file_defaults: HashMap<String, Value>,
) -> Result<DefaultSettingService, AppDirsBuilderError> {
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let app_version = file_defaults
    .get(BODHI_VERSION)
    .map(|v| v.as_str())
    .unwrap_or(None);
  let app_commit_sha = file_defaults
    .get(BODHI_COMMIT_SHA)
    .map(|v| v.as_str())
    .unwrap_or(None);
  let app_settings = build_system_settings(options, app_version, app_commit_sha);
  let setting_service = DefaultSettingService::new_with_defaults(
    options.env_wrapper.clone(),
    Setting {
      key: BODHI_HOME.to_string(),
      value: Value::String(bodhi_home.display().to_string()),
      source,
      metadata: SettingMetadata::String,
    },
    app_settings,
    file_defaults,
    settings_file,
  );

  // Load default environment first
  setting_service.load_default_env();

  // Apply app settings from options using the setting service
  for (key, value) in &options.app_settings {
    // Get the metadata for this setting to parse it correctly
    let metadata = setting_service.get_setting_metadata(key);
    let parsed_value = metadata.parse(Value::String(value.clone()));
    setting_service.set_setting_with_source(key, &parsed_value, SettingSource::SettingsFile);
  }

  Ok(setting_service)
}

/// Builds the system settings that are injected into the settings service.
fn build_system_settings(
  options: &AppOptions,
  app_version: Option<&str>,
  app_commit_sha: Option<&str>,
) -> Vec<Setting> {
  vec![
    Setting {
      key: BODHI_ENV_TYPE.to_string(),
      value: serde_yaml::Value::String(options.env_type.to_string()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_APP_TYPE.to_string(),
      value: serde_yaml::Value::String(options.app_type.to_string()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_VERSION.to_string(),
      value: serde_yaml::Value::String(
        app_version
          .unwrap_or(options.app_version.as_str())
          .to_string(),
      ),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_COMMIT_SHA.to_string(),
      value: serde_yaml::Value::String(
        app_commit_sha
          .unwrap_or(options.app_commit_sha.as_str())
          .to_string(),
      ),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_AUTH_URL.to_string(),
      value: serde_yaml::Value::String(options.auth_url.clone()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
    Setting {
      key: BODHI_AUTH_REALM.to_string(),
      value: serde_yaml::Value::String(options.auth_realm.clone()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    },
  ]
}

fn load_defaults_yaml() -> HashMap<String, Value> {
  let exe_path = match env::current_exe() {
    Ok(path) => path,
    Err(err) => {
      warn!(
        ?err,
        "failed to determine executable path for defaults.yaml loading"
      );
      return HashMap::new();
    }
  };

  let exe_dir = match exe_path.parent() {
    Some(dir) => dir,
    None => {
      warn!(exe_path = %exe_path.display(), "executable path has no parent directory");
      return HashMap::new();
    }
  };

  let defaults_path = exe_dir.join(DEFAULTS_YAML);
  if !defaults_path.exists() {
    return HashMap::new();
  }

  match fs::read_to_string(&defaults_path) {
    Ok(contents) => match serde_yaml::from_str::<HashMap<String, Value>>(&contents) {
      Ok(defaults) => {
        info!(path = %defaults_path.display(), count = defaults.len(), "loaded defaults from file");
        defaults
      }
      Err(err) => {
        warn!(?err, path = %defaults_path.display(), "failed to parse defaults.yaml, using hardcoded defaults");
        HashMap::new()
      }
    },
    Err(err) => {
      warn!(?err, path = %defaults_path.display(), "failed to read defaults.yaml, using hardcoded defaults");
      HashMap::new()
    }
  }
}
fn find_bodhi_home(
  env_wrapper: Arc<dyn EnvWrapper>,
  env_type: &EnvType,
  file_defaults: &HashMap<String, Value>,
) -> Result<(PathBuf, SettingSource), AppDirsBuilderError> {
  let value = env_wrapper.var(BODHI_HOME);
  let bodhi_home = match value {
    Ok(value) => (PathBuf::from(value), SettingSource::Environment),
    Err(_) => {
      if let Some(file_value) = file_defaults.get(BODHI_HOME) {
        if let Some(path_str) = file_value.as_str() {
          return Ok((PathBuf::from(path_str), SettingSource::Default));
        }
      }

      // Fall back to computed default
      let home_dir = env_wrapper.home_dir();
      match home_dir {
        Some(home_dir) => {
          let path = if env_type.is_production() {
            "bodhi"
          } else {
            "bodhi-dev"
          };
          (home_dir.join(".cache").join(path), SettingSource::Default)
        }
        None => return Err(AppDirsBuilderError::BodhiHomeNotFound),
      }
    }
  };
  Ok(bodhi_home)
}

/// Sets up subdirectories within the Bodhi home directory (databases).
fn setup_bodhi_subdirs(setting_service: &dyn SettingService) -> Result<(), AppDirsBuilderError> {
  let db_path = setting_service.app_db_path();
  if !db_path.exists() {
    File::create_new(&db_path).map_err(|err| AppDirsBuilderError::IoFileWrite {
      source: err,
      path: db_path.display().to_string(),
    })?;
  }
  let session_db_path = setting_service.session_db_path();
  if !session_db_path.exists() {
    File::create_new(&session_db_path).map_err(|err| AppDirsBuilderError::IoFileWrite {
      source: err,
      path: session_db_path.display().to_string(),
    })?;
  }
  Ok(())
}

/// Sets up HuggingFace home directory and hub subdirectory.
fn setup_hf_home(setting_service: &dyn SettingService) -> Result<PathBuf, AppDirsBuilderError> {
  let hf_home = match setting_service.get_setting(HF_HOME) {
    Some(hf_home) => PathBuf::from(hf_home),
    None => match setting_service.home_dir() {
      Some(home_dir) => {
        let hf_home = home_dir.join(".cache").join("huggingface");
        setting_service.set_setting(HF_HOME, &hf_home.display().to_string());
        hf_home
      }
      None => return Err(AppDirsBuilderError::HfHomeNotFound),
    },
  };
  let hf_hub = hf_home.join("hub");
  if !hf_hub.exists() {
    fs::create_dir_all(&hf_hub).map_err(|err| AppDirsBuilderError::DirCreate {
      source: err,
      path: "$HF_HOME/hub".to_string(),
    })?;
  }
  Ok(hf_home)
}

/// Sets up the logs directory.
fn setup_logs_dir(setting_service: &dyn SettingService) -> Result<PathBuf, AppDirsBuilderError> {
  let logs_dir = setting_service.logs_dir();
  if !logs_dir.exists() {
    fs::create_dir_all(&logs_dir).map_err(|err| AppDirsBuilderError::DirCreate {
      source: err,
      path: logs_dir.display().to_string(),
    })?;
  }
  Ok(logs_dir)
}

#[cfg(test)]
#[path = "test_app_dirs_builder.rs"]
mod test_app_dirs_builder;
