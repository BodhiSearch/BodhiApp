#[allow(unused)]
#[cfg(not(test))]
use super::env_wrapper::EnvWrapper;
#[cfg(test)]
use crate::test_utils::MockEnvWrapper as EnvWrapper;

use super::DataServiceError;
use std::{fs, path::PathBuf};

pub static PROD_DB: &str = "bodhi.sqlite";
pub static DEFAULT_PORT: u16 = 1135;
pub static DEFAULT_PORT_STR: &str = "1135";
pub static DEFAULT_HOST: &str = "127.0.0.1";

pub static BODHI_HOME: &str = "BODHI_HOME";
pub static BODHI_HOST: &str = "BODHI_HOST";
pub static BODHI_PORT: &str = "BODHI_PORT";
pub static HF_HOME: &str = "HF_HOME";

pub struct EnvService {
  env_wrapper: EnvWrapper,
}

impl EnvService {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    let env_wrapper = EnvWrapper::new();
    EnvService { env_wrapper }
  }

  pub fn load_dotenv(&self) -> Option<PathBuf> {
    if let Ok(bodhi_home) = self.bodhi_home() {
      let envfile = bodhi_home.join(".env");
      if envfile.exists() {
        if let Err(err) = dotenv::from_path(&envfile) {
          eprintln!(
            "error loading .env file. err: {}, path: {}",
            err,
            envfile.display()
          );
        } else {
          return Some(envfile);
        }
      }
    }
    None
  }

  pub fn bodhi_home(&self) -> Result<PathBuf, DataServiceError> {
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
      fs::create_dir_all(&bodhi_home).map_err(|err| DataServiceError::DirCreate {
        source: err,
        path: bodhi_home.display().to_string(),
      })?;
    }
    Ok(bodhi_home)
  }

  pub fn hf_cache(&self) -> Result<PathBuf, DataServiceError> {
    let hf_cache = match self.env_wrapper.var(HF_HOME) {
      Ok(hf_home) => PathBuf::from(hf_home).join("hub"),
      Err(_) => match self.env_wrapper.home_dir() {
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

  pub fn host(&self) -> String {
    match self.env_wrapper.var(BODHI_HOST) {
      Ok(value) => value,
      Err(_) => DEFAULT_HOST.to_string(),
    }
  }

  pub fn port(&self) -> u16 {
    match self.env_wrapper.var(BODHI_PORT) {
      Ok(value) => match value.parse::<u16>() {
        Ok(port) => port,
        Err(_) => DEFAULT_PORT,
      },
      Err(_) => DEFAULT_PORT,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test_utils::MockEnvWrapper;
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
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(move |_| Ok(bodhi_home_str.clone()));
    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().bodhi_home()?;
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
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(|_| Err(VarError::NotPresent));
    mock
      .expect_home_dir()
      .returning(move || Some(PathBuf::from(home_dir.clone())));

    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().bodhi_home()?;
    assert_eq!(bodhi_home, result);
    Ok(())
  }

  #[rstest::rstest]
  #[serial(env_service)]
  fn test_init_service_fails_if_not_able_to_find_bodhi_home() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(|_| Err(VarError::NotPresent));
    mock.expect_home_dir().returning(move || None);

    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().bodhi_home();
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
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Ok(hf_home.clone()));
    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().hf_cache()?;
    assert_eq!(hf_cache.canonicalize()?, result);
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
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
    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().hf_cache()?;
    assert_eq!(hf_cache, result);
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
  fn test_init_service_hf_cache_fails_otherwise() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .returning(move |_| Err(VarError::NotPresent));
    mock.expect_home_dir().returning(move || None);
    let ctx = EnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().hf_cache();
    assert!(result.is_err());
    assert_eq!("hf_home_err: failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.", result.unwrap_err().to_string());
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
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
      .returning(move |_| Ok(bodhi_home_str.clone()));
    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().load_dotenv();
    assert_eq!(Some(envfile), result);
    let result = std::env::var("TEST_NAME")?;
    assert_eq!("load_from_dotenv", result);
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
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
    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = func(&EnvService::new());
    assert_eq!(expected, result);
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
  fn test_env_service_host_from_fallback() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOST))
      .return_once(move |_| Err(VarError::NotPresent));
    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().host();
    assert_eq!("127.0.0.1", result);
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
  fn test_env_service_port_from_env_var() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_PORT))
      .return_once(move |_| Ok("8080".to_string()));
    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().port();
    assert_eq!(8080, result);
    Ok(())
  }

  #[rstest]
  #[serial(env_service)]
  fn test_env_service_port_from_fallback() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_PORT))
      .return_once(move |_| Err(VarError::NotPresent));
    let ctx = MockEnvWrapper::new_context();
    ctx.expect().return_once(move || mock);
    let result = EnvService::new().port();
    assert_eq!(1135, result);
    Ok(())
  }
}

/*

#[allow(unused)]
pub fn port_from_env_vars(port: Result<String, env::VarError>) -> u16 {
  match port {
    Ok(port) => match port.parse::<u16>() {
      Ok(port) => port,
      Err(err) => {
        tracing::debug!(
          err = ?err,
          port = port,
          default_port = DEFAULT_PORT,
          "error parsing port set in environment variable, using default port",
        );
        DEFAULT_PORT
      }
    },
    Err(err) => {
      tracing::debug!(
        err = ?err,
        default_port = DEFAULT_PORT,
        "error reading port from environment variable, using default port",
      );
      DEFAULT_PORT
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{port_from_env_vars, DEFAULT_PORT};
  use rstest::rstest;

  #[test]
  pub fn test_port_from_env_vars_not_present() {
    let port = port_from_env_vars(Err(std::env::VarError::NotPresent));
    assert_eq!(port, DEFAULT_PORT);
  }

  #[test]
  pub fn test_port_from_env_vars_valid() {
    let port = port_from_env_vars(Ok("8055".to_string()));
    assert_eq!(port, 8055);
  }

  #[rstest]
  #[case("notu16")]
  #[case("65536")]
  #[case("-1")]
  pub fn test_port_from_env_vars_malformed(#[case] input: &str) {
    let port = port_from_env_vars(Ok(input.to_string()));
    assert_eq!(port, DEFAULT_PORT);
  }
}


*/
