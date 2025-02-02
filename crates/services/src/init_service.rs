use crate::{EnvWrapper, SettingService, BODHI_HOME, HF_HOME};
use objs::{EnvType, SettingSource};
use std::{
  fs::{self, File},
  io,
  path::PathBuf,
  sync::Arc,
};

#[derive(Debug, thiserror::Error)]
pub enum InitServiceError {
  #[error("failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  BodhiHomeNotFound,
  #[error("failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.")]
  HfHomeNotFound,
  #[error("io_error: failed to create directory {path}, error: {source}")]
  DirCreate {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("io_error: failed to update file {path}, error: {source}")]
  IoFileWrite {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("setting_error: failed to update the setting, check $BODHI_HOME/settings.yaml has write permission")]
  SettingServiceError,
}

#[derive(derive_new::new)]
pub struct InitService {
  env_wrapper: Arc<dyn EnvWrapper>,
  env_type: EnvType,
}

impl InitService {
  pub fn setup_bodhi_home_dir(&self) -> Result<(PathBuf, SettingSource), InitServiceError> {
    let (bodhi_home, source) = self.find_bodhi_home()?;
    if !bodhi_home.exists() {
      fs::create_dir_all(&bodhi_home).map_err(|err| InitServiceError::DirCreate {
        source: err,
        path: format!("$BODHI_HOME={}", &bodhi_home.display()),
      })?;
    }
    Ok((bodhi_home, source))
  }

  fn find_bodhi_home(&self) -> Result<(PathBuf, SettingSource), InitServiceError> {
    let value = self.env_wrapper.var(BODHI_HOME);
    let bodhi_home = match value {
      Ok(value) => (PathBuf::from(value), SettingSource::Environment),
      Err(_) => {
        let home_dir = self.env_wrapper.home_dir();
        match home_dir {
          Some(home_dir) => {
            let path = if self.env_type.is_production() {
              "bodhi"
            } else {
              "bodhi-dev"
            };
            (home_dir.join(".cache").join(path), SettingSource::Default)
          }
          None => return Err(InitServiceError::BodhiHomeNotFound),
        }
      }
    };
    Ok(bodhi_home)
  }

  pub fn set_bodhi_home(
    &self,
    setting_service: &dyn SettingService,
  ) -> Result<(), InitServiceError> {
    let alias_home = setting_service.aliases_dir();
    if !alias_home.exists() {
      fs::create_dir_all(&alias_home).map_err(|err| InitServiceError::DirCreate {
        source: err,
        path: alias_home.display().to_string(),
      })?;
    }
    let db_path = setting_service.app_db_path();
    if !db_path.exists() {
      File::create_new(&db_path).map_err(|err| InitServiceError::IoFileWrite {
        source: err,
        path: db_path.display().to_string(),
      })?;
    }
    let session_db_path = setting_service.session_db_path();
    if !session_db_path.exists() {
      File::create_new(&session_db_path).map_err(|err| InitServiceError::IoFileWrite {
        source: err,
        path: session_db_path.display().to_string(),
      })?;
    }
    let models_file = setting_service.models_yaml();
    if !models_file.exists() {
      let contents = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/models.yaml"));
      fs::write(&models_file, contents).map_err(|err| InitServiceError::IoFileWrite {
        source: err,
        path: models_file.display().to_string(),
      })?;
    }
    Ok(())
  }

  pub fn setup_hf_home<T: AsRef<dyn SettingService>>(
    setting_service: T,
  ) -> Result<PathBuf, InitServiceError> {
    let setting_service = setting_service.as_ref();
    let hf_home = match setting_service.get_setting(HF_HOME) {
      Some(hf_home) => PathBuf::from(hf_home),
      None => match setting_service.home_dir() {
        Some(home_dir) => {
          let hf_home = home_dir.join(".cache").join("huggingface");
          setting_service.set_setting(HF_HOME, &hf_home.display().to_string());
          hf_home
        }
        None => return Err(InitServiceError::HfHomeNotFound),
      },
    };
    let hf_hub = hf_home.join("hub");
    if !hf_hub.exists() {
      fs::create_dir_all(&hf_hub).map_err(|err| InitServiceError::DirCreate {
        source: err,
        path: "$HF_HOME/hub".to_string(),
      })?;
    }
    Ok(hf_home)
  }

  pub fn setup_logs_dir<T: AsRef<dyn SettingService>>(
    setting_service: T,
  ) -> Result<PathBuf, InitServiceError> {
    let setting_service = setting_service.as_ref();
    let logs_dir = setting_service.logs_dir();
    if !logs_dir.exists() {
      std::fs::create_dir_all(&logs_dir).map_err(|err| InitServiceError::DirCreate {
        source: err,
        path: logs_dir.display().to_string(),
      })?;
    }
    Ok(logs_dir)
  }
}

#[cfg(test)]
mod tests {
  use super::{InitService, InitServiceError};
  use crate::{
    test_utils::{
      EnvWrapperStub, TEST_ALIASES_DIR, TEST_MODELS_YAML, TEST_PROD_DB, TEST_SESSION_DB,
      TEST_SETTINGS_YAML,
    },
    DefaultSettingService, MockEnvWrapper, MockSettingService, SettingService, BODHI_HOME, HF_HOME,
  };
  use mockall::predicate::eq;
  use objs::{
    test_utils::{empty_bodhi_home, temp_dir},
    EnvType, Setting, SettingMetadata, SettingSource,
  };
  use rstest::rstest;
  use serde_yaml::Value;
  use std::{env::VarError, sync::Arc};
  use tempfile::TempDir;

  #[rstest]
  #[case(InitServiceError::BodhiHomeNotFound,
    "failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  #[case(InitServiceError::HfHomeNotFound,
    "failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.")]
  fn test_init_service_error_messages(#[case] error: InitServiceError, #[case] message: String) {
    assert_eq!(message, error.to_string());
  }

  #[rstest]
  fn test_init_service_bodhi_home_from_env(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();
    let env_wrapper = Arc::new(EnvWrapperStub::new(maplit::hashmap! {
      BODHI_HOME.to_string() => bodhi_home_str.clone(),
    }));
    let init_service = InitService::new(env_wrapper.clone(), EnvType::Development);
    let (result, source) = init_service.setup_bodhi_home_dir()?;
    assert_eq!(bodhi_home, result);
    assert_eq!(SettingSource::Environment, source);
    Ok(())
  }

  #[rstest]
  fn test_init_service_bodhi_home_from_home_dir(temp_dir: TempDir) -> anyhow::Result<()> {
    let env_wrapper = Arc::new(EnvWrapperStub::new(maplit::hashmap! {
      "HOME".to_string() => temp_dir.path().display().to_string(),
    }));
    let init_service = InitService::new(env_wrapper.clone(), EnvType::Development);
    let (result, source) = init_service.setup_bodhi_home_dir()?;
    assert_eq!(temp_dir.path().join(".cache").join("bodhi-dev"), result);
    assert_eq!(SettingSource::Default, source);
    Ok(())
  }

  #[rstest]
  fn test_init_service_fails_if_not_able_to_find_bodhi_home() -> anyhow::Result<()> {
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
    let init_service = InitService::new(Arc::new(mock_env_wrapper), EnvType::Development);
    let result = init_service.setup_bodhi_home_dir();
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      InitServiceError::BodhiHomeNotFound
    ));
    Ok(())
  }

  #[rstest]
  fn test_init_service_creates_home_dirs(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();
    let env_wrapper = Arc::new(EnvWrapperStub::new(maplit::hashmap! {
      BODHI_HOME.to_string() => bodhi_home_str.clone(),
    }));
    let init_service = InitService::new(env_wrapper.clone(), EnvType::Development);
    let settings_service = DefaultSettingService::new_with_defaults(
      env_wrapper,
      Setting {
        key: BODHI_HOME.to_string(),
        value: Value::String(bodhi_home_str),
        source: SettingSource::Default,
        metadata: SettingMetadata::String,
      },
      vec![],
      bodhi_home.join(TEST_SETTINGS_YAML),
    )?;
    init_service.set_bodhi_home(&settings_service)?;
    assert!(bodhi_home.join(TEST_ALIASES_DIR).exists());
    assert!(bodhi_home.join(TEST_PROD_DB).exists());
    assert!(bodhi_home.join(TEST_SESSION_DB).exists());
    assert!(bodhi_home.join(TEST_MODELS_YAML).exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_hf_home_when_setting_exists(temp_dir: TempDir) -> anyhow::Result<()> {
    let hf_home = temp_dir.path().join("hf_home");
    let mut mock = MockSettingService::default();
    mock
      .expect_get_setting()
      .with(eq(HF_HOME))
      .times(1)
      .return_const(Some(hf_home.display().to_string()));

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let result = InitService::setup_hf_home(&setting_service)?;

    assert_eq!(result, hf_home);
    assert!(hf_home.join("hub").exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_hf_home_when_setting_and_home_dir_missing() -> anyhow::Result<()> {
    let mut mock = MockSettingService::default();
    mock
      .expect_get_setting()
      .with(eq(HF_HOME))
      .times(1)
      .return_const(None);
    mock.expect_home_dir().times(1).return_const(None);

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let result = InitService::setup_hf_home(&setting_service);

    assert!(matches!(result, Err(InitServiceError::HfHomeNotFound)));
    Ok(())
  }

  #[rstest]
  fn test_setup_hf_home_when_setting_missing_but_home_dir_exists(
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let home_dir = temp_dir.path().to_path_buf();
    let expected_hf_home = home_dir.join(".cache").join("huggingface");
    let mut mock = MockSettingService::default();

    mock
      .expect_get_setting()
      .with(eq(HF_HOME))
      .times(1)
      .return_const(None);
    mock.expect_home_dir().times(1).return_const(Some(home_dir));
    mock
      .expect_set_setting()
      .with(eq(HF_HOME), eq(expected_hf_home.display().to_string()))
      .times(1)
      .return_once(|_, _| ());

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let result = InitService::setup_hf_home(&setting_service)?;

    assert_eq!(expected_hf_home, result);
    assert!(expected_hf_home.join("hub").exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_logs_dir(temp_dir: TempDir) -> anyhow::Result<()> {
    let logs_dir = temp_dir.path().join("logs");
    let mut mock = MockSettingService::default();
    mock
      .expect_logs_dir()
      .times(1)
      .return_const(logs_dir.clone());

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let result = InitService::setup_logs_dir(&setting_service)?;

    assert_eq!(result, logs_dir);
    assert!(logs_dir.exists());
    Ok(())
  }
}
