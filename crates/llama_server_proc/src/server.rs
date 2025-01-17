use crate::error::{Result, ServerError};
use derive_builder::Builder;
use objs::{BuilderError, GptContextParams};
use reqwest::Response;
use serde_json::Value;
use std::{
  fmt::Display,
  io::{BufRead, BufReader},
  net::{IpAddr, Ipv4Addr},
  path::{Path, PathBuf},
  process::{Child, ChildStderr, ChildStdout, Command, Stdio},
  sync::Mutex,
  thread,
  time::Duration,
};
use tracing::{debug, warn};

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned", setter(into, strip_option), build_fn(error = BuilderError))]
pub struct LlamaServerArgs {
  pub model: PathBuf,
  pub alias: String,
  #[builder(default)]
  api_key: Option<String>,
  #[builder(default = "portpicker::pick_unused_port().unwrap_or(8080)")]
  port: u16,
  #[builder(default)]
  host: Option<String>,
  #[builder(default)]
  verbose: bool,
  #[builder(default)]
  no_webui: bool,
  #[builder(default)]
  embeddings: bool,
  #[builder(default)]
  seed: Option<u32>,
  #[builder(default)]
  n_ctx: Option<i32>,
  #[builder(default)]
  n_predict: Option<i32>,
  #[builder(default)]
  n_parallel: Option<i32>,
  #[builder(default)]
  n_keep: Option<i32>,
}

impl LlamaServerArgsBuilder {
  pub fn server_params(mut self, slf: &GptContextParams) -> Self {
    self.seed = Some(slf.n_seed);
    self.n_ctx = Some(slf.n_ctx);
    self.n_predict = Some(slf.n_predict);
    self.n_parallel = Some(slf.n_parallel);
    self.n_keep = Some(slf.n_keep);
    self
  }
}

impl Display for LlamaServerArgs {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_args().join(" "))
  }
}

impl LlamaServerArgs {
  // Convert the struct into command line arguments
  pub fn to_args(&self) -> Vec<String> {
    let mut args = vec![
      "--alias".to_string(),
      self.alias.clone(),
      "--model".to_string(),
      self.model.to_string_lossy().to_string(),
    ];

    if let Some(seed) = &self.seed {
      args.push("--seed".to_string());
      args.push(seed.to_string());
    }

    if let Some(api_key) = &self.api_key {
      args.push("--api-key".to_string());
      args.push(api_key.clone());
    }

    if let Some(host) = &self.host {
      args.push("--host".to_string());
      args.push(host.clone());
    }

    args.push("--port".to_string());
    args.push(self.port.to_string());

    if self.verbose {
      args.push("--verbose".to_string());
    }

    if self.no_webui {
      args.push("--no-webui".to_string());
    }

    if self.embeddings {
      args.push("--embeddings".to_string());
    }

    if let Some(n_ctx) = self.n_ctx {
      args.push("--ctx-size".to_string());
      args.push(n_ctx.to_string());
    }

    if let Some(n_predict) = self.n_predict {
      args.push("--n-predict".to_string());
      args.push(n_predict.to_string());
    }

    if let Some(n_parallel) = self.n_parallel {
      args.push("--parallel".to_string());
      args.push(n_parallel.to_string());
    }

    if let Some(n_keep) = self.n_keep {
      args.push("--n-keep".to_string());
      args.push(n_keep.to_string());
    }

    args
  }
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait Server: std::fmt::Debug + Send + Sync {
  async fn start(&self) -> Result<()>;

  fn get_server_args(&self) -> LlamaServerArgs;

  async fn stop(self: Box<Self>) -> Result<()>;

  async fn stop_unboxed(self) -> Result<()>;

  async fn chat_completions(&self, body: &Value) -> Result<Response>;

  async fn embeddings(&self, body: &Value) -> Result<Response>;

  async fn tokenize(&self, body: &Value) -> Result<Response>;

  async fn detokenize(&self, body: &Value) -> Result<Response>;
}

#[derive(Debug)]
pub struct LlamaServer {
  process: Mutex<Option<Child>>,
  client: reqwest::Client,
  base_url: String,
  executable_path: PathBuf,
  server_args: LlamaServerArgs,
}

impl LlamaServer {
  pub fn new<T: Into<LlamaServerArgs>>(executable_path: &Path, server_args: T) -> Result<Self> {
    let server_args = server_args.into();
    let port = server_args.port;
    let base_url = format!("http://127.0.0.1:{}", port);
    let client = reqwest::Client::builder()
      .pool_idle_timeout(Duration::from_secs(300))
      .pool_max_idle_per_host(32)
      .tcp_keepalive(Duration::from_secs(60))
      .tcp_nodelay(true)
      .local_address(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)))
      .build()?;

    Ok(Self {
      process: Mutex::new(None),
      client,
      base_url,
      executable_path: executable_path.into(),
      server_args,
    })
  }

  fn monitor_output(stdout: Option<ChildStdout>, stderr: Option<ChildStderr>) {
    // Monitor stdout in a separate thread
    if let Some(stdout) = stdout {
      thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
          match line {
            Ok(line) => debug!(target: "bodhi_server", "{}", line),
            Err(e) => warn!(target: "bodhi_server", "Error reading stdout: {}", e),
          }
        }
      });

      // Monitor stderr in a separate thread
      if let Some(stderr) = stderr {
        thread::spawn(move || {
          let reader = BufReader::new(stderr);
          for line in reader.lines() {
            match line {
              Ok(line) => warn!(target: "bodhi_server", "{}", line),
              Err(e) => warn!(target: "bodhi_server", "Error reading stderr: {}", e),
            }
          }
        });
      }
    }
  }

  async fn wait_for_server_ready(&self) -> Result<()> {
    let max_attempts = 300;
    for attempt in 0..max_attempts {
      match self
        .client
        .get(format!("{}/health", self.base_url))
        .send()
        .await
      {
        Ok(response) if response.status().is_success() => {
          return Ok(());
        }
        Ok(_) => {
          tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        Err(e) if attempt == max_attempts - 1 => {
          return Err(ServerError::HealthCheckError(e.to_string()));
        }
        Err(_) => {
          tokio::time::sleep(Duration::from_millis(1000)).await;
        }
      }
    }
    Err(ServerError::TimeoutError(max_attempts))
  }

  async fn proxy_request(&self, endpoint: &str, body: &Value) -> Result<Response> {
    let url = format!("{}{}", self.base_url, endpoint);
    let response = self.client.post(url).json(body).send().await?;

    Ok(response)
  }
}

impl Drop for LlamaServer {
  fn drop(&mut self) {
    let mut lock = self.process.lock().unwrap();
    if let Some(mut process) = lock.take() {
      if let Err(e) = process.kill() {
        warn!("failed to kill process: {}", e);
      }
      if let Err(e) = process.wait() {
        warn!("failed to wait for process: {}", e);
      }
    }
  }
}

#[async_trait::async_trait]
impl Server for LlamaServer {
  async fn start(&self) -> Result<()> {
    let args = self.server_args.to_args();
    let mut process = Command::new(&self.executable_path)
      .args(args)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|e| ServerError::StartupError(e.to_string()))?;
    let stdout = process.stdout.take();
    let stderr = process.stderr.take();

    *self.process.lock().unwrap() = Some(process);

    Self::monitor_output(stdout, stderr);
    self.wait_for_server_ready().await?;
    Ok(())
  }

  fn get_server_args(&self) -> LlamaServerArgs {
    self.server_args.clone()
  }

  async fn stop(self: Box<Self>) -> Result<()> {
    self.stop_unboxed().await
  }

  async fn stop_unboxed(self) -> Result<()> {
    let process = {
      let mut lock = self.process.lock().unwrap();
      lock.take()
    };

    if let Some(mut process) = process {
      process.kill()?;
      process.wait()?;
    }
    Ok(())
  }

  async fn chat_completions(&self, body: &Value) -> Result<Response> {
    self.proxy_request("/v1/chat/completions", body).await
  }

  async fn embeddings(&self, body: &Value) -> Result<Response> {
    self.proxy_request("/v1/embeddings", body).await
  }

  async fn tokenize(&self, body: &Value) -> Result<Response> {
    self.proxy_request("/v1/tokenize", body).await
  }

  async fn detokenize(&self, body: &Value) -> Result<Response> {
    self.proxy_request("/v1/detokenize", body).await
  }
}

impl From<&LlamaServerArgs> for LlamaServerArgs {
  fn from(args: &LlamaServerArgs) -> Self {
    args.clone()
  }
}
