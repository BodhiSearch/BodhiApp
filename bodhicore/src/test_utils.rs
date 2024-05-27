use crate::server::BODHI_HOME;
use axum::{
  body::Body,
  http::{request::Builder, Request},
  response::Response,
};
use http_body_util::BodyExt;
use llama_server_bindings::{bindings::llama_server_disable_logging, disable_llama_log};
use reqwest::header::CONTENT_TYPE;
use rstest::fixture;
use serde::de::DeserializeOwned;
use std::{env, fs, path::PathBuf};
use std::{
  ffi::{c_char, c_void},
  io::Cursor,
  slice,
};
use tempfile::TempDir;
use tokio::sync::mpsc::Sender;
use tracing_subscriber::{fmt, EnvFilter};

pub static TEST_REPO: &str = "meta-llama/Meta-Llama-3-8B";
pub static LLAMA2_CHAT_TEMPLATE: &str = r#"{% if messages[0]['role'] == 'system' -%}
  {% set loop_messages = messages[1:] %}{% set system_message = messages[0]['content'] -%}
{% else -%}
  {% set loop_messages = messages %}{% set system_message = false -%}
{% endif -%}
{% for message in loop_messages -%}
  {% if (message['role'] == 'user') != (loop.index0 % 2 == 0) -%}
    {{ raise_exception("Conversation roles must alternate user/assistant/user/assistant/...") }}
  {% endif -%}
  {% if loop.index0 == 0 and system_message != false -%}
    {% set content = '<<SYS>>\\n' + system_message + '\\n<</SYS>>\\n\\n' + message['content'] -%}
  {% else -%}
    {% set content = message['content'] -%}
  {% endif -%}
  {% if message['role'] == 'user' -%}
    {{ bos_token + '[INST] ' + content.strip() + ' [/INST]' -}}
  {% elif message['role'] == 'assistant' -%}
    {{ ' '  + content.strip() + ' ' + eos_token -}}
  {% endif -%}
{% endfor -%}
"#;
pub struct ConfigDirs(pub TempDir, pub PathBuf, pub &'static str);
pub const TEST_MODELS_YAML: &str = include_str!("../tests/data/test_models.yaml");

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
