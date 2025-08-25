use crate::{
  db::{DbService, TimeService},
  test_utils::{test_db_service, SecretServiceStub, SettingServiceStub, TestDbService},
  AppRegInfoBuilder, AppService, AuthService, CacheService, DataService, HfHubService, HubService,
  LocalDataService, MockAuthService, MockHubService, MokaCacheService, SecretService,
  SessionService, SettingService, SqliteSessionService,
};
use derive_builder::Builder;
use objs::test_utils::{build_temp_dir, copy_test_dir, temp_dir};
use objs::LocalizationService;
use rstest::fixture;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tempfile::TempDir;

use super::{FrozenTimeService, OfflineHubService};

#[fixture]
#[awt]
pub async fn app_service_stub(
  #[future] app_service_stub_builder: AppServiceStubBuilder,
) -> AppServiceStub {
  app_service_stub_builder.build().unwrap()
}

#[fixture]
#[awt]
pub async fn app_service_stub_builder(
  #[future] test_db_service: TestDbService,
) -> AppServiceStubBuilder {
  AppServiceStubBuilder::default()
    // .with_temp_home()
    .with_hub_service()
    .with_data_service()
    .db_service(Arc::new(test_db_service))
    .with_session_service()
    .await
    .with_secret_service()
    .to_owned()
}

#[derive(Debug, Default, Builder)]
#[builder(default, setter(strip_option))]
pub struct AppServiceStub {
  pub temp_home: Option<Arc<TempDir>>,
  #[builder(default = "self.default_setting_service()")]
  pub setting_service: Option<Arc<dyn SettingService>>,
  #[builder(default = "self.default_hub_service()")]
  pub hub_service: Option<Arc<dyn HubService>>,
  pub data_service: Option<Arc<dyn DataService>>,
  #[builder(default = "self.default_auth_service()")]
  pub auth_service: Option<Arc<dyn AuthService>>,
  pub db_service: Option<Arc<dyn DbService>>,
  pub session_service: Option<Arc<dyn SessionService>>,
  #[builder(default = "self.default_secret_service()")]
  pub secret_service: Option<Arc<dyn SecretService>>,
  #[builder(default = "self.default_cache_service()")]
  pub cache_service: Option<Arc<dyn CacheService>>,
  pub localization_service: Option<Arc<dyn LocalizationService>>,
  #[builder(default = "self.default_time_service()")]
  pub time_service: Option<Arc<dyn TimeService>>,
}

impl AppServiceStubBuilder {
  fn default_setting_service(&self) -> Option<Arc<dyn SettingService>> {
    Some(Arc::new(SettingServiceStub::default()))
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

  fn default_time_service(&self) -> Option<Arc<dyn TimeService>> {
    Some(Arc::new(FrozenTimeService::default()))
  }

  fn with_temp_home(&mut self) -> &mut Self {
    self.with_temp_home_as(build_temp_dir());
    self
  }

  pub fn with_temp_home_as(&mut self, temp_dir: TempDir) -> &mut Self {
    let temp_home = Arc::new(temp_dir);
    self.temp_home = Some(Some(temp_home.clone()));
    let setting_service = SettingServiceStub::with_defaults_in(temp_home.clone());
    self.setting_service = Some(Some(Arc::new(setting_service)));
    self
  }

  pub fn setup_temp_home(&mut self) -> Arc<TempDir> {
    match &self.temp_home {
      Some(Some(temp_home)) => temp_home.clone(),
      None | Some(None) => {
        self.with_temp_home();
        self.temp_home.clone().unwrap().unwrap().clone()
      }
    }
  }

  pub fn with_settings(&mut self, settings: HashMap<&str, &str>) -> &mut Self {
    if let Some(Some(setting_service)) = &self.setting_service {
      for (key, value) in settings {
        setting_service.set_setting(key, value);
      }
    } else {
      let setting_service = SettingServiceStub::default();
      for (key, value) in settings {
        setting_service.set_setting(key, value);
      }
      self.setting_service = Some(Some(Arc::new(setting_service)));
    }
    self
  }

  pub fn with_hub_service(&mut self) -> &mut Self {
    if let Some(Some(_)) = self.hub_service.clone() {
      return self;
    }
    let temp_home = self.setup_temp_home();
    let hf_home = temp_home.path().join("huggingface");
    copy_test_dir("tests/data/huggingface", &hf_home);
    let hf_cache = hf_home.join("hub");
    let hub_service = OfflineHubService::new(HfHubService::new(hf_cache, false, None));
    self.hub_service = Some(Some(Arc::new(hub_service)));
    self
  }

  pub fn with_data_service(&mut self) -> &mut Self {
    let temp_home = self.setup_temp_home();
    let hub_service = self
      .with_hub_service()
      .hub_service
      .clone()
      .unwrap()
      .unwrap()
      .clone();
    let bodhi_home = temp_home.path().join("bodhi");
    copy_test_dir("tests/data/bodhi", &bodhi_home);
    let data_service = LocalDataService::new(bodhi_home, hub_service);
    self.data_service = Some(Some(Arc::new(data_service)));
    self
  }

  pub async fn with_session_service(&mut self) -> &mut Self {
    let temp_home = self.setup_temp_home();
    let dbfile = temp_home.path().join("test.db");
    self.build_session_service(dbfile).await;
    self
  }

  pub async fn with_db_service(&mut self) -> &mut Self {
    self.db_service = Some(Some(Arc::new(test_db_service(temp_dir()).await)));
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

  pub fn with_secret_service(&mut self) -> &mut Self {
    let secret_service = SecretServiceStub::default()
      .with_app_reg_info(&AppRegInfoBuilder::test_default().build().unwrap());
    self.secret_service = Some(Some(Arc::new(secret_service)));
    self
  }
}

impl AppServiceStub {
  pub fn bodhi_home(&self) -> PathBuf {
    self.temp_home.clone().unwrap().path().join("bodhi")
  }

  pub fn hf_cache(&self) -> PathBuf {
    self
      .temp_home
      .clone()
      .unwrap()
      .path()
      .join("huggingface")
      .join("hub")
  }
}

impl AppService for AppServiceStub {
  fn setting_service(&self) -> Arc<dyn SettingService> {
    self.setting_service.clone().unwrap()
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

  fn localization_service(&self) -> Arc<dyn LocalizationService> {
    self.localization_service.clone().unwrap()
  }

  fn time_service(&self) -> Arc<dyn TimeService> {
    self.time_service.clone().unwrap()
  }
}
