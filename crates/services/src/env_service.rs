use crate::{DataServiceError, EnvWrapper};
use objs::{EnvType, IoDirCreateError, IoFileWriteError};
use std::{
  collections::HashMap,
  fs::{self, File},
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

pub static PROD_DB: &str = "bodhi.sqlite";
pub static ALIASES_DIR: &str = "aliases";
pub static MODELS_YAML: &str = "models.yaml";

pub static LOGS_DIR: &str = "logs";
pub static DEFAULT_SCHEME: &str = "http";
pub static DEFAULT_HOST: &str = "localhost";
pub static DEFAULT_PORT: u16 = 1135;
pub static DEFAULT_PORT_STR: &str = "1135";

pub static BODHI_HOME: &str = "BODHI_HOME";
pub static BODHI_SCHEME: &str = "BODHI_SCHEME";
pub static BODHI_FRONTEND_URL: &str = "BODHI_FRONTEND_URL";
pub static BODHI_HOST: &str = "BODHI_HOST";
pub static BODHI_PORT: &str = "BODHI_PORT";
pub static BODHI_LOGS: &str = "BODHI_LOGS";
pub static HF_HOME: &str = "HF_HOME";
pub static BODHI_AUTH_URL: &str = "BODHI_AUTH_URL";
pub static BODHI_AUTH_REALM: &str = "BODHI_AUTH_REALM";

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait EnvService: Send + Sync + std::fmt::Debug {
  fn env_type(&self) -> EnvType;

  fn is_production(&self) -> bool {
    self.env_type() == EnvType::Production
  }

  fn version(&self) -> String;

  fn frontend_url(&self) -> String;

  fn bodhi_home(&self) -> PathBuf;

  fn hf_home(&self) -> PathBuf;

  fn logs_dir(&self) -> PathBuf;

  fn scheme(&self) -> String;

  fn host(&self) -> String;

  fn port(&self) -> u16;

  fn db_path(&self) -> PathBuf;

  fn list(&self) -> HashMap<String, String>;

  fn auth_url(&self) -> String;

  fn auth_realm(&self) -> String;

  fn hf_cache(&self) -> PathBuf {
    self.hf_home().join("hub")
  }

  fn server_url(&self) -> String {
    format!("{}://{}:{}", self.scheme(), self.host(), self.port())
  }

  fn aliases_dir(&self) -> PathBuf {
    self.bodhi_home().join("aliases")
  }

  fn login_url(&self) -> String {
    format!(
      "{}/realms/{}/protocol/openid-connect/auth",
      self.auth_url(),
      self.auth_realm()
    )
  }

  fn token_url(&self) -> String {
    format!(
      "{}/realms/{}/protocol/openid-connect/token",
      self.auth_url(),
      self.auth_realm()
    )
  }

  fn login_callback_url(&self) -> String {
    format!(
      "{}://{}:{}/app/login/callback",
      self.scheme(),
      self.host(),
      self.port()
    )
  }
}

#[derive(Debug, Clone)]
pub struct DefaultEnvService {
  env_type: EnvType,
  auth_url: String,
  auth_realm: String,
  version: String,
  env_wrapper: Arc<dyn EnvWrapper>,
  bodhi_home: Option<PathBuf>,
  hf_home: Option<PathBuf>,
  logs_dir: Option<PathBuf>,
  host: Arc<Mutex<Option<String>>>,
  port: Arc<Mutex<Option<u16>>>,
}

impl DefaultEnvService {
  pub fn set_host(&self, host: &str) {
    *self.host.lock().unwrap() = Some(host.to_string());
  }

  pub fn set_port(&self, port: u16) {
    *self.port.lock().unwrap() = Some(port);
  }
}

impl EnvService for DefaultEnvService {
  fn env_type(&self) -> EnvType {
    self.env_type.clone()
  }

  fn version(&self) -> String {
    self.version.clone()
  }

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

  fn logs_dir(&self) -> PathBuf {
    self
      .logs_dir
      .as_ref()
      .expect(
        "unreachable: logs_dir is None. setup_logs_dir should be called before calling logs_dir",
      )
      .clone()
  }

  fn frontend_url(&self) -> String {
    match self.env_wrapper.var(BODHI_FRONTEND_URL) {
      Ok(value) => value,
      Err(_) => self.server_url(),
    }
  }

  fn scheme(&self) -> String {
    match self.env_wrapper.var(BODHI_SCHEME) {
      Ok(value) => value,
      Err(_) => DEFAULT_SCHEME.to_string(),
    }
  }

  fn host(&self) -> String {
    match self.host.lock().unwrap().as_ref() {
      Some(host) => host.clone(),
      None => match self.env_wrapper.var(BODHI_HOST) {
        Ok(value) => value,
        Err(_) => DEFAULT_HOST.to_string(),
      },
    }
  }

  fn port(&self) -> u16 {
    match self.port.lock().unwrap().as_ref() {
      Some(port) => *port,
      None => match self.env_wrapper.var(BODHI_PORT) {
        Ok(value) => match value.parse::<u16>() {
          Ok(port) => port,
          Err(_) => DEFAULT_PORT,
        },
        Err(_) => DEFAULT_PORT,
      },
    }
  }

  fn db_path(&self) -> PathBuf {
    self.bodhi_home().join(PROD_DB)
  }

  fn list(&self) -> HashMap<String, String> {
    let mut result = HashMap::<String, String>::new();
    result.insert(
      BODHI_HOME.to_string(),
      self.bodhi_home().display().to_string(),
    );
    result.insert(HF_HOME.to_string(), self.hf_home().display().to_string());
    result.insert(
      BODHI_LOGS.to_string(),
      self.logs_dir().display().to_string(),
    );
    result.insert(BODHI_HOST.to_string(), self.host());
    result.insert(BODHI_PORT.to_string(), self.port().to_string());
    result
  }

  fn auth_url(&self) -> String {
    self.auth_url.clone()
  }

  fn auth_realm(&self) -> String {
    self.auth_realm.clone()
  }
}

impl DefaultEnvService {
  #[allow(clippy::new_without_default)]
  pub fn new(
    env_type: EnvType,
    auth_url: String,
    auth_realm: String,
    env_wrapper: Arc<dyn EnvWrapper>,
  ) -> Self {
    DefaultEnvService {
      env_type,
      auth_url,
      auth_realm,
      version: env!("CARGO_PKG_VERSION").to_string(),
      env_wrapper,
      bodhi_home: None,
      hf_home: None,
      logs_dir: None,
      host: Arc::new(Mutex::new(None)),
      port: Arc::new(Mutex::new(None)),
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
          Some(home_dir) => {
            let path = if self.is_production() {
              "bodhi"
            } else {
              "bodhi-dev"
            };
            home_dir.join(".cache").join(path)
          }
          None => return Err(DataServiceError::BodhiHome),
        }
      }
    };
    self.create_home_dirs(&bodhi_home)?;
    self.bodhi_home = Some(bodhi_home.clone());
    Ok(bodhi_home)
  }

  pub fn create_home_dirs(&self, bodhi_home: &Path) -> Result<(), DataServiceError> {
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
      fs::create_dir_all(&hf_cache)
        .map_err(|err| IoDirCreateError::new(err, hf_cache.display().to_string()))?;
    }
    self.hf_home = Some(hf_home.clone());
    Ok(hf_cache)
  }

  pub fn setup_logs_dir(&mut self) -> Result<PathBuf, DataServiceError> {
    let logs_dir = match self.env_wrapper.var(BODHI_LOGS) {
      Ok(logs_dir) => PathBuf::from(logs_dir),
      Err(_) => self.bodhi_home().join(LOGS_DIR),
    };
    fs::create_dir_all(&logs_dir)
      .map_err(|err| IoDirCreateError::new(err, logs_dir.display().to_string()))?;
    self.logs_dir = Some(logs_dir.clone());
    Ok(logs_dir)
  }
}

#[cfg(test)]
mod test {
  use crate::{
    test_utils::EnvWrapperStub, DataServiceError, DefaultEnvService, EnvService, MockEnvWrapper,
    BODHI_HOME, BODHI_HOST, BODHI_PORT, HF_HOME,
  };
  use mockall::predicate::eq;
  use objs::{
    test_utils::{empty_bodhi_home, empty_hf_home, temp_dir},
    EnvType,
  };
  use rstest::rstest;
  use std::{collections::HashMap, env::VarError, fs, sync::Arc};
  use strfmt::strfmt;
  use tempfile::TempDir;

  #[rstest]
  fn test_init_service_bodhi_home_from_env(empty_bodhi_home: TempDir) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let bodhi_home_str = bodhi_home.display().to_string();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .returning(move |_| Ok(bodhi_home_str.clone()));
    let result = DefaultEnvService::test_new(Arc::new(mock)).setup_bodhi_home()?;
    assert_eq!(bodhi_home, result);
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

    let result = DefaultEnvService::test_new(Arc::new(mock)).setup_bodhi_home();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DataServiceError::BodhiHome));
    Ok(())
  }

  #[rstest]
  fn test_init_service_hf_cache_from_env(empty_hf_home: TempDir) -> anyhow::Result<()> {
    let hf_home = empty_hf_home.path().join(".cache").join("huggingface");
    let hf_home_str = hf_home.display().to_string();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(HF_HOME))
      .return_once(move |_| Ok(hf_home_str));
    let result = DefaultEnvService::test_new(Arc::new(mock)).setup_hf_cache()?;
    assert_eq!(hf_home.join("hub"), result);
    Ok(())
  }

  #[rstest]
  fn test_init_service_hf_cache_from_dirs_home(empty_hf_home: TempDir) -> anyhow::Result<()> {
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
    let result = DefaultEnvService::test_new(Arc::new(mock)).setup_hf_cache()?;
    assert_eq!(hf_home.join("hub"), result);
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
    let result = DefaultEnvService::test_new(Arc::new(mock)).setup_hf_cache();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DataServiceError::HfHome));
    Ok(())
  }

  #[rstest]
  fn test_init_service_loads_dotenv_from_bodhi_home(
    empty_bodhi_home: TempDir,
  ) -> anyhow::Result<()> {
    let bodhi_home = empty_bodhi_home.path().join("bodhi");
    let envfile = bodhi_home.join(".env");
    fs::write(&envfile, r#"TEST_NAME=load_from_dotenv"#)?;
    let bodhi_home_str = bodhi_home.display().to_string();
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_HOME))
      .return_once(move |_| Ok(bodhi_home_str));
    let mut env_service = DefaultEnvService::test_new(Arc::new(mock));
    env_service.setup_bodhi_home()?;
    let result = env_service.load_dotenv();
    assert_eq!(Some(envfile), result);
    let result = std::env::var("TEST_NAME")?;
    assert_eq!("load_from_dotenv", result);
    Ok(())
  }

  #[rstest]
  #[case(BODHI_HOST, "localhost", DefaultEnvService::host)]
  fn test_env_service_host_from_env_var(
    #[case] key: &str,
    #[case] value: String,
    #[case] func: for<'a> fn(&'a DefaultEnvService) -> String,
  ) -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    let expected = value.clone();
    mock
      .expect_var()
      .with(eq(key.to_string()))
      .return_once(move |_| Ok(value));
    let result = func(&DefaultEnvService::test_new(Arc::new(mock)));
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
    let result = DefaultEnvService::test_new(Arc::new(mock)).host();
    assert_eq!("localhost", result);
    Ok(())
  }

  #[rstest]
  fn test_env_service_port_from_env_var() -> anyhow::Result<()> {
    let mut mock = MockEnvWrapper::default();
    mock
      .expect_var()
      .with(eq(BODHI_PORT))
      .return_once(move |_| Ok("8080".to_string()));
    let result = DefaultEnvService::test_new(Arc::new(mock)).port();
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
    let result = DefaultEnvService::test_new(Arc::new(mock)).port();
    assert_eq!(1135, result);
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
    let mut envs = HashMap::from([
      ("HOME".to_string(), temp_dir.path().display().to_string()),
      (BODHI_HOST.to_string(), "0.0.0.0".to_string()),
      (BODHI_PORT.to_string(), "8080".to_string()),
    ]);
    let expected_bodhi_home = if let Some(bodhi_home_tmpl) = bodhi_home_tmpl {
      let bodhi_home =
        strfmt!(bodhi_home_tmpl, temp_dir => temp_dir.path().display().to_string()).unwrap();
      envs.insert(BODHI_HOME.to_string(), bodhi_home.clone());
      bodhi_home
    } else {
      format!("{}/.cache/{expected}", temp_dir.path().display())
    };
    let env_wrapper = EnvWrapperStub::new(envs);
    let mut result = DefaultEnvService::new(
      env_type,
      "https://id.getbodhi.app".to_string(),
      "bodhi-realm".to_string(),
      Arc::new(env_wrapper),
    );
    result.setup_bodhi_home()?;
    result.setup_hf_cache()?;
    result.setup_logs_dir()?;
    let actual = result.list();
    let mut expected = HashMap::<String, String>::new();
    expected.insert("BODHI_HOME".to_string(), expected_bodhi_home);
    expected.insert(
      "HF_HOME".to_string(),
      format!("{}/.cache/huggingface", temp_dir.path().display()),
    );
    expected.insert("BODHI_HOST".to_string(), "0.0.0.0".to_string());
    expected.insert("BODHI_PORT".to_string(), "8080".to_string());
    for key in expected.keys() {
      assert_eq!(
        expected
          .get(key)
          .unwrap_or_else(|| panic!("{} to be present", &key)),
        actual
          .get(key)
          .unwrap_or_else(|| panic!("{} to be present", &key))
      );
    }
    Ok(())
  }
}
