#[cfg(not(test))]
use super::env_service::EnvService;
use super::DataServiceError;
use crate::server::{BODHI_HOME, HF_HOME};
#[cfg(test)]
use crate::test_utils::MockEnvService as EnvService;
use std::{fs, path::PathBuf};

pub struct InitService {}

impl InitService {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    InitService {}
  }

  pub fn bodhi_home(&self) -> Result<PathBuf, DataServiceError> {
    let env_service = EnvService::new();
    let value = env_service.var(BODHI_HOME);
    let bodhi_home = match value {
      Ok(value) => PathBuf::from(value),
      Err(_) => {
        let home_dir = env_service.home_dir();
        match home_dir {
          Some(home_dir) => home_dir.join(".cache").join("bodhi"),
          None => return Err(DataServiceError::BodhiHome),
        }
      }
    };
    if !bodhi_home.exists() {
      fs::create_dir_all(&bodhi_home).map_err(|err| DataServiceError::DirCreate {
        source: err,
        path: bodhi_home.display().to_string(),
      })?;
    }
    Ok(bodhi_home)
  }

  pub fn hf_cache(&self) -> Result<PathBuf, DataServiceError> {
    let env_service = EnvService::new();
    let hf_cache = match env_service.var(HF_HOME) {
      Ok(hf_home) => PathBuf::from(hf_home).join("hub"),
      Err(_) => match env_service.home_dir() {
        Some(home) => home.join(".cache").join("huggingface").join("hub"),
        None => return Err(DataServiceError::HfHome),
      },
    };
    if !hf_cache.exists() {
      fs::create_dir_all(&hf_cache).map_err(|err| DataServiceError::DirCreate {
        source: err,
        path: hf_cache.display().to_string(),
      })?;
    }
    Ok(hf_cache)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{server::BODHI_HOME, test_utils::MockEnvService};
  use mockall::predicate::eq;
  use rstest::{fixture, rstest};
  use serial_test::serial;
  use std::{env::VarError, fs};
  use tempfile::TempDir;

  #[fixture]
  fn bodhi_home() -> (TempDir, PathBuf) {
    let tempdir = tempfile::tempdir().unwrap();
    let bodhi_home = tempdir.path().join(".cache").join("bodhi");
    fs::create_dir_all(&bodhi_home).unwrap();
    (tempdir, bodhi_home)
  }

  #[fixture]
  fn hf_cache() -> (TempDir, PathBuf) {
    let tempdir = tempfile::tempdir().unwrap();
    let hf_cache = tempdir
      .path()
      .join(".cache")
      .join("huggingface")
      .join("hub");
    fs::create_dir_all(&hf_cache).unwrap();
    (tempdir, hf_cache)
  }

  #[rstest::rstest]
  #[serial(env_service)]
  fn test_init_service_bodhi_home_from_env(bodhi_home: (TempDir, PathBuf)) -> anyhow::Result<()> {
    let (_tempdir, bodhi_home) = bodhi_home;
    let bodhi_home_str = bodhi_home.display().to_string();
    let mut mock = MockEnvService::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(move |_| Ok(bodhi_home_str.clone()));
    let ctx = MockEnvService::new_context();
    ctx.expect().return_once(move || mock);
    let result = InitService::new().bodhi_home()?;
    assert_eq!(bodhi_home, result);
    Ok(())
  }

  #[rstest::rstest]
  #[serial(env_service)]
  fn test_init_service_bodhi_home_from_home_dir(
    bodhi_home: (TempDir, PathBuf),
  ) -> anyhow::Result<()> {
    let (homedir, bodhi_home) = bodhi_home;
    let home_dir = homedir.path().display().to_string();
    let mut mock = MockEnvService::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(|_| Err(VarError::NotPresent));
    mock
      .expect_home_dir()
      .returning(move || Some(PathBuf::from(home_dir.clone())));

    let ctx = MockEnvService::new_context();
    ctx.expect().return_once(move || mock);
    let result = InitService::new().bodhi_home()?;
    assert_eq!(bodhi_home, result);
    Ok(())
  }

  #[rstest::rstest]
  #[serial(env_service)]
  fn test_init_service_fails_if_not_able_to_find_bodhi_home() -> anyhow::Result<()> {
    let mut mock = MockEnvService::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(|_| Err(VarError::NotPresent));
    mock.expect_home_dir().returning(move || None);

    let ctx = MockEnvService::new_context();
    ctx.expect().return_once(move || mock);
    let result = InitService::new().bodhi_home();
    assert!(result.is_err());
    assert_eq!("bodhi_home_err: failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.", result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
  fn test_init_service_hf_cache_from_env(hf_cache: (TempDir, PathBuf)) -> anyhow::Result<()> {
    let (_tempdir, hf_cache) = hf_cache;
    let hf_home = hf_cache
      .join("..")
      .canonicalize()
      .unwrap()
      .display()
      .to_string();
    let mut mock = MockEnvService::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Ok(hf_home.clone()));
    let ctx = MockEnvService::new_context();
    ctx.expect().return_once(move || mock);
    let result = InitService::new().hf_cache()?;
    assert_eq!(hf_cache.canonicalize()?, result);
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
  fn test_init_service_hf_cache_from_dirs_home(hf_cache: (TempDir, PathBuf)) -> anyhow::Result<()> {
    let (tempdir, hf_cache) = hf_cache;
    let home_dir = tempdir.path().to_path_buf();
    let mut mock = MockEnvService::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Err(VarError::NotPresent));
    mock
      .expect_home_dir()
      .returning(move || Some(home_dir.clone()));
    let ctx = MockEnvService::new_context();
    ctx.expect().return_once(move || mock);
    let result = InitService::new().hf_cache()?;
    assert_eq!(hf_cache, result);
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
  fn test_init_service_hf_cache_fails_otherwise() -> anyhow::Result<()> {
    let mut mock = MockEnvService::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Err(VarError::NotPresent));
    mock.expect_home_dir().returning(move || None);
    let ctx = MockEnvService::new_context();
    ctx.expect().return_once(move || mock);
    let result = InitService::new().hf_cache();
    assert!(result.is_err());
    assert_eq!("hf_home_err: failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.", result.unwrap_err().to_string());
    Ok(())
  }
}
