use crate::{
  db::DbService,
  test_utils::{EnvServiceStub, SecretServiceStub},
  AppRegInfoBuilder, AppService, AuthService, CacheService, DataService, EnvService, HfHubService,
  HubService, LocalDataService, MockAuthService, MockHubService, MokaCacheService, SecretService,
  SessionService, SqliteSessionService, BODHI_HOME, HF_HOME,
};
use derive_builder::Builder;
use objs::test_utils::{build_temp_dir, copy_test_dir};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tempfile::TempDir;

#[derive(Debug, Default, Builder)]
#[builder(default, setter(strip_option))]
pub struct AppServiceStub {
  #[builder(default = "self.default_env_service()")]
  pub env_service: Option<Arc<dyn EnvService>>,
  #[builder(default = "self.default_hub_service()")]
  pub hub_service: Option<Arc<dyn HubService>>,
  pub temp_home: Option<Arc<TempDir>>,
  pub data_service: Option<Arc<dyn DataService>>,
  #[builder(default = "self.default_auth_service()")]
  pub auth_service: Option<Arc<dyn AuthService>>,
  pub db_service: Option<Arc<dyn DbService>>,
  pub session_service: Option<Arc<dyn SessionService>>,
  #[builder(default = "self.default_secret_service()")]
  pub secret_service: Option<Arc<dyn SecretService>>,
  #[builder(default = "self.default_cache_service()")]
  pub cache_service: Option<Arc<dyn CacheService>>,
}

impl AppServiceStubBuilder {
  fn default_env_service(&self) -> Option<Arc<dyn EnvService>> {
    Some(Arc::new(EnvServiceStub::default()))
  }

  fn default_cache_service(&self) -> Option<Arc<dyn CacheService>> {
    Some(Arc::new(MokaCacheService::default()))
  }

  fn default_auth_service(&self) -> Option<Arc<dyn AuthService>> {
    Some(Arc::new(MockAuthService::default()))
  }

  fn default_hub_service(&self) -> Option<Arc<dyn HubService>> {
    Some(Arc::new(MockHubService::default()))
  }

  fn default_secret_service(&self) -> Option<Arc<dyn SecretService>> {
    Some(Arc::new(SecretServiceStub::default()))
  }

  pub fn with_hub_service(&mut self) -> &mut Self {
    let temp_home = self.with_temp_home();
    let hf_home = temp_home.path().join("huggingface");
    copy_test_dir("tests/data/huggingface", &hf_home);
    let hf_cache = hf_home.join("hub");
    let hub_service = HfHubService::new(hf_cache, false, None);
    self.hub_service = Some(Some(Arc::new(hub_service)));
    self
  }

  pub fn with_data_service(&mut self) -> &mut Self {
    let temp_home = self.with_temp_home();
    let bodhi_home = temp_home.path().join("bodhi");
    copy_test_dir("tests/data/bodhi", &bodhi_home);
    let data_service = LocalDataService::new(bodhi_home);
    self.data_service = Some(Some(Arc::new(data_service)));
    self
  }

  pub fn with_temp_home_as(&mut self, temp_dir: TempDir) -> &mut Self {
    self.temp_home = Some(Some(Arc::new(temp_dir)));
    self.with_temp_home();
    self
  }

  pub fn with_temp_home(&mut self) -> Arc<TempDir> {
    match &self.temp_home {
      Some(Some(temp_home)) => temp_home.clone(),
      None | Some(None) => {
        let temp_home = Arc::new(build_temp_dir());
        self.temp_home = Some(Some(temp_home.clone()));
        let envs = HashMap::from([
          (
            BODHI_HOME.to_string(),
            temp_home.path().join("bodhi").display().to_string(),
          ),
          (
            HF_HOME.to_string(),
            temp_home.path().join("huggingface").display().to_string(),
          ),
        ]);
        let env_service = EnvServiceStub::new(envs);
        self.env_service = Some(Some(Arc::new(env_service)));
        temp_home
      }
    }
  }

  pub async fn with_session_service(&mut self) -> &mut Self {
    let temp_home = self.with_temp_home();
    let dbfile = temp_home.path().join("test.db");
    self.build_session_service(dbfile).await;
    self
  }

  pub async fn build_session_service(&mut self, dbfile: PathBuf) -> &mut Self {
    let session_service = SqliteSessionService::build_session_service(dbfile).await;
    let session_service: Arc<dyn SessionService + Send + Sync> = Arc::new(session_service);
    self.session_service = Some(Some(session_service));
    self
  }

  pub fn with_sqlite_session_service(
    &mut self,
    session_service: Arc<SqliteSessionService>,
  ) -> &mut Self {
    self.session_service = Some(Some(session_service));
    self
  }

  pub fn with_envs(&mut self, envs: HashMap<&str, &str>) -> &mut Self {
    let mut env_service = EnvServiceStub::default();
    for (key, value) in envs {
      env_service = env_service.with_env(key, value);
    }
    self.env_service = Some(Some(Arc::new(env_service)));
    self
  }

  pub fn with_secret_service(&mut self) -> &mut Self {
    let mut secret_service = SecretServiceStub::default();
    secret_service.with_app_reg_info(&AppRegInfoBuilder::test_default().build().unwrap());
    self.secret_service = Some(Some(Arc::new(secret_service)));
    self
  }
}

impl AppServiceStub {
  pub fn bodhi_home(&self) -> PathBuf {
    self.temp_home.clone().unwrap().path().join("bodhi")
  }
}

impl AppService for AppServiceStub {
  fn env_service(&self) -> Arc<dyn EnvService> {
    self.env_service.clone().unwrap()
  }

  fn data_service(&self) -> Arc<dyn DataService> {
    self.data_service.clone().unwrap()
  }

  fn hub_service(&self) -> Arc<dyn HubService> {
    self.hub_service.clone().unwrap()
  }

  fn auth_service(&self) -> Arc<dyn AuthService> {
    self.auth_service.clone().unwrap()
  }

  fn db_service(&self) -> Arc<dyn DbService> {
    self.db_service.clone().unwrap()
  }

  fn session_service(&self) -> Arc<dyn SessionService> {
    self.session_service.clone().unwrap()
  }

  fn secret_service(&self) -> Arc<dyn SecretService> {
    self.secret_service.clone().unwrap()
  }

  fn cache_service(&self) -> Arc<dyn CacheService> {
    self.cache_service.clone().unwrap()
  }
}
