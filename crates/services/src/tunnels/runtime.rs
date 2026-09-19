use std::{
  path::{Path, PathBuf},
  sync::mpsc::Receiver,
  time::Duration,
};

#[derive(Debug, thiserror::Error)]
pub enum TunnelRuntimeError {
  #[error("{0}")]
  Io(String),
  #[error("the operation did not complete in time")]
  Timeout,
}

#[derive(Debug, Clone, Default)]
pub struct RawOutput {
  pub success: bool,
  pub code: Option<i32>,
  pub stdout: Vec<u8>,
  pub stderr: Vec<u8>,
}

impl RawOutput {
  pub fn stdout_utf8(&self) -> std::borrow::Cow<'_, str> {
    String::from_utf8_lossy(&self.stdout)
  }

  pub fn stderr_utf8(&self) -> std::borrow::Cow<'_, str> {
    String::from_utf8_lossy(&self.stderr)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
  Stdout,
  Stderr,
}

impl OutputStream {
  pub fn as_str(self) -> &'static str {
    match self {
      OutputStream::Stdout => "stdout",
      OutputStream::Stderr => "stderr",
    }
  }
}

/// Plain status and body, so the seam names no HTTP type and a fake needs none to answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
  Get,
  Delete,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
  pub method: HttpMethod,
  pub url: String,
  pub bearer: Option<String>,
  pub query: Vec<(String, String)>,
}

impl HttpRequest {
  pub fn get(url: impl Into<String>) -> Self {
    Self {
      method: HttpMethod::Get,
      url: url.into(),
      bearer: None,
      query: Vec::new(),
    }
  }

  pub fn delete(url: impl Into<String>) -> Self {
    Self {
      method: HttpMethod::Delete,
      url: url.into(),
      bearer: None,
      query: Vec::new(),
    }
  }

  pub fn bearer(mut self, token: impl Into<String>) -> Self {
    self.bearer = Some(token.into());
    self
  }

  pub fn query(mut self, key: &str, value: &str) -> Self {
    self.query.push((key.to_string(), value.to_string()));
    self
  }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
  pub status: u16,
  pub body: String,
}

impl HttpResponse {
  pub fn is_success(&self) -> bool {
    (200..300).contains(&self.status)
  }
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait CloudflaredCli: std::fmt::Debug + Send + Sync {
  async fn run(
    &self,
    binary: PathBuf,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    timeout: Duration,
  ) -> Result<RawOutput, TunnelRuntimeError>;
}

pub trait ConnectorHandle: std::fmt::Debug + Send {
  fn try_wait(&mut self) -> Result<Option<Option<i32>>, TunnelRuntimeError>;
  fn stop(&mut self) -> Result<(), TunnelRuntimeError>;
  fn take_output(&mut self) -> Option<Receiver<(OutputStream, String)>>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
pub trait ConnectorProcess: std::fmt::Debug + Send + Sync {
  fn spawn(
    &self,
    binary: PathBuf,
    args: Vec<String>,
    envs: Vec<(String, String)>,
  ) -> Result<Box<dyn ConnectorHandle>, TunnelRuntimeError>;
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TunnelIo: std::fmt::Debug + Send + Sync {
  async fn read_to_string(&self, path: PathBuf) -> Result<String, TunnelRuntimeError>;
  fn is_file(&self, path: &Path) -> bool;
  fn home_dir(&self) -> Option<PathBuf>;
  fn path_candidates(&self) -> Vec<PathBuf>;
  fn standard_locations(&self) -> Vec<PathBuf>;
  async fn prepare_scratch_dir(&self, dir: PathBuf) -> Result<(), TunnelRuntimeError>;
  async fn discard_scratch(&self, path: PathBuf);
  async fn http(&self, request: HttpRequest) -> Result<HttpResponse, TunnelRuntimeError>;
}
