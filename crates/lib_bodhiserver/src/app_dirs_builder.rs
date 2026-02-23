use crate::app_options::AppOptions;
use crate::BootstrapError;
use crate::BootstrapService;
use objs::{AppCommand, EnvType, Setting, SettingMetadata, SettingSource};
use serde_yaml::Value;
use services::{
  EnvWrapper, BODHI_APP_TYPE, BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_COMMIT_SHA, BODHI_ENV_TYPE,
  BODHI_HOME, BODHI_VERSION, SETTINGS_YAML,
};
use std::{collections::HashMap, env, fs, path::PathBuf, sync::Arc};
use tracing::{info, warn};

const DEFAULTS_YAML: &str = "defaults.yaml";

/// Primary entry point for setting up all application directories and configuration.
/// Returns the bodhi home path, setting source, and file defaults for use with
/// `setup_bootstrap_service`.
pub fn setup_app_dirs(
  options: &AppOptions,
) -> Result<(PathBuf, SettingSource, HashMap<String, Value>), BootstrapError> {
  let file_defaults = load_defaults_yaml();
  let (bodhi_home, source) = create_bodhi_home(
    options.env_wrapper.clone(),
    &options.env_type,
    &file_defaults,
  )?;
  Ok((bodhi_home, source, file_defaults))
}

/// Creates the main Bodhi home directory if it doesn't exist.
/// Returns the path and source (environment or default).
fn create_bodhi_home(
  env_wrapper: Arc<dyn EnvWrapper>,
  env_type: &EnvType,
  file_defaults: &HashMap<String, Value>,
) -> Result<(PathBuf, SettingSource), BootstrapError> {
  let (bodhi_home, source) = find_bodhi_home(env_wrapper, env_type, file_defaults)?;
  if !bodhi_home.exists() {
    fs::create_dir_all(&bodhi_home).map_err(|err| BootstrapError::DirCreate {
      source: err,
      path: format!("$BODHI_HOME={}", &bodhi_home.display()),
    })?;
  }
  Ok((bodhi_home, source))
}

/// Sets up the bootstrap service with system defaults and loads environment variables.
pub fn setup_bootstrap_service(
  options: &AppOptions,
  bodhi_home: PathBuf,
  source: SettingSource,
  file_defaults: HashMap<String, Value>,
  command: AppCommand,
) -> Result<BootstrapService, BootstrapError> {
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let app_version = file_defaults
    .get(BODHI_VERSION)
    .map(|v| v.as_str())
    .unwrap_or(None);
  let app_commit_sha = file_defaults
    .get(BODHI_COMMIT_SHA)
    .map(|v| v.as_str())
    .unwrap_or(None);
  let system_settings =
    build_system_settings(options, app_version, app_commit_sha, &bodhi_home, &source);
  let bootstrap_service = BootstrapService::new(
    options.env_wrapper.clone(),
    system_settings,
    file_defaults,
    settings_file,
    options.app_settings.clone(),
    command,
  )?;
  Ok(bootstrap_service)
}

/// Builds the system settings that are injected into the settings service.
fn build_system_settings(
  options: &AppOptions,
  app_version: Option<&str>,
  app_commit_sha: Option<&str>,
  bodhi_home: &PathBuf,
  source: &SettingSource,
) -> Vec<Setting> {
  vec![
    Setting {
      key: BODHI_HOME.to_string(),
      value: Value::String(bodhi_home.display().to_string()),
      source: source.clone(),
      metadata: SettingMetadata::String,
    },
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
) -> Result<(PathBuf, SettingSource), BootstrapError> {
  let value = env_wrapper.var(BODHI_HOME);
  let bodhi_home = match value {
    Ok(value) => (PathBuf::from(value), SettingSource::Environment),
    Err(_) => {
      if let Some(file_value) = file_defaults.get(BODHI_HOME) {
        if let Some(path_str) = file_value.as_str() {
          return Ok((PathBuf::from(path_str), SettingSource::Default));
        }
      }

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
        None => return Err(BootstrapError::BodhiHomeNotResolved),
      }
    }
  };
  Ok(bodhi_home)
}

#[cfg(test)]
#[path = "test_app_dirs_builder.rs"]
mod test_app_dirs_builder;
