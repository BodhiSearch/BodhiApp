use anyhow::{anyhow, Result};
use bodhi::{build_server_handle, ServerHandle, ServerParams, BODHI_HOME};
use lazy_static::lazy_static;
use llama_server_bindings::GptParams;
use rstest::fixture;
use tempdir::TempDir;
use tokio::sync::Mutex;

lazy_static! {
  pub static ref LLAMA_BACKEND_LOCK: Mutex<()> = Mutex::new(());
}

pub struct TestServerHandle {
  pub host: String,
  pub port: u16,
  pub shutdown: tokio::sync::oneshot::Sender<()>,
  pub join: tokio::task::JoinHandle<Result<()>>,
  pub bodhi_home: TempDir,
}

#[fixture]
pub fn bodhi_home() -> TempDir {
  let bodhi_home = tempdir::TempDir::new("bodhi_home").unwrap();
  std::env::set_var(BODHI_HOME, format!("{}", bodhi_home.path().display()));
  bodhi_home
}

#[fixture]
pub async fn test_server(bodhi_home: TempDir) -> anyhow::Result<TestServerHandle> {
  let _guard = LLAMA_BACKEND_LOCK.lock().await;
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
  } = build_server_handle(server_params)?;
  let model_path = dirs::home_dir()
    .ok_or_else(|| anyhow!("unable to locate home dir"))?
    .join(".cache/huggingface/llama-2-7b-chat.Q4_K_M.gguf")
    .canonicalize()?
    .to_str()
    .unwrap()
    .to_owned();
  let mut gpt_params = GptParams::default();
  gpt_params.model = Some(model_path);
  let join = tokio::spawn(server.start(gpt_params));
  ready_rx.await?;
  Ok(TestServerHandle {
    host,
    port,
    shutdown,
    join,
    bodhi_home,
  })
}
