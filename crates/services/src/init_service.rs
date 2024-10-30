use crate::{EnvWrapper, ALIASES_DIR, BODHI_HOME, MODELS_YAML, PROD_DB};
use objs::{AppError, EnvType, ErrorType, IoDirCreateError, IoFileWriteError};
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
  pub fn setup(self) -> Result<PathBuf, InitServiceError> {
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
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::{InitService, InitServiceError};
  use crate::{MockEnvWrapper, ALIASES_DIR, BODHI_HOME, MODELS_YAML, PROD_DB};
  use mockall::predicate::eq;
  use objs::{
    test_utils::{assert_error_message, empty_bodhi_home, setup_l10n, temp_dir},
    AppError, EnvType, FluentLocalizationService,
  };
  use rstest::rstest;
  use std::{env::VarError, sync::Arc};
  use tempfile::TempDir;

  #[rstest]
  #[case(&InitServiceError::BodhiHomeNotFound,
    "failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  fn test_init_service_error_messages(
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
    let result = init_service.setup()?;
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
    let result = init_service.setup()?;
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
    let result = init_service.setup();
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
}
