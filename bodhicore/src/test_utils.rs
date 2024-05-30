use crate::{
  objs::{Alias, ChatTemplate, ChatTemplateId, LocalModelFile, RemoteModel},
  server::BODHI_HOME,
  service::{
    AppService, AppServiceFn, DataService, HfHubService, HubService, LocalDataService,
    MockDataService, MockHubService,
  },
  Repo,
};
use axum::{
  body::Body,
  http::{request::Builder, Request},
  response::Response,
};
use derive_new::new;
use dircpy::CopyBuilder;
use http_body_util::BodyExt;
use llama_server_bindings::{bindings::llama_server_disable_logging, disable_llama_log};
use reqwest::header::CONTENT_TYPE;
use rstest::fixture;
use serde::de::DeserializeOwned;
use std::{
  env, fs,
  path::{Path, PathBuf},
};
use std::{
  ffi::{c_char, c_void},
  io::Cursor,
  slice,
};
use tempfile::{tempdir, TempDir};
use tokio::sync::mpsc::Sender;
use tracing_subscriber::{fmt, EnvFilter};

pub static TEST_REPO: &str = "meta-llama/Meta-Llama-3-8B";
pub struct ConfigDirs(pub TempDir, pub PathBuf, pub &'static str);
pub static SNAPSHOT: &str = "5007652f7a641fe7170e0bad4f63839419bd9213";

#[fixture]
pub fn config_dirs(bodhi_home: TempDir) -> ConfigDirs {
  let repo_dir = TEST_REPO.replace('/', "--");
  let repo_dir = format!("configs--{repo_dir}");
  let repo_dir = bodhi_home.path().join(repo_dir);
  fs::create_dir_all(repo_dir.clone()).unwrap();
  ConfigDirs(bodhi_home, repo_dir, TEST_REPO)
}

#[fixture]
pub fn bodhi_home() -> TempDir {
  let bodhi_home = tempfile::Builder::new()
    .prefix("bodhi_home")
    .tempdir()
    .unwrap();
  env::set_var(BODHI_HOME, format!("{}", bodhi_home.path().display()));
  bodhi_home
}

pub trait ResponseTestExt {
  async fn json<T>(self) -> anyhow::Result<T>
  where
    T: DeserializeOwned;

  async fn text(self) -> anyhow::Result<String>;

  async fn sse<T>(self) -> anyhow::Result<Vec<T>>
  where
    T: DeserializeOwned;
}

impl ResponseTestExt for Response {
  async fn json<T>(self) -> anyhow::Result<T>
  where
    T: DeserializeOwned,
  {
    let bytes = self.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    let reader = Cursor::new(str.into_owned());
    let result = serde_json::from_reader::<_, T>(reader)?;
    Ok(result)
  }

  async fn text(self) -> anyhow::Result<String> {
    let bytes = self.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    Ok(str.into_owned())
  }

  async fn sse<T>(self) -> anyhow::Result<Vec<T>>
  where
    T: DeserializeOwned,
  {
    let text = self.text().await?;
    let lines = text.lines().peekable();
    let mut result = Vec::<T>::new();
    for line in lines {
      if line.is_empty() {
        continue;
      }
      let (_, value) = line.split_once(':').unwrap();
      let value = value.trim();
      let value = serde_json::from_reader::<_, T>(Cursor::new(value.to_owned()))?;
      result.push(value);
    }
    Ok(result)
  }
}

pub trait RequestTestExt {
  fn content_type_json(self) -> Self;

  fn json(self, value: serde_json::Value) -> Result<Request<Body>, anyhow::Error>;
}

impl RequestTestExt for Builder {
  fn content_type_json(self) -> Self {
    self.header(CONTENT_TYPE, "application/json")
  }

  fn json(self, value: serde_json::Value) -> std::result::Result<Request<Body>, anyhow::Error> {
    let content = serde_json::to_string(&value)?;
    let result = self.body(Body::from(content))?;
    Ok(result)
  }
}

pub(crate) fn init_test_tracing() {
  let filter = EnvFilter::from_default_env(); // Use RUST_LOG environment variable
  let subscriber = fmt::Subscriber::builder()
    .with_env_filter(filter) // Set the filter to use the RUST_LOG environment variable
    .finish();
  let _ = tracing::subscriber::set_global_default(subscriber);
}

pub(crate) fn disable_test_logging() {
  disable_llama_log();
  unsafe {
    llama_server_disable_logging();
  }
}

pub unsafe extern "C" fn test_callback(
  contents: *const c_char,
  size: usize,
  userdata: *mut c_void,
) -> usize {
  let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
  let input_str = match std::str::from_utf8(slice) {
    Ok(s) => s,
    Err(_) => return 0,
  };
  let user_data_str = unsafe { &mut *(userdata as *mut String) };
  user_data_str.push_str(input_str);
  size
}

pub unsafe extern "C" fn test_callback_stream(
  contents: *const c_char,
  size: usize,
  userdata: *mut c_void,
) -> usize {
  let slice = unsafe { slice::from_raw_parts(contents as *const u8, size) };
  let input_str = match std::str::from_utf8(slice) {
    Ok(s) => s,
    Err(_) => return 0,
  }
  .to_owned();
  let sender = unsafe { &mut *(userdata as *mut Sender<String>) }.clone();
  // TODO: handle closed receiver
  tokio::spawn(async move { sender.send(input_str).await.unwrap() });
  size
}

#[fixture]
pub(crate) fn hf_test_token_allowed() -> Option<String> {
  dotenv::from_filename(".env.test").ok().unwrap();
  Some(std::env::var("HF_TEST_TOKEN_ALLOWED").unwrap())
}

pub(crate) fn hf_test_token_public() -> Option<String> {
  dotenv::from_filename(".env.test").ok().unwrap();
  Some(std::env::var("HF_TEST_TOKEN_PUBLIC").unwrap())
}

#[fixture]
pub(crate) fn temp_hf_home() -> TempDir {
  let temp_dir = tempdir().expect("Failed to create a temporary directory");
  let dst_path = temp_dir.path().join("huggingface");
  copy_test_dir("tests/data/huggingface", &dst_path);
  temp_dir
}

#[fixture]
pub(crate) fn temp_bodhi_home() -> TempDir {
  let temp_dir = tempdir().expect("Failed to create a temporary directory");
  let dst_path = temp_dir.path().join("bodhi");
  copy_test_dir("tests/data/bodhi", &dst_path);
  temp_dir
}

fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  CopyBuilder::new(src_path, dst_path)
    .overwrite(true)
    .run()
    .unwrap();
}

pub struct HubServiceTuple(pub TempDir, pub PathBuf, pub HfHubService);

#[fixture]
pub fn hub_service(temp_hf_home: TempDir) -> HubServiceTuple {
  let hf_cache = temp_hf_home.path().join("huggingface/hub");
  let hub_service = HfHubService::new(hf_cache.clone(), false, None);
  HubServiceTuple(temp_hf_home, hf_cache, hub_service)
}

pub struct DataServiceTuple(pub TempDir, pub PathBuf, pub LocalDataService);

#[fixture]
pub fn data_service(temp_bodhi_home: TempDir) -> DataServiceTuple {
  let bodhi_home = temp_bodhi_home.path().join("bodhi");
  let data_service = LocalDataService::new(bodhi_home.clone());
  DataServiceTuple(temp_bodhi_home, bodhi_home, data_service)
}

pub struct AppServiceTuple(
  pub TempDir,
  pub TempDir,
  pub PathBuf,
  pub PathBuf,
  pub AppService,
);

#[fixture]
pub fn app_service_stub(
  hub_service: HubServiceTuple,
  data_service: DataServiceTuple,
) -> AppServiceTuple {
  let DataServiceTuple(temp_bodhi_home, bodhi_home, data_service) = data_service;
  let HubServiceTuple(temp_hf_home, hf_cache, hub_service) = hub_service;
  let service = AppService::new(hub_service, data_service);
  AppServiceTuple(temp_bodhi_home, temp_hf_home, bodhi_home, hf_cache, service)
}

#[derive(Debug, new)]
pub struct MockAppServiceFn {
  pub hub_service: MockHubService,
  pub data_service: MockDataService,
}

impl HubService for MockAppServiceFn {
  fn download(
    &self,
    repo: &str,
    filename: &str,
    force: bool,
  ) -> crate::service::Result<LocalModelFile> {
    self.hub_service.download(repo, filename, force)
  }

  fn list_local_models(&self) -> Vec<LocalModelFile> {
    self.hub_service.list_local_models()
  }

  fn find_local_model(
    &self,
    repo: &Repo,
    filename: &str,
    snapshot: &str,
  ) -> Option<LocalModelFile> {
    self.hub_service.find_local_model(repo, filename, snapshot)
  }
}

impl DataService for MockAppServiceFn {
  fn list_aliases(&self) -> crate::service::Result<Vec<Alias>> {
    self.data_service.list_aliases()
  }

  fn find_remote_model(&self, alias: &str) -> crate::service::Result<Option<RemoteModel>> {
    self.data_service.find_remote_model(alias)
  }

  fn save_alias(&self, alias: Alias) -> crate::service::Result<PathBuf> {
    self.data_service.save_alias(alias)
  }

  fn find_alias(&self, alias: &str) -> Option<Alias> {
    self.data_service.find_alias(alias)
  }

  fn list_remote_models(&self) -> crate::service::Result<Vec<RemoteModel>> {
    self.data_service.list_remote_models()
  }
}

// Implement AppServiceFn for the combined struct
impl AppServiceFn for MockAppServiceFn {}

#[fixture]
pub fn mock_app_service() -> MockAppServiceFn {
  MockAppServiceFn {
    hub_service: MockHubService::new(),
    data_service: MockDataService::new(),
  }
}

impl Default for ChatTemplate {
  fn default() -> Self {
    ChatTemplate::Id(ChatTemplateId::Llama3)
  }
}

impl LocalModelFile {
  pub fn never_download() -> LocalModelFile {
    LocalModelFile::new(
      PathBuf::from("/tmp/ignored/huggingface/hub"),
      Repo::try_new("MyFactory/testalias-neverdownload-gguf".to_string()).unwrap(),
      "testalias-neverdownload.Q8_0.gguf".to_string(),
      SNAPSHOT.to_string(),
      Some(22),
    )
  }

  pub fn testalias() -> LocalModelFile {
    LocalModelFile::new(
      PathBuf::from("/tmp/ignored/huggingface/hub"),
      Repo::try_new("MyFactory/testalias-gguf".to_string()).unwrap(),
      "testalias.Q8_0.gguf".to_string(),
      SNAPSHOT.to_string(),
      Some(22),
    )
  }
}
