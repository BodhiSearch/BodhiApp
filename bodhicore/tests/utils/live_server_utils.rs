use bodhicore::{
  bindings::{disable_llama_log, llama_server_disable_logging},
  db::{DbPool, DbService, TimeService},
  server::{build_routes, build_server_handle, ServerHandle},
  service::{AppService, AppServiceFn, HfHubService, LocalDataService},
  BodhiError, SharedContextRw, SharedContextRwFn,
};
use dircpy::CopyBuilder;
use futures_util::{future::BoxFuture, FutureExt};
use llama_server_bindings::GptParamsBuilder;
use rstest::fixture;
use std::{fs::File, path::Path, sync::Arc};
use tempfile::TempDir;
use tokio::{sync::oneshot::Sender, task::JoinHandle};

pub fn copy_test_dir(src: &str, dst_path: &Path) {
  let src_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(src);
  CopyBuilder::new(src_path, dst_path)
    .overwrite(true)
    .run()
    .unwrap();
}

#[fixture]
pub async fn db_service() -> (TempDir, DbService) {
  let tempdir = tempfile::tempdir().unwrap();
  let db_path = tempdir.path().join("test_live_db.sqlite");
  File::create_new(&db_path).unwrap();
  let pool = DbPool::connect(&format!("sqlite:{}", db_path.display()))
    .await
    .unwrap();
  sqlx::migrate!("./migrations").run(&pool).await.unwrap();
  let db_service = DbService::new(pool, Arc::new(TimeService));
  (tempdir, db_service)
}

#[fixture]
#[once]
pub fn tinyllama() -> (TempDir, Arc<dyn AppServiceFn>) {
  let temp_dir = tempfile::tempdir().unwrap();
  let cache_dir = temp_dir.path().join(".cache");
  std::fs::create_dir_all(&cache_dir).unwrap();

  copy_test_dir("tests/data/live", &cache_dir);

  let bodhi_home = cache_dir.join("bodhi");
  let hf_cache = cache_dir.join("huggingface").join("hub");
  let data_service = LocalDataService::new(bodhi_home);
  let hub_service = HfHubService::new(hf_cache, false, None);
  let app_service = AppService::new(hub_service, data_service);
  (temp_dir, Arc::new(app_service))
}

#[fixture]
pub fn setup_logs() {
  disable_llama_log();
  unsafe {
    llama_server_disable_logging();
  }
}

#[fixture]
pub fn setup(#[from(setup_logs)] _setup_logs: ()) {}

#[fixture]
#[awt]
pub async fn live_server(
  #[from(setup)] _setup: (),
  #[future] db_service: (TempDir, DbService),
  tinyllama: &(TempDir, Arc<dyn AppServiceFn>),
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::random::<u16>();
  let ServerHandle {
    server,
    shutdown,
    ready_rx,
  } = build_server_handle(&host, port);
  let (_, app_service) = tinyllama;

  let alias = app_service.find_alias("tinyllama:instruct").unwrap();
  let local_file = app_service
    .find_local_file(&alias.repo, &alias.filename, &alias.snapshot)?
    .unwrap();

  let mut gpt_params = GptParamsBuilder::default()
    .model(local_file.path().display().to_string())
    .seed(42u32)
    .build()?;
  alias.context_params.update(&mut gpt_params);
  let shared_ctx = SharedContextRw::new_shared_rw(Some(gpt_params)).await?;
  let (temp_db_home, db_service) = db_service;
  let ctx = Arc::new(shared_ctx);
  let router = build_routes(ctx.clone(), app_service.clone(), Arc::new(db_service));

  let callback: Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + 'static> = Box::new(|| {
    async move {
      if let Err(err) = ctx.try_stop().await {
        tracing::warn!(err = ?err, "error unloading context");
      }
    }
    .boxed()
  });
  let join = tokio::spawn(server.start_new(router, Some(callback)));
  ready_rx.await?;
  Ok(TestServerHandle {
    host,
    port,
    shutdown,
    join,
    temp_db_home,
  })
}

pub struct TestServerHandle {
  pub host: String,
  pub port: u16,
  pub shutdown: Sender<()>,
  pub join: JoinHandle<Result<(), BodhiError>>,
  pub temp_db_home: TempDir,
}
