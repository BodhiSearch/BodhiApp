use bodhicore::{
  bindings::{disable_llama_log, llama_server_disable_logging},
  db::{DbPool, DbService, TimeService},
  server::{build_routes, build_server_handle, ServerHandle, ServerParams},
  service::{AppService, AppServiceFn, HfHubService, LocalDataService},
  BodhiError, SharedContextRw, SharedContextRwFn,
};
use dircpy::CopyBuilder;
use futures_util::{future::BoxFuture, FutureExt};
use llama_server_bindings::GptParamsBuilder;
use rstest::fixture;
use serde_json::Value;
use std::path::Path;
use std::{fs::File, sync::Arc};
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
async fn db_service() -> (TempDir, DbService) {
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
fn tinyllama() -> (TempDir, Arc<dyn AppServiceFn>) {
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
fn setup_logs() {
  disable_llama_log();
  unsafe {
    llama_server_disable_logging();
  }
}

#[fixture]
fn setup(#[from(setup_logs)] _setup_logs: ()) {}

#[fixture]
#[awt]
async fn live_server(
  #[from(setup)] _setup: (),
  #[future] db_service: (TempDir, DbService),
  tinyllama: &(TempDir, Arc<dyn AppServiceFn>),
) -> anyhow::Result<TestServerHandle> {
  let host = String::from("127.0.0.1");
  let port = rand::random::<u16>();
  let server_params = ServerParams {
    host: host.clone(),
    port,
  };
  let ServerHandle {
    server,
    shutdown,
    ready_rx,
  } = build_server_handle(server_params);
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
  let (_temp_db_home, db_service) = db_service;
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
    _temp_db_home,
  })
}

struct TestServerHandle {
  host: String,
  port: u16,
  shutdown: Sender<()>,
  join: JoinHandle<Result<(), BodhiError>>,
  _temp_db_home: TempDir,
}

#[rstest::rstest]
#[awt]
#[serial_test::serial(live_server)]
#[tokio::test]
async fn test_live_chat_completions(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    host,
    port,
    shutdown,
    join,
    _temp_db_home,
  } = live_server?;
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");
  let client = reqwest::Client::new();
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .json(&serde_json::json!({
      "model": "tinyllama:instruct",
      "seed": 42,
      "messages": [
        {
          "role": "system",
          "content": "You are a helpful assistant."
        },
        {
          "role": "user",
          "content": "Answer in one word. What day comes after Monday?"
        }
      ]
    }))
    .send()
    .await?
    .json::<Value>()
    .await?;
  assert_eq!(
    "Answer: Tuesday",
    response["choices"][0]["message"]["content"]
  );
  let expected: Value = serde_json::from_str(
    r#"[{"finish_reason":"stop","index":0,"message":{"content":"Answer: Tuesday","role":"assistant"}}]"#,
  )?;
  assert_eq!(expected, response["choices"]);
  assert_eq!("tinyllama:instruct", response["model"]);
  shutdown.send(()).unwrap();
  (join.await?)?;
  Ok(())
}

#[rstest::rstest]
#[awt]
#[serial_test::serial(live_server)]
#[tokio::test]
async fn test_live_chat_completions_stream(
  #[future] live_server: anyhow::Result<TestServerHandle>,
) -> anyhow::Result<()> {
  let TestServerHandle {
    host,
    port,
    shutdown,
    join,
    _temp_db_home,
  } = live_server?;
  let chat_endpoint = format!("http://{host}:{port}/v1/chat/completions");
  let client = reqwest::Client::new();
  let response = client
    .post(&chat_endpoint)
    .header("Content-Type", "application/json")
    .json(&serde_json::json!({
      "model": "tinyllama:instruct",
      "seed": 42,
      "stream": true,
      "messages": [
        {
          "role": "system",
          "content": "You are a helpful assistant."
        },
        {
          "role": "user",
          "content": "Answer in one word. What day comes after Monday?"
        }
      ]
    }))
    .send()
    .await?
    .text()
    .await?;
  let streams = response
    .lines()
    .filter_map(|line| {
      if line.is_empty() {
        None
      } else if line.starts_with("data: ") {
        let value: Value = serde_json::from_str(line.strip_prefix("data: ").unwrap()).unwrap();
        Some(value)
      } else {
        None
      }
    })
    .collect::<Vec<_>>();
  for (index, content) in ["Answer", ":", " T", "ues", "day"].iter().enumerate() {
    // TODO: have index 0, 1, 2 ... from llama.cpp
    let expected: Value = serde_json::from_str(&format!(
      r#"[{{"delta":{{"content":"{}"}},"finish_reason":null,"index":0}}]"#,
      content
    ))?;
    assert_eq!(expected, streams.get(index).unwrap()["choices"]);
  }
  let expected: Value = serde_json::from_str(r#"[{"delta":{},"finish_reason":"stop","index":0}]"#)?;
  assert_eq!(expected, streams.get(5).unwrap()["choices"]);
  shutdown
    .send(())
    .map_err(|_| anyhow::anyhow!("send error"))?;
  (join.await?)?;
  Ok(())
}
