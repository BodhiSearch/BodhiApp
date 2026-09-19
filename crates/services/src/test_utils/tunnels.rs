use crate::{
  CloudflaredCli, ConnectorHandle, ConnectorProcess, HttpMethod, HttpRequest, HttpResponse,
  OutputStream, RawOutput, TunnelIo, TunnelRuntimeError,
};
use base64::Engine;
use std::{
  path::{Path, PathBuf},
  sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
  },
  time::Duration,
};

pub const FAKE_TUNNEL_ID: &str = "11111111-2222-3333-4444-555555555555";
pub const FAKE_TUNNEL_TOKEN: &str = "ZmFrZS1jbG91ZGZsYXJlZC10dW5uZWwtdG9rZW4=";
pub const FAKE_ZONE_ID: &str = "zone-id";
pub const FAKE_ZONE: &str = "example.com";
pub const FAKE_BINARY: &str = "/fake/cloudflared";
pub const FAKE_ORIGIN_CERT: &str = "/fake/cert.pem";
pub const FAKE_METRICS_ADDR: &str = "127.0.0.1:42042";

#[derive(Debug)]
struct FakeState {
  calls: Mutex<Vec<String>>,
  credentials_files: Mutex<Vec<PathBuf>>,
  discarded: Mutex<Vec<PathBuf>>,
  existing_tunnel: Mutex<Option<String>>,
  zone_lookups: Mutex<usize>,
  dns_records: Mutex<String>,
  deleted_dns_records: Mutex<Vec<String>>,
  dns_gate: Mutex<Option<Arc<tokio::sync::Semaphore>>>,
  connector_args: Mutex<Option<String>>,
  connector_envs: Mutex<Vec<(String, String)>>,
  connector_stopped: Mutex<bool>,
  connector_exit: Mutex<Option<Option<i32>>>,
  connector_lines: Mutex<Option<Sender<(OutputStream, String)>>>,
  emit_metrics_line: Mutex<bool>,
  ready: Mutex<bool>,
}

impl Default for FakeState {
  fn default() -> Self {
    Self {
      calls: Mutex::new(Vec::new()),
      credentials_files: Mutex::new(Vec::new()),
      discarded: Mutex::new(Vec::new()),
      existing_tunnel: Mutex::new(None),
      zone_lookups: Mutex::new(0),
      dns_records: Mutex::new(r#"{"success":true,"result":[]}"#.to_string()),
      deleted_dns_records: Mutex::new(Vec::new()),
      dns_gate: Mutex::new(None),
      connector_args: Mutex::new(None),
      connector_envs: Mutex::new(Vec::new()),
      connector_stopped: Mutex::new(false),
      connector_exit: Mutex::new(None),
      connector_lines: Mutex::new(None),
      emit_metrics_line: Mutex::new(false),
      ready: Mutex::new(true),
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct FakeCloudflared {
  state: Arc<FakeState>,
}

impl FakeCloudflared {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_existing_tunnel(name: &str) -> Self {
    let fake = Self::new();
    *fake.state.existing_tunnel.lock().unwrap() = Some(name.to_string());
    fake
  }

  pub fn with_dns_records(self, records: &str) -> Self {
    *self.state.dns_records.lock().unwrap() = records.to_string();
    self
  }

  pub fn with_dns_gate(self) -> Self {
    *self.state.dns_gate.lock().unwrap() = Some(Arc::new(tokio::sync::Semaphore::new(0)));
    self
  }

  pub fn with_metrics_line(self) -> Self {
    *self.state.emit_metrics_line.lock().unwrap() = true;
    self
  }

  pub fn not_ready(self) -> Self {
    *self.state.ready.lock().unwrap() = false;
    self
  }

  pub fn release_dns(&self) {
    if let Some(gate) = self.state.dns_gate.lock().unwrap().as_ref() {
      gate.add_permits(1);
    }
  }

  pub fn exit_connector(&self, code: Option<i32>) {
    *self.state.connector_exit.lock().unwrap() = Some(code);
  }

  pub fn calls(&self) -> Vec<String> {
    self.state.calls.lock().unwrap().clone()
  }

  pub fn credentials_files(&self) -> Vec<PathBuf> {
    self.state.credentials_files.lock().unwrap().clone()
  }

  pub fn discarded_files(&self) -> Vec<PathBuf> {
    self.state.discarded.lock().unwrap().clone()
  }

  pub fn connector_args(&self) -> Option<String> {
    self.state.connector_args.lock().unwrap().clone()
  }

  pub fn connector_token(&self) -> Option<String> {
    self
      .state
      .connector_envs
      .lock()
      .unwrap()
      .iter()
      .find(|(key, _)| key == "TUNNEL_TOKEN")
      .map(|(_, value)| value.clone())
  }

  pub fn connector_stopped(&self) -> bool {
    *self.state.connector_stopped.lock().unwrap()
  }

  pub fn zone_lookups(&self) -> usize {
    *self.state.zone_lookups.lock().unwrap()
  }

  pub fn deleted_dns_records(&self) -> Vec<String> {
    self.state.deleted_dns_records.lock().unwrap().clone()
  }

  pub fn binary_path() -> PathBuf {
    PathBuf::from(FAKE_BINARY)
  }

  pub fn origin_cert_path() -> PathBuf {
    PathBuf::from(FAKE_ORIGIN_CERT)
  }

  pub fn origin_cert_pem() -> String {
    let payload =
      format!(r#"{{"zoneID":"{FAKE_ZONE_ID}","accountID":"account-id","apiToken":"token"}}"#);
    let encoded = base64::engine::general_purpose::STANDARD.encode(payload);
    format!("-----BEGIN ARGO TUNNEL TOKEN-----\n{encoded}\n-----END ARGO TUNNEL TOKEN-----\n")
  }
}

fn stdout(body: impl Into<String>) -> RawOutput {
  RawOutput {
    success: true,
    code: Some(0),
    stdout: body.into().into_bytes(),
    stderr: Vec::new(),
  }
}

#[async_trait::async_trait]
impl CloudflaredCli for FakeCloudflared {
  async fn run(
    &self,
    _binary: PathBuf,
    args: Vec<String>,
    _envs: Vec<(String, String)>,
    _timeout: Duration,
  ) -> Result<RawOutput, TunnelRuntimeError> {
    self.state.calls.lock().unwrap().push(args.join(" "));
    if args.first().map(String::as_str) == Some("--version") {
      return Ok(stdout(
        "cloudflared version 2026.9.1 (built 2026-09-01-0000 UTC)\n",
      ));
    }
    match args.get(1).map(String::as_str) {
      Some("list") => {
        let existing = self.state.existing_tunnel.lock().unwrap().clone();
        Ok(stdout(match existing {
          Some(name) => format!(r#"[{{"id":"{FAKE_TUNNEL_ID}","name":"{name}"}}]"#),
          None => "[]".to_string(),
        }))
      }
      Some("create") => {
        if let Some(index) = args.iter().position(|arg| arg == "--credentials-file") {
          if let Some(path) = args.get(index + 1) {
            self
              .state
              .credentials_files
              .lock()
              .unwrap()
              .push(PathBuf::from(path));
          }
        }
        let name = args.last().cloned().unwrap_or_default();
        *self.state.existing_tunnel.lock().unwrap() = Some(name.clone());
        Ok(stdout(format!(
          r#"{{"id":"{FAKE_TUNNEL_ID}","name":"{name}"}}"#
        )))
      }
      Some("token") => Ok(stdout(format!("{FAKE_TUNNEL_TOKEN}\n"))),
      Some("route") => {
        let gate = self.state.dns_gate.lock().unwrap().clone();
        if let Some(gate) = gate {
          let _permit = gate.acquire().await;
        }
        Ok(stdout(""))
      }
      _ => Ok(RawOutput {
        success: false,
        code: Some(1),
        stdout: Vec::new(),
        stderr: format!(
          "fake cloudflared: unsupported invocation: {}",
          args.join(" ")
        )
        .into_bytes(),
      }),
    }
  }
}

#[derive(Debug)]
struct FakeConnectorHandle {
  state: Arc<FakeState>,
  output: Option<Receiver<(OutputStream, String)>>,
}

impl ConnectorHandle for FakeConnectorHandle {
  fn try_wait(&mut self) -> Result<Option<Option<i32>>, TunnelRuntimeError> {
    Ok(*self.state.connector_exit.lock().unwrap())
  }

  fn stop(&mut self) -> Result<(), TunnelRuntimeError> {
    *self.state.connector_stopped.lock().unwrap() = true;
    Ok(())
  }

  fn take_output(&mut self) -> Option<Receiver<(OutputStream, String)>> {
    self.output.take()
  }
}

impl ConnectorProcess for FakeCloudflared {
  fn spawn(
    &self,
    _binary: PathBuf,
    args: Vec<String>,
    envs: Vec<(String, String)>,
  ) -> Result<Box<dyn ConnectorHandle>, TunnelRuntimeError> {
    *self.state.connector_args.lock().unwrap() = Some(args.join(" "));
    *self.state.connector_envs.lock().unwrap() = envs;
    *self.state.connector_stopped.lock().unwrap() = false;
    *self.state.connector_exit.lock().unwrap() = None;
    let (sender, receiver) = channel();
    if *self.state.emit_metrics_line.lock().unwrap() {
      let _ = sender.send((
        OutputStream::Stdout,
        format!("2026-09-18T00:00:00Z INF Starting metrics server on {FAKE_METRICS_ADDR}/metrics"),
      ));
    }
    *self.state.connector_lines.lock().unwrap() = Some(sender);
    Ok(Box::new(FakeConnectorHandle {
      state: self.state.clone(),
      output: Some(receiver),
    }))
  }
}

#[async_trait::async_trait]
impl TunnelIo for FakeCloudflared {
  async fn read_to_string(&self, path: PathBuf) -> Result<String, TunnelRuntimeError> {
    if path == Self::origin_cert_path() {
      return Ok(Self::origin_cert_pem());
    }
    Err(TunnelRuntimeError::Io(format!(
      "no such file: {}",
      path.display()
    )))
  }

  fn is_file(&self, path: &Path) -> bool {
    path == Self::binary_path() || path == Self::origin_cert_path()
  }

  fn home_dir(&self) -> Option<PathBuf> {
    None
  }

  fn path_candidates(&self) -> Vec<PathBuf> {
    Vec::new()
  }

  fn standard_locations(&self) -> Vec<PathBuf> {
    Vec::new()
  }

  async fn prepare_scratch_dir(&self, _dir: PathBuf) -> Result<(), TunnelRuntimeError> {
    Ok(())
  }

  async fn discard_scratch(&self, path: PathBuf) {
    self.state.discarded.lock().unwrap().push(path);
  }

  async fn http(&self, request: HttpRequest) -> Result<HttpResponse, TunnelRuntimeError> {
    let url = request.url.clone();
    if url.ends_with("/ready") {
      let ready = *self.state.ready.lock().unwrap();
      return Ok(HttpResponse {
        status: if ready { 200 } else { 503 },
        body: r#"{"readyConnections":1}"#.to_string(),
      });
    }
    if url.contains("/dns_records") {
      if request.method == HttpMethod::Delete {
        let id = url.rsplit('/').next().unwrap_or_default().to_string();
        self.state.deleted_dns_records.lock().unwrap().push(id);
        return Ok(HttpResponse {
          status: 200,
          body: r#"{"success":true}"#.to_string(),
        });
      }
      return Ok(HttpResponse {
        status: 200,
        body: self.state.dns_records.lock().unwrap().clone(),
      });
    }
    if url.contains("/zones/") {
      *self.state.zone_lookups.lock().unwrap() += 1;
      return Ok(HttpResponse {
        status: 200,
        body: format!(r#"{{"success":true,"result":{{"name":"{FAKE_ZONE}"}}}}"#),
      });
    }
    Err(TunnelRuntimeError::Io(format!("unexpected request: {url}")))
  }
}
