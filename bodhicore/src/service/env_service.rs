#[allow(unused)]
#[cfg(not(test))]
use super::env_wrapper::EnvWrapper;
#[cfg(test)]
use crate::test_utils::MockEnvWrapper as EnvWrapper;

use super::DataServiceError;
use std::{
  fs::{self, File},
  path::{Path, PathBuf},
};

pub static PROD_DB: &str = "bodhi.sqlite";
pub static ALIASES_DIR: &str = "aliases";
pub static LOGS_DIR: &str = "logs";
pub static DEFAULT_PORT: u16 = 1135;
pub static DEFAULT_PORT_STR: &str = "1135";
pub static DEFAULT_HOST: &str = "127.0.0.1";

pub static BODHI_HOME: &str = "BODHI_HOME";
pub static BODHI_HOST: &str = "BODHI_HOST";
pub static BODHI_PORT: &str = "BODHI_PORT";
pub static BODHI_LOGS: &str = "BODHI_LOGS";
pub static HF_HOME: &str = "HF_HOME";

#[cfg_attr(test, mockall::automock)]
pub trait EnvServiceFn: std::fmt::Debug {
  fn bodhi_home(&self) -> PathBuf;

  fn hf_cache(&self) -> PathBuf;

  fn hf_home(&self) -> PathBuf;

  fn aliases_dir(&self) -> PathBuf;

  fn logs_dir(&self) -> PathBuf;

  fn host(&self) -> String;

  fn port(&self) -> u16;

  fn db_path(&self) -> PathBuf;
}

#[derive(Debug, Clone)]
pub struct EnvService {
  env_wrapper: EnvWrapper,
  bodhi_home: Option<PathBuf>,
  hf_home: Option<PathBuf>,
  logs_dir: Option<PathBuf>,
}

impl EnvServiceFn for EnvService {
  fn bodhi_home(&self) -> PathBuf {
    self
      .bodhi_home
      .as_ref()
      .expect(
        "unreachable: bodhi_home is None. setup_bodhi_home should be called before calling bodhi_home",
      )
      .clone()
  }

  fn hf_home(&self) -> PathBuf {
    self
      .hf_home
      .as_ref()
      .expect(
        "unreachable: hf_cache is None. setup_hf_cache should be called before calling hf_cache",
      )
      .clone()
  }

  fn hf_cache(&self) -> PathBuf {
    self.hf_home().join("hub")
  }

  fn aliases_dir(&self) -> PathBuf {
    self.bodhi_home().join("aliases")
  }

  fn logs_dir(&self) -> PathBuf {
    self
      .logs_dir
      .as_ref()
      .expect(
        "unreachable: logs_dir is None. setup_logs_dir should be called before calling logs_dir",
      )
      .clone()
  }

  fn host(&self) -> String {
    match self.env_wrapper.var(BODHI_HOST) {
      Ok(value) => value,
      Err(_) => DEFAULT_HOST.to_string(),
    }
  }

  fn port(&self) -> u16 {
    match self.env_wrapper.var(BODHI_PORT) {
      Ok(value) => match value.parse::<u16>() {
        Ok(port) => port,
        Err(_) => DEFAULT_PORT,
      },
      Err(_) => DEFAULT_PORT,
    }
  }

  fn db_path(&self) -> PathBuf {
    self.bodhi_home().join(PROD_DB)
  }
}

impl EnvService {
  #[allow(clippy::new_without_default)]
  pub fn new(env_wrapper: EnvWrapper) -> Self {
    EnvService {
      env_wrapper,
      bodhi_home: None,
      hf_home: None,
      logs_dir: None,
    }
  }

  #[allow(private_interfaces)]
  pub fn new_with_args(env_wrapper: EnvWrapper, bodhi_home: PathBuf, hf_home: PathBuf) -> Self {
    let logs_dir = hf_home.join("logs");
    Self {
      env_wrapper,
      bodhi_home: Some(bodhi_home),
      hf_home: Some(hf_home),
      logs_dir: Some(logs_dir),
    }
  }

  pub fn load_dotenv(&self) -> Option<PathBuf> {
    let envfile = self.bodhi_home().join(".env");
    if envfile.exists() {
      if let Err(err) = dotenv::from_path(&envfile) {
        eprintln!(
          "error loading .env file. err: {}, path: {}",
          err,
          envfile.display()
        );
        None
      } else {
        Some(envfile)
      }
    } else {
      None
    }
  }

  pub fn setup_bodhi_home(&mut self) -> Result<PathBuf, DataServiceError> {
    let value = self.env_wrapper.var(BODHI_HOME);
    let bodhi_home = match value {
      Ok(value) => PathBuf::from(value),
      Err(_) => {
        let home_dir = self.env_wrapper.home_dir();
        match home_dir {
          Some(home_dir) => home_dir.join(".cache").join("bodhi"),
          None => return Err(DataServiceError::BodhiHome),
        }
      }
    };
    if !bodhi_home.exists() {
      self.create_home_dirs(&bodhi_home)?;
    }
    self.bodhi_home = Some(bodhi_home.clone());
    Ok(bodhi_home)
  }

  pub fn create_home_dirs(&self, bodhi_home: &Path) -> Result<(), DataServiceError> {
    fs::create_dir_all(bodhi_home).map_err(|err| DataServiceError::DirCreate {
      source: err,
      path: bodhi_home.display().to_string(),
    })?;
    let alias_home = bodhi_home.join(ALIASES_DIR);
    fs::create_dir_all(&alias_home).map_err(|err| DataServiceError::DirCreate {
      source: err,
      path: alias_home.display().to_string(),
    })?;
    let db_path = bodhi_home.join(PROD_DB);
    File::create_new(&db_path).map_err(|err| DataServiceError::DirCreate {
      source: err,
      path: db_path.display().to_string(),
    })?;
    Ok(())
  }

  pub fn setup_hf_cache(&mut self) -> Result<PathBuf, DataServiceError> {
    let hf_home = match self.env_wrapper.var(HF_HOME) {
      Ok(hf_home) => PathBuf::from(hf_home),
      Err(_) => match self.env_wrapper.home_dir() {
        Some(home) => home.join(".cache").join("huggingface"),
        None => return Err(DataServiceError::HfHome),
      },
    };
    let hf_cache = hf_home.join("hub");
    if !hf_cache.exists() {
      fs::create_dir_all(&hf_cache).map_err(|err| DataServiceError::DirCreate {
        source: err,
        path: hf_cache.display().to_string(),
      })?;
    }
    self.hf_home = Some(hf_home.clone());
    Ok(hf_cache)
  }

  pub fn setup_logs_dir(&mut self) -> Result<PathBuf, DataServiceError> {
    let logs_dir = match self.env_wrapper.var(BODHI_LOGS) {
      Ok(logs_dir) => PathBuf::from(logs_dir),
      Err(_) => self.bodhi_home().join(LOGS_DIR),
    };
    fs::create_dir_all(&logs_dir).map_err(|err| DataServiceError::DirCreate {
      source: err,
      path: logs_dir.display().to_string(),
    })?;
    self.logs_dir = Some(logs_dir.clone());
    Ok(logs_dir)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test_utils::MockEnvWrapper;
  use mockall::predicate::eq;
  use rstest::{fixture, rstest};
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
  fn test_init_service_bodhi_home_from_env(bodhi_home: (TempDir, PathBuf)) -> anyhow::Result<()> {
    let (_tempdir, bodhi_home) = bodhi_home;
    let bodhi_home_str = bodhi_home.display().to_string();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(move |_| Ok(bodhi_home_str.clone()));
    let result = EnvService::new(mock).setup_bodhi_home()?;
    assert_eq!(bodhi_home, result);
    Ok(())
  }

  #[rstest::rstest]
  fn test_init_service_bodhi_home_from_home_dir(
    bodhi_home: (TempDir, PathBuf),
  ) -> anyhow::Result<()> {
    let (homedir, bodhi_home) = bodhi_home;
    let home_dir = homedir.path().display().to_string();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(|_| Err(VarError::NotPresent));
    mock
      .expect_home_dir()
      .returning(move || Some(PathBuf::from(home_dir.clone())));

    let result = EnvService::new(mock).setup_bodhi_home()?;
    assert_eq!(bodhi_home, result);
    Ok(())
  }

  #[rstest::rstest]
  fn test_init_service_fails_if_not_able_to_find_bodhi_home() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(|_| Err(VarError::NotPresent));
    mock.expect_home_dir().returning(move || None);

    let result = EnvService::new(mock).setup_bodhi_home();
    assert!(result.is_err());
    assert_eq!("bodhi_home_err: failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.", result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_init_service_hf_cache_from_env(hf_cache: (TempDir, PathBuf)) -> anyhow::Result<()> {
    let (_tempdir, hf_cache) = hf_cache;
    let hf_home = hf_cache
      .join("..")
      .canonicalize()
      .unwrap()
      .display()
      .to_string();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Ok(hf_home.clone()));
    let result = EnvService::new(mock).setup_hf_cache()?;
    assert_eq!(hf_cache.canonicalize()?, result);
    Ok(())
  }

  #[rstest]
  fn test_init_service_hf_cache_from_dirs_home(hf_cache: (TempDir, PathBuf)) -> anyhow::Result<()> {
    let (tempdir, hf_cache) = hf_cache;
    let home_dir = tempdir.path().to_path_buf();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Err(VarError::NotPresent));
    mock
      .expect_home_dir()
      .returning(move || Some(home_dir.clone()));
    let result = EnvService::new(mock).setup_hf_cache()?;
    assert_eq!(hf_cache, result);
    Ok(())
  }

  #[rstest]
  fn test_init_service_hf_cache_fails_otherwise() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Err(VarError::NotPresent));
    mock.expect_home_dir().returning(move || None);
    let result = EnvService::new(mock).setup_hf_cache();
    assert!(result.is_err());
    assert_eq!("hf_home_err: failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.", result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  fn test_init_service_loads_dotenv_from_bodhi_home(
    bodhi_home: (TempDir, PathBuf),
  ) -> anyhow::Result<()> {
    let (_tempdir, bodhi_home) = bodhi_home;
    let envfile = bodhi_home.join(".env");
    fs::write(&envfile, r#"TEST_NAME=load_from_dotenv"#)?;
    let bodhi_home_str = bodhi_home.display().to_string();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .return_once(move |_| Ok(bodhi_home_str));
    let mut env_service = EnvService::new(mock);
    env_service.setup_bodhi_home()?;
    let result = env_service.load_dotenv();
    assert_eq!(Some(envfile), result);
    let result = std::env::var("TEST_NAME")?;
    assert_eq!("load_from_dotenv", result);
    Ok(())
  }

  #[rstest]
  #[case(BODHI_HOST, "localhost", EnvService::host)]
  fn test_env_service_host_from_env_var(
    #[case] key: &str,
    #[case] value: String,
    #[case] func: for<'a> fn(&'a EnvService) -> String,
  ) -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    let expected = value.clone();
    mock
      .expect_var()
      .with(eq(key.to_string()))
      .return_once(move |_| Ok(value));
    let result = func(&EnvService::new(mock));
    assert_eq!(expected, result);
    Ok(())
  }

  #[rstest]
  fn test_env_service_host_from_fallback() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOST))
      .return_once(move |_| Err(VarError::NotPresent));
    let result = EnvService::new(mock).host();
    assert_eq!("127.0.0.1", result);
    Ok(())
  }

  #[rstest]
  fn test_env_service_port_from_env_var() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_PORT))
      .return_once(move |_| Ok("8080".to_string()));
    let result = EnvService::new(mock).port();
    assert_eq!(8080, result);
    Ok(())
  }

  #[rstest]
  fn test_env_service_port_from_fallback() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_PORT))
      .return_once(move |_| Err(VarError::NotPresent));
    let result = EnvService::new(mock).port();
    assert_eq!(1135, result);
    Ok(())
  }
}
