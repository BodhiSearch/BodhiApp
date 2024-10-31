use crate::{
  EnvWrapper, SettingService, ALIASES_DIR, BODHI_HOME, BODHI_LOGS, HF_HOME, LOGS_DIR, MODELS_YAML,
  PROD_DB,
};
use objs::EnvType;
use std::{
  fs::{self, File},
  io,
  path::{Path, PathBuf},
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
pub struct InitService<'a> {
  env_wrapper: &'a dyn EnvWrapper,
  env_type: &'a EnvType,
}

impl InitService<'_> {
  pub fn setup_bodhi_home(self) -> Result<PathBuf, InitServiceError> {
    let bodhi_home = self.find_bodhi_home()?;
    self.create_bodhi_home_dirs(&bodhi_home)?;
    Ok(bodhi_home)
  }

  fn find_bodhi_home(&self) -> Result<PathBuf, InitServiceError> {
    let value = self.env_wrapper.var(BODHI_HOME);
    let bodhi_home = match value {
      Ok(value) => PathBuf::from(value),
      Err(_) => {
        let home_dir = self.env_wrapper.home_dir();
        match home_dir {
          Some(home_dir) => {
            let path = if self.env_type.is_production() {
              "bodhi"
            } else {
              "bodhi-dev"
            };
            home_dir.join(".cache").join(path)
          }
          None => return Err(InitServiceError::BodhiHomeNotFound),
        }
      }
    };
    Ok(bodhi_home)
  }

  fn create_bodhi_home_dirs(&self, bodhi_home: &Path) -> Result<(), InitServiceError> {
    if !bodhi_home.exists() {
      fs::create_dir_all(bodhi_home).map_err(|err| InitServiceError::DirCreate {
        source: err,
        path: format!("$BODHI_HOME={}", bodhi_home.display()),
      })?;
    }
    let alias_home = bodhi_home.join(ALIASES_DIR);
    if !alias_home.exists() {
      fs::create_dir_all(&alias_home).map_err(|err| InitServiceError::DirCreate {
        source: err,
        path: "$BODHI_HOME/aliases".to_string(),
      })?;
    }
    let db_path = bodhi_home.join(PROD_DB);
    if !db_path.exists() {
      File::create_new(&db_path).map_err(|err| InitServiceError::IoFileWrite {
        source: err,
        path: format!("$BODHI_HOME/{}", PROD_DB),
      })?;
    }
    let models_file = bodhi_home.join(MODELS_YAML);
    if !models_file.exists() {
      let contents = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/models.yaml"));
      fs::write(&models_file, contents).map_err(|err| InitServiceError::IoFileWrite {
        source: err,
        path: format!("$BODHI_HOME/{}", MODELS_YAML),
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
          setting_service
            .set_setting(HF_HOME, &hf_home.display().to_string())
            .map_err(|_| InitServiceError::SettingServiceError)?;
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
    bodhi_home: &Path,
  ) -> Result<PathBuf, InitServiceError> {
    let setting_service = setting_service.as_ref();
    let logs_dir = match setting_service.get_setting(BODHI_LOGS) {
      Some(logs_dir) => PathBuf::from(logs_dir),
      None => {
        let logs_dir = bodhi_home.join(LOGS_DIR);
        setting_service
          .set_setting(BODHI_LOGS, &logs_dir.display().to_string())
          .map_err(|_| InitServiceError::SettingServiceError)?;
        logs_dir
      }
    };
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
    MockEnvWrapper, SettingService, ALIASES_DIR, BODHI_HOME, BODHI_LOGS, LOGS_DIR, MODELS_YAML,
    PROD_DB,
  };
  use crate::{MockSettingService, HF_HOME};
  use mockall::predicate::eq;
  use objs::{
    test_utils::{empty_bodhi_home, temp_dir},
    EnvType,
  };
  use rstest::rstest;
  use std::env::VarError;
  use std::sync::Arc;
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
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(move |_| Ok(bodhi_home_str.clone()));
    let init_service = InitService::new(&mock, &EnvType::Development);
    let result = init_service.setup_bodhi_home()?;
    assert_eq!(bodhi_home, result);
    Ok(())
  }

  #[rstest]
  fn test_init_service_bodhi_home_from_home_dir(temp_dir: TempDir) -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(move |_| Err(VarError::NotPresent));
    let home_dir = temp_dir.path().to_path_buf();
    mock.expect_home_dir().times(1).return_const(Some(home_dir));
    let init_service = InitService::new(&mock, &EnvType::Development);
    let result = init_service.setup_bodhi_home()?;
    assert_eq!(temp_dir.path().join(".cache").join("bodhi-dev"), result);
    Ok(())
  }

  #[rstest]
  fn test_init_service_fails_if_not_able_to_find_bodhi_home() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(|_| Err(VarError::NotPresent));
    mock.expect_home_dir().returning(move || None);

    let init_service = InitService::new(&mock, &EnvType::Development);
    let result = init_service.setup_bodhi_home();
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
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(move |_| Ok(bodhi_home_str.clone()));
    let init_service = InitService::new(&mock, &EnvType::Development);
    init_service.create_bodhi_home_dirs(&bodhi_home)?;
    assert!(bodhi_home.join(ALIASES_DIR).exists());
    assert!(bodhi_home.join(PROD_DB).exists());
    assert!(bodhi_home.join(MODELS_YAML).exists());
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
      .return_once(|_, _| Ok(()));

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let result = InitService::setup_hf_home(&setting_service)?;

    assert_eq!(result, expected_hf_home);
    assert!(expected_hf_home.join("hub").exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_logs_dir_when_setting_exists(temp_dir: TempDir) -> anyhow::Result<()> {
    let logs_dir = temp_dir.path().join("logs");
    let mut mock = MockSettingService::default();
    mock
      .expect_get_setting()
      .with(eq(BODHI_LOGS))
      .times(1)
      .return_const(Some(logs_dir.display().to_string()));

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let bodhi_home = temp_dir.path().to_path_buf();
    let result = InitService::setup_logs_dir(&setting_service, &bodhi_home)?;

    assert_eq!(result, logs_dir);
    assert!(logs_dir.exists());
    Ok(())
  }

  #[rstest]
  fn test_setup_logs_dir_when_setting_missing(temp_dir: TempDir) -> anyhow::Result<()> {
    let bodhi_home = temp_dir.path().to_path_buf();
    let expected_logs_dir = bodhi_home.join(LOGS_DIR);
    let mut mock = MockSettingService::default();

    mock
      .expect_get_setting()
      .with(eq(BODHI_LOGS))
      .times(1)
      .return_const(None);

    mock
      .expect_set_setting()
      .with(eq(BODHI_LOGS), eq(expected_logs_dir.display().to_string()))
      .return_once(|_, _| Ok(()));

    let setting_service: Arc<dyn SettingService> = Arc::new(mock);
    let result = InitService::setup_logs_dir(&setting_service, &bodhi_home)?;

    assert_eq!(result, expected_logs_dir);
    assert!(expected_logs_dir.exists());
    Ok(())
  }
}
