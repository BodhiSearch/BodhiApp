use crate::{
  db::{DbService, TimeService},
  queue_service::{MockQueueProducer, QueueProducer},
  test_utils::{
    test_db_service, test_db_service_with_temp_dir, SecretServiceStub, SettingServiceStub,
    TestDbService,
  },
  AccessRequestService, AiApiService, AppRegInfoBuilder, AppService, AuthService, CacheService,
  ConcurrencyService, DataService, DefaultMcpService, DefaultToolService, HfHubService, HubService,
  LocalConcurrencyService, LocalDataService, McpService, MockAccessRequestService, MockAuthService,
  MockExaService, MockHubService, MokaCacheService, NetworkService, SecretService, SessionService,
  SettingService, SqliteSessionService, ToolService, BODHI_EXEC_LOOKUP_PATH,
};

use crate::network_service::StubNetworkService;
use derive_builder::Builder;
use objs::test_utils::{build_temp_dir, copy_test_dir};
use rstest::fixture;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tempfile::TempDir;

use super::{FrozenTimeService, OfflineHubService};

#[fixture]
#[awt]
pub async fn app_service_stub(
  #[future] mut app_service_stub_builder: AppServiceStubBuilder,
) -> AppServiceStub {
  app_service_stub_builder.build().await.unwrap()
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
    .await
    .db_service(Arc::new(test_db_service))
    .with_session_service()
    .await
    .with_secret_service()
    .to_owned()
}

#[derive(Debug, Default, Builder)]
#[builder(
  default,
  setter(strip_option),
  build_fn(private, name = "fallback_build")
)]
pub struct AppServiceStub {
  // Foundation - no dependencies
  pub temp_home: Option<Arc<TempDir>>,
  #[builder(default = "self.default_time_service()")]
  pub time_service: Option<Arc<dyn TimeService>>,

  // Core infrastructure - depends on temp_home
  #[builder(default = "self.default_setting_service()")]
  pub setting_service: Option<Arc<dyn SettingService>>,
  pub db_service: Option<Arc<dyn DbService>>,
  pub session_service: Option<Arc<dyn SessionService>>,
  #[builder(default = "self.default_secret_service()")]
  pub secret_service: Option<Arc<dyn SecretService>>,
  #[builder(default = "self.default_cache_service()")]
  pub cache_service: Option<Arc<dyn CacheService>>,

  // External services
  #[builder(default = "self.default_auth_service()")]
  pub auth_service: Option<Arc<dyn AuthService>>,
  #[builder(default = "self.default_hub_service()")]
  pub hub_service: Option<Arc<dyn HubService>>,
  #[builder(default = "self.default_network_service()")]
  pub network_service: Option<Arc<dyn NetworkService>>,

  // Business logic - depends on core infrastructure
  pub data_service: Option<Arc<dyn DataService>>,
  #[builder(default = "self.default_tool_service()")]
  pub tool_service: Option<Arc<dyn ToolService>>,
  pub ai_api_service: Option<Arc<dyn AiApiService>>,
  #[builder(default = "self.default_concurrency_service()")]
  pub concurrency_service: Option<Arc<dyn ConcurrencyService>>,
  #[builder(default = "self.default_queue_producer()")]
  pub queue_producer: Option<Arc<dyn QueueProducer>>,
  #[builder(default = "self.default_access_request_service()")]
  pub access_request_service: Option<Arc<dyn AccessRequestService>>,
  #[builder(default = "self.default_mcp_service()")]
  pub mcp_service: Option<Arc<dyn McpService>>,
}

impl AppServiceStubBuilder {
  /// Async build that auto-initializes db_service and session_service if not explicitly set.
  pub async fn build(&mut self) -> Result<AppServiceStub, AppServiceStubBuilderError> {
    if !matches!(&self.db_service, Some(Some(_))) {
      self.with_db_service().await;
    }
    if !matches!(&self.session_service, Some(Some(_))) {
      self.with_session_service().await;
    }
    self.fallback_build()
  }

  fn default_setting_service(&self) -> Option<Arc<dyn SettingService>> {
    if let Some(Some(temp_home)) = &self.temp_home {
      Some(Arc::new(SettingServiceStub::with_defaults_in(
        temp_home.clone(),
      )))
    } else {
      Some(Arc::new(SettingServiceStub::default()))
    }
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

  fn default_concurrency_service(&self) -> Option<Arc<dyn ConcurrencyService>> {
    Some(Arc::new(LocalConcurrencyService::new()))
  }

  fn default_queue_producer(&self) -> Option<Arc<dyn QueueProducer>> {
    Some(Arc::new(MockQueueProducer::default()))
  }

  fn default_tool_service(&self) -> Option<Arc<dyn ToolService>> {
    let db_service = self
      .db_service
      .as_ref()
      .and_then(|o| o.as_ref())
      .cloned()
      .expect("db_service must be set before building tool_service");
    let time_service: Arc<dyn TimeService> = self
      .time_service
      .as_ref()
      .and_then(|o| o.as_ref())
      .cloned()
      .unwrap_or_else(|| Arc::new(FrozenTimeService::default()));
    let exa_service: Arc<dyn crate::ExaService> = Arc::new(MockExaService::new());
    Some(Arc::new(DefaultToolService::new(
      db_service,
      exa_service,
      time_service,
    )))
  }

  fn default_network_service(&self) -> Option<Arc<dyn NetworkService>> {
    Some(Arc::new(StubNetworkService { ip: None }))
  }

  fn default_access_request_service(&self) -> Option<Arc<dyn AccessRequestService>> {
    Some(Arc::new(MockAccessRequestService::new()))
  }

  fn default_mcp_service(&self) -> Option<Arc<dyn McpService>> {
    let db_service = self
      .db_service
      .as_ref()
      .and_then(|o| o.as_ref())
      .cloned()
      .expect("db_service must be set before building mcp_service");
    let time_service: Arc<dyn TimeService> = self
      .time_service
      .as_ref()
      .and_then(|o| o.as_ref())
      .cloned()
      .unwrap_or_else(|| Arc::new(FrozenTimeService::default()));
    let mcp_client: Arc<dyn mcp_client::McpClient> = Arc::new(mcp_client::MockMcpClient::new());
    Some(Arc::new(DefaultMcpService::new(
      db_service,
      mcp_client,
      time_service,
    )))
  }

  fn with_temp_home(&mut self) -> &mut Self {
    self.with_temp_home_as(build_temp_dir());
    self
  }

  pub fn with_temp_home_as(&mut self, temp_dir: TempDir) -> &mut Self {
    let temp_home = Arc::new(temp_dir);
    self.temp_home = Some(Some(temp_home.clone()));
    // Only set default setting_service if not already explicitly configured
    if !matches!(&self.setting_service, Some(Some(_))) {
      let setting_service = SettingServiceStub::with_defaults_in(temp_home.clone());
      self.setting_service = Some(Some(Arc::new(setting_service)));
    }
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
      let setting_service = if let Some(Some(temp_home)) = &self.temp_home {
        SettingServiceStub::with_defaults_in(temp_home.clone())
      } else {
        SettingServiceStub::default()
      };
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

  pub fn get_hub_service(&mut self) -> Arc<dyn HubService> {
    if let Some(Some(hub_service)) = self.hub_service.as_ref() {
      return hub_service.clone();
    }
    self.with_hub_service();
    self.hub_service.clone().unwrap().unwrap()
  }

  pub async fn with_data_service(&mut self) -> &mut Self {
    if let Some(Some(_)) = self.data_service.as_ref() {
      return self;
    }
    let db_service = self.get_db_service().await;
    // Seed user aliases into DB
    crate::test_utils::seed_test_user_aliases(db_service.as_ref())
      .await
      .unwrap();
    let data_service = LocalDataService::new(self.get_hub_service(), db_service);
    self.data_service = Some(Some(Arc::new(data_service)));
    self
  }

  pub async fn with_session_service(&mut self) -> &mut Self {
    let temp_home = self.setup_temp_home();
    let dbfile = temp_home.path().join("test-session.sqlite");
    self.build_session_service(dbfile).await;
    self
  }

  pub async fn with_db_service(&mut self) -> &mut Self {
    if let Some(Some(_)) = self.db_service.as_ref() {
      return self;
    }
    let temp_home = self.setup_temp_home();
    self.db_service = Some(Some(Arc::new(
      test_db_service_with_temp_dir(temp_home).await,
    )));
    self
  }

  pub async fn get_db_service(&mut self) -> Arc<dyn DbService> {
    if let Some(Some(db_service)) = self.db_service.as_ref() {
      return db_service.clone();
    }
    self.with_db_service().await;
    self.db_service.clone().unwrap().unwrap()
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

  pub fn with_tool_service(&mut self, tool_service: Arc<dyn ToolService>) -> &mut Self {
    self.tool_service = Some(Some(tool_service));
    self
  }

  pub fn with_live_services(&mut self) -> &mut Self {
    let _temp_home = self.setup_temp_home();

    // Override exec lookup to real binary at crates/llama_server_proc/bin/
    let exec_lookup_path =
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../llama_server_proc/bin");
    let exec_lookup_str = exec_lookup_path.display().to_string();
    self.with_settings(HashMap::from([(
      BODHI_EXEC_LOOKUP_PATH,
      exec_lookup_str.as_str(),
    )]));

    // Real HF cache with OfflineHubService (panics on download, allows local reads)
    let hf_cache = dirs::home_dir()
      .expect("home dir should exist")
      .join(".cache/huggingface/hub");
    let hub_service = OfflineHubService::new(HfHubService::new(hf_cache, false, None));
    self.hub_service = Some(Some(Arc::new(hub_service)));

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

  fn time_service(&self) -> Arc<dyn TimeService> {
    self.time_service.clone().unwrap()
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

  fn auth_service(&self) -> Arc<dyn AuthService> {
    self.auth_service.clone().unwrap()
  }

  fn hub_service(&self) -> Arc<dyn HubService> {
    self.hub_service.clone().unwrap()
  }

  fn network_service(&self) -> Arc<dyn NetworkService> {
    self.network_service.clone().unwrap()
  }

  fn data_service(&self) -> Arc<dyn DataService> {
    self.data_service.clone().unwrap()
  }

  fn tool_service(&self) -> Arc<dyn ToolService> {
    self.tool_service.clone().unwrap()
  }

  fn ai_api_service(&self) -> Arc<dyn AiApiService> {
    self
      .ai_api_service
      .clone()
      .expect("ai_api_service not configured in test stub - call with_ai_api_service() or build with default")
  }

  fn concurrency_service(&self) -> Arc<dyn ConcurrencyService> {
    self.concurrency_service.clone().unwrap()
  }

  fn queue_producer(&self) -> Arc<dyn QueueProducer> {
    self.queue_producer.clone().unwrap()
  }

  fn access_request_service(&self) -> Arc<dyn AccessRequestService> {
    self.access_request_service.clone().unwrap()
  }

  fn mcp_service(&self) -> Arc<dyn McpService> {
    self.mcp_service.clone().unwrap()
  }
}
