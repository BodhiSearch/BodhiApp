use crate::error::{Result, ServerError};
use derive_builder::Builder;
use reqwest::Response;
use serde_json::Value;
use std::{
  net::{IpAddr, Ipv4Addr},
  path::PathBuf,
  process::Stdio,
  time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tracing::{debug, warn};

#[derive(Debug, Builder)]
#[builder(pattern = "owned")]
pub struct LlamaServerArgs {
  #[builder(setter(into))]
  model: PathBuf,
  #[builder(default, setter(into, strip_option))]
  api_key: Option<String>,
  #[builder(default = "portpicker::pick_unused_port().unwrap_or(8080)")]
  port: u16,
  #[builder(default, setter(into, strip_option))]
  host: Option<String>,
  #[builder(default)]
  verbose: bool,
  #[builder(default)]
  no_webui: bool,
  #[builder(default)]
  embeddings: bool,
}

impl LlamaServerArgs {
  // Convert the struct into command line arguments
  pub fn to_args(&self) -> Vec<String> {
    let mut args = Vec::new();

    // Required arg
    args.push("--model".to_string());
    args.push(self.model.to_string_lossy().to_string());

    // Optional args
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

    args
  }
}

#[async_trait::async_trait]
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait Server {
  async fn start(&mut self) -> Result<()>;

  async fn shutdown(&mut self) -> Result<()>;

  async fn chat_completions(&self, body: Value) -> Result<Response>;

  async fn embeddings(&self, body: Value) -> Result<Response>;

  async fn tokenize(&self, body: Value) -> Result<Response>;

  async fn detokenize(&self, body: Value) -> Result<Response>;
}

pub struct LlamaServer {
  process: Option<TokioChild>,
  client: reqwest::Client,
  base_url: String,
  executable_path: PathBuf,
  server_args: LlamaServerArgs,
}

impl LlamaServer {
  pub fn new(executable_path: PathBuf, server_args: LlamaServerArgs) -> Result<Self> {
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
      process: None,
      client,
      base_url,
      executable_path,
      server_args,
    })
  }

  fn monitor_output<R1, R2>(stdout: R1, stderr: R2)
  where
    R1: AsyncRead + Unpin + Send + 'static,
    R2: AsyncRead + Unpin + Send + 'static,
  {
    // Monitor stdout
    tokio::spawn(async move {
      let mut stdout = BufReader::new(stdout).lines();
      while let Ok(Some(line)) = stdout.next_line().await {
        debug!(target: "bodhi_server", "{}", line);
      }
    });

    // Monitor stderr
    tokio::spawn(async move {
      let mut stderr = BufReader::new(stderr).lines();
      while let Ok(Some(line)) = stderr.next_line().await {
        warn!(target: "bodhi_server", "{}", line);
      }
    });
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
          tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(e) if attempt == max_attempts - 1 => {
          return Err(ServerError::HealthCheckError(e.to_string()));
        }
        Err(_) => {
          tokio::time::sleep(Duration::from_millis(100)).await;
        }
      }
    }
    Err(ServerError::TimeoutError(max_attempts))
  }

  async fn proxy_request(&self, endpoint: &str, body: Value) -> Result<Response> {
    let url = format!("{}{}", self.base_url, endpoint);
    let response = self.client.post(url).json(&body).send().await?;

    Ok(response)
  }

  pub async fn chat_completions(&self, body: Value) -> Result<Response> {
    self.proxy_request("/v1/chat/completions", body).await
  }

  pub async fn embeddings(&self, body: Value) -> Result<Response> {
    self.proxy_request("/v1/embeddings", body).await
  }

  pub async fn tokenize(&self, body: Value) -> Result<Response> {
    self.proxy_request("/v1/tokenize", body).await
  }

  pub async fn detokenize(&self, body: Value) -> Result<Response> {
    self.proxy_request("/v1/detokenize", body).await
  }

  pub async fn shutdown(&mut self) -> Result<()> {
    if let Some(mut process) = self.process.take() {
      process.kill().await?;
      process.wait().await?;
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl Server for LlamaServer {
  async fn start(&mut self) -> Result<()> {
    let args = self.server_args.to_args();
    let mut process = TokioCommand::new(&self.executable_path)
      .args(args)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|e| ServerError::StartupError(e.to_string()))?;

    // Set up stdout/stderr forwarding to tracing
    let stdout = BufReader::new(process.stdout.take().unwrap());
    let stderr = BufReader::new(process.stderr.take().unwrap());
    // Start stdout/stderr monitoring tasks
    Self::monitor_output(stdout, stderr);

    self.process = Some(process);

    // Wait for server to be ready via health check
    self.wait_for_server_ready().await?;

    Ok(())
  }

  async fn shutdown(&mut self) -> Result<()> {
    if let Some(mut process) = self.process.take() {
      process.kill().await?;
      process.wait().await?;
    }
    Ok(())
  }

  async fn chat_completions(&self, body: Value) -> Result<Response> {
    self.proxy_request("/v1/chat/completions", body).await
  }

  async fn embeddings(&self, body: Value) -> Result<Response> {
    self.proxy_request("/v1/embeddings", body).await
  }

  async fn tokenize(&self, body: Value) -> Result<Response> {
    self.proxy_request("/v1/tokenize", body).await
  }

  async fn detokenize(&self, body: Value) -> Result<Response> {
    self.proxy_request("/v1/detokenize", body).await
  }
}
