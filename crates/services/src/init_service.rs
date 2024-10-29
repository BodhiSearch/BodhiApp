use crate::{
  EnvWrapper, ALIASES_DIR, BODHI_HOME, BODHI_LOGS, HF_HOME, LOGS_DIR, MODELS_YAML, PROD_DB,
  SETTINGS_YAML,
};
use objs::{AppError, EnvType, ErrorType, IoDirCreateError, IoFileWriteError, Settings};
use std::{
  fs::{self, File},
  path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum InitServiceError {
  #[error("bodhi_home_not_found")]
  #[error_meta(error_type = ErrorType::InvalidAppState, status = 500)]
  BodhiHomeNotFound,
  #[error("hf_home_not_found")]
  #[error_meta(error_type = ErrorType::InvalidAppState, status = 500)]
  HfHomeNotFound,
  #[error(transparent)]
  DirCreate(#[from] IoDirCreateError),
  #[error(transparent)]
  IoFileWrite(#[from] IoFileWriteError),
}

#[derive(derive_new::new)]
pub struct InitService<'a> {
  env_wrapper: &'a dyn EnvWrapper,
  env_type: &'a EnvType,
}

impl InitService<'_> {
  pub fn setup(self) -> Result<(PathBuf, PathBuf, PathBuf), InitServiceError> {
    let bodhi_home = self.setup_bodhi_home()?;
    let env_file = bodhi_home.join(".env");
    if env_file.exists() {
      self.env_wrapper.load(&env_file);
    }
    let hf_home = self.setup_hf_home()?;
    let logs_dir = self.setup_logs_dir(&bodhi_home)?;
    Ok((bodhi_home, hf_home, logs_dir))
  }

  fn setup_bodhi_home(&self) -> Result<PathBuf, InitServiceError> {
    let bodhi_home = self.find_bodhi_home()?;
    self.create_home_dirs(&bodhi_home)?;
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

  fn create_home_dirs(&self, bodhi_home: &Path) -> Result<(), InitServiceError> {
    if !bodhi_home.exists() {
      fs::create_dir_all(bodhi_home)
        .map_err(|err| IoDirCreateError::new(err, bodhi_home.display().to_string()))?;
    }

    let alias_home = bodhi_home.join(ALIASES_DIR);
    if !alias_home.exists() {
      fs::create_dir_all(&alias_home)
        .map_err(|err| IoDirCreateError::new(err, alias_home.display().to_string()))?;
    }
    let db_path = bodhi_home.join(PROD_DB);
    if !db_path.exists() {
      File::create_new(&db_path)
        .map_err(|err| IoFileWriteError::new(err, db_path.display().to_string()))?;
    }
    let models_file = bodhi_home.join(MODELS_YAML);
    if !models_file.exists() {
      let contents = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/models.yaml"));
      if let Err(err) = fs::write(models_file, contents) {
        eprintln!("failed to copy models.yaml to $BODHI_HOME. err: {err}");
      };
    }
    let settings_file = bodhi_home.join(SETTINGS_YAML);
    if !settings_file.exists() {
      let settings = Settings::app_default();
      let contents = serde_yaml::to_string(&settings).unwrap_or_default();
      if let Err(err) = fs::write(settings_file, contents) {
        eprintln!("failed to write settings.yaml to $BODHI_HOME. err: {err}");
      }
    }
    Ok(())
  }

  fn setup_hf_home(&self) -> Result<PathBuf, InitServiceError> {
    let hf_home = match self.env_wrapper.var(HF_HOME) {
      Ok(hf_home) => PathBuf::from(hf_home),
      Err(_) => match self.env_wrapper.home_dir() {
        Some(home) => home.join(".cache").join("huggingface"),
        None => return Err(InitServiceError::HfHomeNotFound),
      },
    };
    let hf_hub = hf_home.join("hub");
    if !hf_hub.exists() {
      fs::create_dir_all(&hf_hub)
        .map_err(|err| IoDirCreateError::new(err, hf_hub.display().to_string()))?;
    }
    Ok(hf_home)
  }

  fn setup_logs_dir(&self, bodhi_home: &Path) -> Result<PathBuf, InitServiceError> {
    let logs_dir = match self.env_wrapper.var(BODHI_LOGS) {
      Ok(logs_dir) => PathBuf::from(logs_dir),
      Err(_) => bodhi_home.join(LOGS_DIR),
    };
    fs::create_dir_all(&logs_dir)
      .map_err(|err| IoDirCreateError::new(err, logs_dir.display().to_string()))?;
    Ok(logs_dir)
  }
}

#[cfg(test)]
mod tests {
  use super::{InitService, InitServiceError};
  use crate::{
    test_utils::EnvWrapperStub, MockEnvWrapper, ALIASES_DIR, BODHI_HOME, BODHI_LOGS, HF_HOME,
    MODELS_YAML, PROD_DB, SETTINGS_YAML,
  };
  use mockall::predicate::eq;
  use objs::{
    test_utils::{assert_error_message, empty_bodhi_home, empty_hf_home, setup_l10n, temp_dir},
    AppError, EnvType, FluentLocalizationService,
  };
  use rstest::rstest;
  use std::{collections::HashMap, env::VarError, fs, sync::Arc};
  use strfmt::strfmt;
  use tempfile::TempDir;

  #[rstest]
  #[case(&InitServiceError::BodhiHomeNotFound,
    "failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  #[case(&InitServiceError::HfHomeNotFound,
    "failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.")]
  fn test_init_service_error(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] message: String,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), &message);
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
  fn test_init_service_loads_dotenv_from_bodhi_home(
    empty_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let hf_home = empty_bodhi_home.path().join(".cache").join("huggingface");
    let logs_home = empty_bodhi_home.path().join("bodhi").join("logs");
    let envfile = bodhi_home.join(".env");
    fs::write(&envfile, r#"TEST_NAME=load_from_dotenv"#)?;
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .return_once(move |_| Ok(bodhi_home.display().to_string()));
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .return_once(move |_| Ok(hf_home.display().to_string()));
    mock
      .expect_var()
      .with(eq(BODHI_LOGS))
      .return_once(move |_| Ok(logs_home.display().to_string()));
    mock.expect_load().returning(move |_| ());
    let init_service = InitService::new(&mock, &EnvType::Development);
    let _ = init_service.setup()?;
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
    init_service.create_home_dirs(&bodhi_home)?;
    assert!(bodhi_home.join(ALIASES_DIR).exists());
    assert!(bodhi_home.join(PROD_DB).exists());
    assert!(bodhi_home.join(MODELS_YAML).exists());
    assert!(bodhi_home.join(SETTINGS_YAML).exists());
    Ok(())
  }

  #[rstest]
  fn test_init_service_hf_home_from_env(empty_hf_home: TempDir) -> anyhow::Result<()> {
    let hf_home = empty_hf_home.path().join(".cache").join("huggingface");
    let hf_home_str = hf_home.display().to_string();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .return_once(move |_| Ok(hf_home_str));
    let init_service = InitService::new(&mock, &EnvType::Development);
    let result = init_service.setup_hf_home()?;
    assert_eq!(hf_home, result);
    Ok(())
  }

  #[rstest]
  fn test_init_service_hf_home_from_dirs_home(empty_hf_home: TempDir) -> anyhow::Result<()> {
    let hf_home = empty_hf_home.path().join(".cache").join("huggingface");
    let home_dir = empty_hf_home.path().to_path_buf();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Err(VarError::NotPresent));
    mock
      .expect_home_dir()
      .returning(move || Some(home_dir.clone()));
    let init_service = InitService::new(&mock, &EnvType::Development);
    let result = init_service.setup_hf_home()?;
    assert_eq!(hf_home, result);
    Ok(())
  }

  #[rstest]
  fn test_init_service_hf_home_fails_otherwise() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Err(VarError::NotPresent));
    mock.expect_home_dir().returning(move || None);
    let init_service = InitService::new(&mock, &EnvType::Development);
    let result = init_service.setup_hf_home();
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      InitServiceError::HfHomeNotFound
    ));
    Ok(())
  }

  #[rstest]
  #[case::dev_from_home(EnvType::Development, None, "bodhi-dev")]
  #[case::prod_from_home(EnvType::Production, None, "bodhi")]
  #[case::dev_from_env(
    EnvType::Development,
    Some("{temp_dir}/.cache/bodhi-dev-from-env"),
    "bodhi-dev-from-env"
  )]
  #[case::prod_from_env(
    EnvType::Production,
    Some("{temp_dir}/.cache/bodhi-prod-from-env"),
    "bodhi-prod-from-env"
  )]
  fn test_env_service_setup_updates_dirs_in_env_service(
    #[case] env_type: EnvType,
    #[case] bodhi_home_tmpl: Option<&str>,
    #[case] expected: String,
    temp_dir: TempDir,
  ) -> anyhow::Result<()> {
    let mut envs = HashMap::from([("HOME".to_string(), temp_dir.path().display().to_string())]);
    let expected_bodhi_home = if let Some(bodhi_home_tmpl) = bodhi_home_tmpl {
      let bodhi_home =
        strfmt!(bodhi_home_tmpl, temp_dir => temp_dir.path().display().to_string()).unwrap();
      envs.insert(BODHI_HOME.to_string(), bodhi_home.clone());
      bodhi_home
    } else {
      format!("{}/.cache/{expected}", temp_dir.path().display())
    };
    let env_wrapper = EnvWrapperStub::new(envs);
    let service = InitService::new(&env_wrapper, &env_type);
    let (bodhi_home, hf_home, logs_dir) = service.setup()?;
    assert_eq!(expected_bodhi_home, bodhi_home.display().to_string());
    assert_eq!(
      format!("{}/.cache/huggingface", temp_dir.path().display()),
      hf_home.display().to_string()
    );
    assert_eq!(
      format!("{}/logs", temp_dir.path().join(bodhi_home).display()),
      logs_dir.display().to_string()
    );
    Ok(())
  }
}
