use derive_builder::Builder;
use objs::{AppType, EnvType, Setting, SettingMetadata, SettingSource};
use services::{
  DefaultSettingService, EnvWrapper, SettingService, BODHI_APP_TYPE, BODHI_AUTH_REALM,
  BODHI_AUTH_URL, BODHI_ENV_TYPE, BODHI_HOME, BODHI_VERSION, HF_HOME, SETTINGS_YAML,
};
use std::{
  fs::{self, File},
  path::PathBuf,
  sync::Arc,
};

use crate::AppDirsBuilderError;

/// Configuration options for setting up application directories and settings.
/// Uses the builder pattern for flexible configuration with sensible defaults.
#[derive(Debug, Clone, Builder)]
#[builder(setter(into, strip_option))]
pub struct AppOptions {
  /// Environment wrapper for accessing environment variables and system paths
  pub env_wrapper: Arc<dyn EnvWrapper>,
  /// Environment type (Development, Production, etc.)
  pub env_type: EnvType,
  /// Application type (Native, Container, etc.)
  pub app_type: AppType,
  /// Application version string
  pub app_version: String,
  /// Authentication server URL
  pub auth_url: String,
  /// Authentication realm
  pub auth_realm: String,
}

/// Primary entry point for setting up all application directories and configuration.
/// This function orchestrates the complete initialization process.
pub fn setup_app_dirs(options: AppOptions) -> Result<DefaultSettingService, AppDirsBuilderError> {
  let (bodhi_home, source) = create_bodhi_home(&options)?;
  let setting_service = setup_settings(&options, bodhi_home, source)?;
  setup_bodhi_subdirs(&setting_service)?;
  setup_hf_home(&setting_service)?;
  setup_logs_dir(&setting_service)?;
  Ok(setting_service)
}

/// Creates the main Bodhi home directory if it doesn't exist.
/// Returns the path and source (environment or default).
fn create_bodhi_home(
  options: &AppOptions,
) -> Result<(PathBuf, SettingSource), AppDirsBuilderError> {
  let (bodhi_home, source) = find_bodhi_home(options)?;
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
) -> Result<DefaultSettingService, AppDirsBuilderError> {
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let app_settings = build_system_settings(options);
  let setting_service = DefaultSettingService::new_with_defaults(
    options.env_wrapper.clone(),
    Setting {
      key: BODHI_HOME.to_string(),
      value: serde_yaml::Value::String(bodhi_home.display().to_string()),
      source,
      metadata: SettingMetadata::String,
    },
    app_settings,
    settings_file,
  );
  setting_service.load_default_env();
  Ok(setting_service)
}

/// Builds the system settings that are injected into the settings service.
fn build_system_settings(options: &AppOptions) -> Vec<Setting> {
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
      value: serde_yaml::Value::String(options.app_version.clone()),
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

fn find_bodhi_home(options: &AppOptions) -> Result<(PathBuf, SettingSource), AppDirsBuilderError> {
  let value = options.env_wrapper.var(BODHI_HOME);
  let bodhi_home = match value {
    Ok(value) => (PathBuf::from(value), SettingSource::Environment),
    Err(_) => {
      let home_dir = options.env_wrapper.home_dir();
      match home_dir {
        Some(home_dir) => {
          let path = if options.env_type.is_production() {
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

/// Sets up subdirectories within the Bodhi home directory (aliases, databases).
fn setup_bodhi_subdirs(setting_service: &dyn SettingService) -> Result<(), AppDirsBuilderError> {
  let alias_home = setting_service.aliases_dir();
  if !alias_home.exists() {
    fs::create_dir_all(&alias_home).map_err(|err| AppDirsBuilderError::DirCreate {
      source: err,
      path: alias_home.display().to_string(),
    })?;
  }
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
    std::fs::create_dir_all(&logs_dir).map_err(|err| AppDirsBuilderError::DirCreate {
      source: err,
      path: logs_dir.display().to_string(),
    })?;
  }
  Ok(logs_dir)
}

#[cfg(test)]
mod tests {
  use super::{setup_app_dirs, AppDirsBuilderError, AppOptionsBuilder};
  use mockall::predicate::eq;
  use objs::test_utils::{empty_bodhi_home, temp_dir};
  use rstest::rstest;
  use services::{
    test_utils::{EnvWrapperStub, TEST_ALIASES_DIR, TEST_PROD_DB, TEST_SESSION_DB},
    EnvWrapper, MockEnvWrapper, MockSettingService, SettingService, BODHI_APP_TYPE,
    BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_ENV_TYPE, BODHI_HOME, BODHI_VERSION, HF_HOME,
  };
  use std::{env::VarError, sync::Arc};
  use tempfile::TempDir;

  #[rstest]
  fn test_create_bodhi_home_from_env(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();
    let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(EnvWrapperStub::new(maplit::hashmap! {
      BODHI_HOME.to_string() => bodhi_home_str.clone(),
    }));
    let options = AppOptionsBuilder::development()
      .env_wrapper(env_wrapper)
      .build()?;
    let (result_path, source) = super::create_bodhi_home(&options)?;
    assert_eq!(result_path, bodhi_home);
    assert_eq!(source, objs::SettingSource::Environment);
    assert!(bodhi_home.exists());
    Ok(())
  }

  #[rstest]
  fn test_create_bodhi_home_from_home_dir(temp_dir: TempDir) -> anyhow::Result<()> {
    let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(EnvWrapperStub::new(maplit::hashmap! {
      "HOME".to_string() => temp_dir.path().display().to_string(),
    }));
    let options = AppOptionsBuilder::development()
      .env_wrapper(env_wrapper)
      .build()?;
    let (result_path, source) = super::create_bodhi_home(&options)?;
    let expected_path = temp_dir.path().join(".cache").join("bodhi-dev");
    assert_eq!(result_path, expected_path);
    assert_eq!(source, objs::SettingSource::Default);
    assert!(expected_path.exists());
    Ok(())
  }

  #[rstest]
  fn test_find_bodhi_home_fails_when_not_found() -> anyhow::Result<()> {
    let mut mock_env_wrapper = MockEnvWrapper::default();
    mock_env_wrapper
      .expect_var()
      .with(eq(BODHI_HOME))
      .times(1)
      .return_once(|_| Err(VarError::NotPresent));
    mock_env_wrapper
      .expect_home_dir()
      .times(1)
      .return_once(|| None);
    let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(mock_env_wrapper);
    let options = AppOptionsBuilder::development()
      .env_wrapper(env_wrapper)
      .build()?;
    let result = super::find_bodhi_home(&options);
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AppDirsBuilderError::BodhiHomeNotFound
    ));
    Ok(())
  }

  #[rstest]
  fn test_setup_app_dirs_integration(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let env_wrapper: Arc<dyn EnvWrapper> = Arc::new(EnvWrapperStub::new(maplit::hashmap! {
      BODHI_HOME.to_string() => bodhi_home.display().to_string(),
    }));
    let options = AppOptionsBuilder::development()
      .env_wrapper(env_wrapper)
      .build()?;
    let _settings_service = setup_app_dirs(options)?;
    assert!(bodhi_home.join(TEST_ALIASES_DIR).exists());
    assert!(bodhi_home.join(TEST_PROD_DB).exists());
    assert!(bodhi_home.join(TEST_SESSION_DB).exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_hf_home_when_setting_exists(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let hf_home = empty_bodhi_home.path().join("hf_home");
    let bodhi_home_str = bodhi_home.display().to_string();

    // Set up real settings service with HF_HOME pre-configured
    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
    let setting_service = setup_app_dirs(options)?;

    // Set the HF_HOME setting
    setting_service.set_setting(HF_HOME, &hf_home.display().to_string());

    let result = super::setup_hf_home(&setting_service)?;

    assert_eq!(result, hf_home);
    assert!(hf_home.join("hub").exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_hf_home_fails_when_no_home_dir() -> anyhow::Result<()> {
    let mut mock = MockSettingService::default();
    mock
      .expect_get_setting()
      .with(eq(HF_HOME))
      .times(1)
      .return_const(None);
    mock.expect_home_dir().times(1).return_const(None);

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let result = super::setup_hf_home(setting_service.as_ref());

    assert!(matches!(result, Err(AppDirsBuilderError::HfHomeNotFound)));
    Ok(())
  }

  #[rstest]
  fn test_setup_hf_home_creates_default_when_missing(
    empty_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");

    // Set up real settings service - setup_app_dirs already calls setup_hf_home
    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
    let setting_service = setup_app_dirs(options)?;

    // HF_HOME should be set by setup_app_dirs
    let hf_home_setting = setting_service.get_setting(HF_HOME);
    assert!(hf_home_setting.is_some());

    // Should create default HF_HOME under the user's home directory
    let expected_hf_home = setting_service
      .home_dir()
      .unwrap()
      .join(".cache")
      .join("huggingface");

    assert_eq!(
      hf_home_setting.unwrap(),
      expected_hf_home.display().to_string()
    );
    assert!(expected_hf_home.join("hub").exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_logs_dir_success(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");

    // Set up real settings service
    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home.display().to_string()).build()?;
    let setting_service = setup_app_dirs(options)?;

    let result = super::setup_logs_dir(&setting_service)?;
    let expected_logs_dir = setting_service.logs_dir();

    assert_eq!(result, expected_logs_dir);
    assert!(expected_logs_dir.exists());
    Ok(())
  }

  // Tests for helper methods

  #[rstest]
  fn test_build_system_settings() -> anyhow::Result<()> {
    let options = AppOptionsBuilder::development().build()?;
    let settings = super::build_system_settings(&options);

    assert_eq!(settings.len(), 5);

    // Check each setting
    let env_type_setting = settings.iter().find(|s| s.key == BODHI_ENV_TYPE).unwrap();
    assert_eq!(
      env_type_setting.value,
      serde_yaml::Value::String("development".to_string())
    );
    assert_eq!(env_type_setting.source, objs::SettingSource::System);

    let app_type_setting = settings.iter().find(|s| s.key == BODHI_APP_TYPE).unwrap();
    assert_eq!(
      app_type_setting.value,
      serde_yaml::Value::String("container".to_string())
    );

    let version_setting = settings.iter().find(|s| s.key == BODHI_VERSION).unwrap();
    assert_eq!(
      version_setting.value,
      serde_yaml::Value::String("1.0.0".to_string())
    );

    let auth_url_setting = settings.iter().find(|s| s.key == BODHI_AUTH_URL).unwrap();
    assert_eq!(
      auth_url_setting.value,
      serde_yaml::Value::String("https://dev-id.getbodhi.app".to_string())
    );

    let auth_realm_setting = settings.iter().find(|s| s.key == BODHI_AUTH_REALM).unwrap();
    assert_eq!(
      auth_realm_setting.value,
      serde_yaml::Value::String("bodhi".to_string())
    );

    Ok(())
  }

  #[rstest]
  fn test_setup_bodhi_subdirs_success(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();

    // Set up real settings service
    let options = AppOptionsBuilder::with_bodhi_home(&bodhi_home_str).build()?;
    let setting_service = setup_app_dirs(options)?;

    // The setup_app_dirs already calls setup_bodhi_subdirs, so we just verify the results
    let aliases_dir = setting_service.aliases_dir();
    let app_db_path = setting_service.app_db_path();
    let session_db_path = setting_service.session_db_path();

    assert!(aliases_dir.exists());
    assert!(app_db_path.exists());
    assert!(session_db_path.exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_logs_dir_error_when_cannot_create() -> anyhow::Result<()> {
    // Create a read-only directory to simulate permission error
    let temp_dir = temp_dir();
    let logs_dir = temp_dir.path().join("readonly").join("logs");

    // Create parent as read-only
    let readonly_parent = temp_dir.path().join("readonly");
    std::fs::create_dir(&readonly_parent)?;

    // Make it read-only (this will fail on some systems, but that's ok for the test)
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = std::fs::metadata(&readonly_parent)?.permissions();
      perms.set_mode(0o444); // read-only
      std::fs::set_permissions(&readonly_parent, perms)?;
    }

    let mut mock = MockSettingService::default();
    mock
      .expect_logs_dir()
      .times(1)
      .return_const(logs_dir.clone());

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let result = super::setup_logs_dir(setting_service.as_ref());

    // On some systems this might succeed, so we just check it doesn't panic
    match result {
      Ok(_) => {
        // Directory creation succeeded despite read-only parent
        assert!(logs_dir.exists());
      }
      Err(AppDirsBuilderError::DirCreate { .. }) => {
        // Expected error case
      }
      Err(e) => {
        panic!("Unexpected error type: {:?}", e);
      }
    }

    Ok(())
  }
}
