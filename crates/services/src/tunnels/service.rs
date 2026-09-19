use super::runtime::{
  CloudflaredCli, ConnectorHandle, ConnectorProcess, HttpRequest, OutputStream, RawOutput,
  TunnelIo, TunnelRuntimeError,
};
use super::runtime_impl::{SystemCloudflaredCli, SystemConnectorProcess, SystemTunnelIo};
use crate::shared_objs::log::scrub_secrets;
use crate::RESOURCE_CLIENT_PREFIX;
use crate::{
  AuthService, EnableTunnelRequest, RemoteAccessInfo, RemoteAccessProvider, RemoteAccessState,
  SettingService, TenantService, TunnelAuthSyncState, TunnelAuthSyncStatus, TunnelBinaryStatus,
  TunnelCheckState, TunnelConnectionState, TunnelErrorCode, TunnelLoginStatus, TunnelPathSource,
  TunnelSetupRequest, TunnelStatus, UpdateTunnelPreferencesRequest, BODHI_TUNNEL_AUTO_RECONNECT,
  BODHI_TUNNEL_CLOUDFLARED_PATH, BODHI_TUNNEL_HOST, BODHI_TUNNEL_ORIGIN_CERT, LOGIN_CALLBACK_PATH,
};
use base64::Engine;
use std::sync::Arc;
use std::{
  path::{Path, PathBuf},
  sync::{mpsc::Receiver, Mutex, MutexGuard},
  thread,
  time::{Duration, Instant},
};
use tracing::{info, warn};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = crate::AppError)]
pub enum TunnelError {
  #[error("Cloudflare Tunnel is disabled. Set BODHI_TUNNEL=true to enable it for this app.")]
  #[error_meta(error_type = crate::ErrorType::BadRequest)]
  Disabled,
  #[error("Enter a valid public hostname, such as bodhi.example.com.")]
  #[error_meta(error_type = crate::ErrorType::BadRequest)]
  InvalidHostname,
  #[error("A DNS record already exists for this address. Confirm replacement to continue.")]
  #[error_meta(error_type = crate::ErrorType::BadRequest)]
  DnsConflict,
  #[error("A usable cloudflared executable was not found.")]
  #[error_meta(error_type = crate::ErrorType::BadRequest)]
  MissingBinary,
  #[error("Cloudflare origin certificate not found. Run `cloudflared tunnel login` first.")]
  #[error_meta(error_type = crate::ErrorType::BadRequest)]
  MissingOriginCertificate,
  #[error("This instance has no authorization client, so its tunnel cannot be identified.")]
  #[error_meta(error_type = crate::ErrorType::InternalServer)]
  UnidentifiedInstance,
  #[error("cloudflared failed: {0}")]
  #[error_meta(error_type = crate::ErrorType::InternalServer)]
  Command(String),
  #[error("Tunnel provisioning failed: {0}")]
  #[error_meta(error_type = crate::ErrorType::BadRequest)]
  Provisioning(String),
  #[error("cloudflared did not respond in time.")]
  #[error_meta(error_type = crate::ErrorType::InternalServer)]
  ProvisioningTimeout,
  #[error("Cloudflare Tunnel runtime state is unavailable after an internal failure.")]
  #[error_meta(error_type = crate::ErrorType::InternalServer)]
  RuntimePoisoned,
}

type Result<T> = std::result::Result<T, TunnelError>;

impl From<TunnelRuntimeError> for TunnelError {
  fn from(error: TunnelRuntimeError) -> Self {
    match error {
      TunnelRuntimeError::Timeout => TunnelError::ProvisioningTimeout,
      TunnelRuntimeError::Io(message) => TunnelError::Command(message),
    }
  }
}

const CLOUDFLARE_API: &str = "https://api.cloudflare.com/client/v4";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const VERSION_TIMEOUT: Duration = Duration::from_secs(10);
const SUPERVISOR_INTERVAL: Duration = Duration::from_secs(1);
/// Slower than the exit check: a loopback HTTP call, and seconds of lag on "connected" are harmless.
const READY_PROBE_INTERVAL: Duration = Duration::from_secs(5);
const BINARY_PROBE_TTL: Duration = Duration::from_secs(30);
const ZONE_PROBE_TTL: Duration = Duration::from_secs(300);
const METRICS_LOG_MARKER: &str = "Starting metrics server on ";
/// Byte-identical across enable/disable cycles, or the update accumulates instead of replacing.
const TUNNEL_GATEWAY: &str = "cloudflared";

#[derive(Debug, serde::Deserialize)]
struct OriginCertificate {
  #[serde(rename = "zoneID")]
  zone_id: String,
  #[allow(dead_code)]
  #[serde(rename = "accountID")]
  account_id: String,
  #[serde(rename = "apiToken")]
  api_token: String,
}

#[derive(Debug, serde::Deserialize)]
struct CloudflareZoneResponse {
  success: bool,
  result: CloudflareZone,
}

#[derive(Debug, serde::Deserialize)]
struct CloudflareZone {
  name: String,
}

#[derive(Debug, serde::Deserialize)]
struct CloudflareDnsResponse {
  success: bool,
  result: Vec<CloudflareDnsRecord>,
}

#[derive(Debug, serde::Deserialize)]
struct CloudflareDnsRecord {
  id: String,
  content: String,
}

#[derive(Debug)]
struct RunningTunnel {
  handle: Box<dyn ConnectorHandle>,
  metrics_addr: Arc<Mutex<Option<String>>>,
  hostname: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeSnapshot {
  pub(crate) state: TunnelConnectionState,
  pub(crate) error_code: Option<String>,
}

#[derive(Debug)]
struct TunnelRuntime {
  state: TunnelConnectionState,
  error_code: Option<String>,
  error_message: Option<String>,
  auth_sync: TunnelAuthSyncStatus,
  running: Option<RunningTunnel>,
  generation: u64,
  events: tokio::sync::broadcast::Sender<RuntimeSnapshot>,
}

impl Default for TunnelRuntime {
  fn default() -> Self {
    Self {
      state: TunnelConnectionState::Disabled,
      error_code: None,
      error_message: None,
      auth_sync: TunnelAuthSyncStatus::default(),
      running: None,
      generation: 0,
      events: tokio::sync::broadcast::channel(32).0,
    }
  }
}

impl TunnelRuntime {
  fn publish(&self) {
    let _ = self.events.send(RuntimeSnapshot {
      state: self.state.clone(),
      error_code: self.error_code.clone(),
    });
  }

  fn mark_exited(&mut self, code: Option<i32>) {
    warn!(?code, "cloudflared exited");
    self.running = None;
    self.state = TunnelConnectionState::Failed;
    self.error_code = Some(TunnelErrorCode::CloudflaredExited.to_string());
    self.error_message = Some("The cloudflared connector exited unexpectedly.".to_string());
    self.publish();
  }
}

#[derive(Debug)]
struct Probe<T> {
  fingerprint: u64,
  at: Instant,
  value: T,
}

impl<T: Clone> Probe<T> {
  fn fresh(&self, fingerprint: u64, ttl: Duration) -> Option<T> {
    (self.fingerprint == fingerprint && self.at.elapsed() < ttl).then(|| self.value.clone())
  }
}

#[derive(Debug, Default)]
struct StatusCache {
  binary: Option<Probe<TunnelBinaryStatus>>,
  zone: Option<Probe<std::result::Result<String, String>>>,
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait TunnelService: std::fmt::Debug + Send + Sync {
  async fn status(&self) -> Result<TunnelStatus>;
  async fn setup(&self, request: TunnelSetupRequest) -> Result<TunnelStatus>;
  async fn enable(&self, request: EnableTunnelRequest) -> Result<TunnelStatus>;
  async fn update_preferences(
    &self,
    request: UpdateTunnelPreferencesRequest,
  ) -> Result<TunnelStatus>;
  async fn sync_authorization(&self) -> Result<TunnelStatus>;
  async fn reconnect(&self) -> Result<()>;
  async fn disable(&self) -> Result<TunnelStatus>;

  /// Must never spawn `cloudflared` or call the Cloudflare API: `/bodhi/v1/info` is anonymous and
  /// becomes internet-reachable through the tunnel. `Option`, so a tunnel fault cannot fail `/info`.
  async fn remote_access_info(&self) -> Option<RemoteAccessInfo>;
}

#[derive(Debug)]
pub struct DefaultTunnelService {
  settings: Arc<dyn SettingService>,
  auth_service: Option<Arc<dyn AuthService>>,
  tenant_service: Option<Arc<dyn TenantService>>,
  cli: Arc<dyn CloudflaredCli>,
  connector: Arc<dyn ConnectorProcess>,
  io: Arc<dyn TunnelIo>,
  runtime: Arc<Mutex<TunnelRuntime>>,
  operation: tokio::sync::Mutex<()>,
  status_cache: Mutex<StatusCache>,
  cloudflare_api: String,
  tunnel_name: tokio::sync::OnceCell<String>,
  supervisor_interval: Duration,
  ready_probe_interval: Duration,
}

impl DefaultTunnelService {
  const MINIMUM_VERSION: &'static str = "2025.2.0";

  pub fn new(settings: Arc<dyn SettingService>) -> Self {
    Self {
      settings,
      auth_service: None,
      tenant_service: None,
      cli: Arc::new(SystemCloudflaredCli),
      connector: Arc::new(SystemConnectorProcess),
      io: Arc::new(SystemTunnelIo::default()),
      runtime: Arc::new(Mutex::new(TunnelRuntime::default())),
      operation: tokio::sync::Mutex::new(()),
      status_cache: Mutex::new(StatusCache::default()),
      cloudflare_api: CLOUDFLARE_API.to_string(),
      tunnel_name: tokio::sync::OnceCell::new(),
      supervisor_interval: SUPERVISOR_INTERVAL,
      ready_probe_interval: READY_PROBE_INTERVAL,
    }
  }

  pub fn with_auth(
    settings: Arc<dyn SettingService>,
    auth_service: Arc<dyn AuthService>,
    tenant_service: Arc<dyn TenantService>,
  ) -> Self {
    let mut service = Self::new(settings);
    service.auth_service = Some(auth_service);
    service.tenant_service = Some(tenant_service);
    service
  }

  pub fn with_runtime(
    mut self,
    cli: Arc<dyn CloudflaredCli>,
    connector: Arc<dyn ConnectorProcess>,
    io: Arc<dyn TunnelIo>,
  ) -> Self {
    self.cli = cli;
    self.connector = connector;
    self.io = io;
    self
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn with_intervals(mut self, supervisor: Duration, ready_probe: Duration) -> Self {
    self.supervisor_interval = supervisor;
    self.ready_probe_interval = ready_probe;
    self
  }

  fn cache_key(value: &impl std::hash::Hash) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
  }

  #[cfg(any(test, feature = "test-utils"))]
  pub fn with_cloudflare_api(mut self, base_url: &str) -> Self {
    self.cloudflare_api = base_url.to_string();
    self
  }

  fn version_tuple(value: &str) -> Option<(u32, u32, u32)> {
    let token = value
      .split_whitespace()
      .find(|token| token.as_bytes().first().is_some_and(u8::is_ascii_digit))?;
    let mut parts = token.trim_start_matches('v').split('.');
    Some((
      parts.next()?.parse().ok()?,
      parts.next()?.parse().ok()?,
      parts.next().unwrap_or("0").parse().ok()?,
    ))
  }

  async fn configured_binary(&self) -> Option<(PathBuf, TunnelPathSource)> {
    let (value, source) = self
      .settings
      .get_setting_value_with_source(BODHI_TUNNEL_CLOUDFLARED_PATH)
      .await;
    value
      .and_then(|value| value.as_str().map(ToOwned::to_owned))
      .map(|value| {
        let source = if source == crate::SettingSource::Environment {
          TunnelPathSource::Environment
        } else {
          TunnelPathSource::Configured
        };
        (PathBuf::from(value), source)
      })
  }

  async fn binary_status(&self) -> TunnelBinaryStatus {
    let first_existing = |candidates: Vec<PathBuf>| {
      candidates
        .into_iter()
        .find(|candidate| self.io.is_file(candidate))
    };
    let candidate = if let Some(candidate) = self.configured_binary().await {
      Some(candidate)
    } else if let Some(path) = first_existing(self.io.path_candidates()) {
      Some((path, TunnelPathSource::Path))
    } else {
      first_existing(self.io.standard_locations())
        .map(|path| (path, TunnelPathSource::StandardLocation))
    };
    let Some((path, source)) = candidate else {
      return TunnelBinaryStatus {
        state: TunnelCheckState::Missing,
        path: None,
        source: None,
        version: None,
        minimum_version: Self::MINIMUM_VERSION.to_string(),
        error: None,
      };
    };
    let cache_key = Self::cache_key(&(path.display().to_string(), format!("{source:?}")));
    if let Some(status) = self
      .status_cache
      .lock()
      .ok()
      .and_then(|cache| cache.binary.as_ref()?.fresh(cache_key, BINARY_PROBE_TTL))
    {
      return status;
    }
    let invalid = |error: String| TunnelBinaryStatus {
      state: TunnelCheckState::Invalid,
      path: Some(path.display().to_string()),
      source: Some(source.clone()),
      version: None,
      minimum_version: Self::MINIMUM_VERSION.to_string(),
      error: Some(error),
    };
    let status = match self.version_output(path.clone()).await {
      Ok(output) if output.success => {
        let version = Self::version_tuple(&output.stdout_utf8());
        let supported = version.is_some_and(|version| version >= (2025, 2, 0));
        TunnelBinaryStatus {
          state: if supported {
            TunnelCheckState::Ready
          } else if version.is_some() {
            TunnelCheckState::Unsupported
          } else {
            TunnelCheckState::Invalid
          },
          path: Some(path.display().to_string()),
          source: Some(source.clone()),
          version: version.map(|(major, minor, patch)| format!("{major}.{minor}.{patch}")),
          minimum_version: Self::MINIMUM_VERSION.to_string(),
          error: if version.is_none() {
            Some("cloudflared did not return a recognizable version".to_string())
          } else {
            None
          },
        }
      }
      Ok(output) => invalid(output.stderr_utf8().trim().chars().take(500).collect()),
      Err(error) => invalid(error.to_string()),
    };
    if let Ok(mut cache) = self.status_cache.lock() {
      cache.binary = Some(Probe {
        fingerprint: cache_key,
        at: Instant::now(),
        value: status.clone(),
      });
    }
    status
  }

  async fn version_output(&self, path: PathBuf) -> Result<RawOutput> {
    Ok(
      self
        .cli
        .run(
          path,
          vec!["--version".to_string()],
          Vec::new(),
          VERSION_TIMEOUT,
        )
        .await?,
    )
  }

  async fn configured_cert(&self) -> Option<(PathBuf, TunnelPathSource)> {
    let (value, source) = self
      .settings
      .get_setting_value_with_source(BODHI_TUNNEL_ORIGIN_CERT)
      .await;
    if let Some(value) = value.and_then(|value| value.as_str().map(ToOwned::to_owned)) {
      return Some((
        PathBuf::from(value),
        if source == crate::SettingSource::Environment {
          TunnelPathSource::Environment
        } else {
          TunnelPathSource::Configured
        },
      ));
    }
    self.io.home_dir().map(|home| {
      (
        home.join(".cloudflared/cert.pem"),
        TunnelPathSource::StandardLocation,
      )
    })
  }

  async fn read_origin_cert(&self, path: &Path) -> std::result::Result<OriginCertificate, String> {
    let contents = self
      .io
      .read_to_string(path.to_path_buf())
      .await
      .map_err(|error| error.to_string())?;
    Self::parse_origin_cert(&contents)
  }

  fn parse_origin_cert(contents: &str) -> std::result::Result<OriginCertificate, String> {
    let payload = contents
      .lines()
      .filter(|line| !line.starts_with("-----"))
      .collect::<String>();
    let decoded = base64::engine::general_purpose::STANDARD
      .decode(payload)
      .map_err(|_| "Not a usable Cloudflare certificate.".to_string())?;
    serde_json::from_slice(&decoded).map_err(|_| "Not a usable Cloudflare certificate.".to_string())
  }

  async fn zone_name(&self, cert: &OriginCertificate) -> std::result::Result<String, String> {
    let response = self
      .io
      .http(
        HttpRequest::get(format!("{}/zones/{}", self.cloudflare_api, cert.zone_id))
          .bearer(&cert.api_token),
      )
      .await
      .map_err(|error| error.to_string())?;
    if !response.is_success() {
      return Err(format!(
        "Cloudflare rejected the certificate ({}).",
        response.status
      ));
    }
    serde_json::from_str::<CloudflareZoneResponse>(&response.body)
      .ok()
      .filter(|response| response.success)
      .map(|response| response.result.name)
      .filter(|name| !name.trim().is_empty())
      .ok_or_else(|| "Cloudflare did not return the selected domain.".to_string())
  }

  async fn login_status(&self, binary_ready: bool) -> TunnelLoginStatus {
    if !binary_ready {
      return TunnelLoginStatus {
        state: TunnelCheckState::Waiting,
        cert_path: None,
        source: None,
        zone: None,
        error: None,
      };
    }
    let Some((path, source)) = self.configured_cert().await else {
      return TunnelLoginStatus {
        state: TunnelCheckState::Missing,
        cert_path: None,
        source: None,
        zone: None,
        error: None,
      };
    };
    if !self.io.is_file(&path) {
      return TunnelLoginStatus {
        state: TunnelCheckState::Missing,
        cert_path: Some(path.display().to_string()),
        source: Some(source),
        zone: None,
        error: None,
      };
    }
    let cert = match self.read_origin_cert(&path).await {
      Ok(cert) => cert,
      Err(error) => {
        return TunnelLoginStatus {
          state: TunnelCheckState::Invalid,
          cert_path: Some(path.display().to_string()),
          source: Some(source),
          zone: None,
          error: Some(error),
        }
      }
    };
    match self.resolved_zone(&cert).await {
      Ok(zone) => TunnelLoginStatus {
        state: TunnelCheckState::Ready,
        cert_path: Some(path.display().to_string()),
        source: Some(source),
        zone: Some(zone),
        error: None,
      },
      Err(error) => TunnelLoginStatus {
        state: TunnelCheckState::Invalid,
        cert_path: Some(path.display().to_string()),
        source: Some(source),
        zone: None,
        error: Some(error),
      },
    }
  }

  fn zone_cache_key(cert: &OriginCertificate) -> u64 {
    Self::cache_key(&(cert.zone_id.as_str(), cert.api_token.as_str()))
  }

  async fn resolved_zone(&self, cert: &OriginCertificate) -> std::result::Result<String, String> {
    let cache_key = Self::zone_cache_key(cert);
    if let Some(outcome) = self
      .status_cache
      .lock()
      .ok()
      .and_then(|cache| cache.zone.as_ref()?.fresh(cache_key, ZONE_PROBE_TTL))
    {
      return outcome;
    }
    let outcome = self.zone_name(cert).await;
    self.store_zone(cache_key, outcome.clone());
    outcome
  }

  fn store_zone(&self, cache_key: u64, value: std::result::Result<String, String>) {
    if let Ok(mut cache) = self.status_cache.lock() {
      cache.zone = Some(Probe {
        fingerprint: cache_key,
        at: Instant::now(),
        value,
      });
    }
  }

  async fn auto_reconnect(&self) -> bool {
    self
      .settings
      .get_setting(BODHI_TUNNEL_AUTO_RECONNECT)
      .await
      .and_then(|value| value.parse().ok())
      .unwrap_or(true)
  }

  fn validate_hostname(hostname: &str) -> Result<String> {
    let hostname = hostname.trim().trim_end_matches('.').to_ascii_lowercase();
    let is_valid = hostname.len() <= 253
      && hostname.contains('.')
      && hostname.split('.').all(|label| {
        !label.is_empty()
          && label.len() <= 63
          && !label.starts_with('-')
          && !label.ends_with('-')
          && label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
      });
    if is_valid {
      Ok(hostname)
    } else {
      Err(TunnelError::InvalidHostname)
    }
  }

  fn validate_subdomain(subdomain: &str) -> Result<String> {
    let subdomain = subdomain.trim().to_ascii_lowercase();
    let valid = !subdomain.is_empty()
      && subdomain.len() <= 63
      && !subdomain.contains('.')
      && !subdomain.starts_with('-')
      && !subdomain.ends_with('-')
      && subdomain
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
    if valid {
      Ok(subdomain)
    } else {
      Err(TunnelError::InvalidHostname)
    }
  }

  fn tunnel_name_for(client_id: &str) -> String {
    let instance = client_id
      .strip_prefix(RESOURCE_CLIENT_PREFIX)
      .unwrap_or(client_id);
    format!("bodhi-app-tunnel-{instance}")
  }

  async fn tunnel_name(&self) -> Result<&str> {
    self
      .tunnel_name
      .get_or_try_init(|| async {
        let tenant_service = self
          .tenant_service
          .as_ref()
          .ok_or(TunnelError::UnidentifiedInstance)?;
        let tenant = tenant_service
          .get_standalone_app()
          .await
          .map_err(|_| TunnelError::UnidentifiedInstance)?
          .ok_or(TunnelError::UnidentifiedInstance)?;
        Ok(Self::tunnel_name_for(&tenant.client_id))
      })
      .await
      .map(String::as_str)
  }

  fn runtime(&self) -> Result<MutexGuard<'_, TunnelRuntime>> {
    self
      .runtime
      .lock()
      .map_err(|_| TunnelError::RuntimePoisoned)
  }

  fn supervise(
    runtime: Arc<Mutex<TunnelRuntime>>,
    generation: u64,
    io: Arc<dyn TunnelIo>,
    supervisor_interval: Duration,
    ready_probe_interval: Duration,
  ) {
    tokio::spawn(async move {
      let mut since_probe = Duration::ZERO;
      loop {
        tokio::time::sleep(supervisor_interval).await;
        if !Self::supervise_tick(&runtime, generation) {
          return;
        }
        // Otherwise a connector only leaves `Connecting` when an admin loads the tunnel page.
        since_probe += supervisor_interval;
        if since_probe >= ready_probe_interval {
          since_probe = Duration::ZERO;
          if let Err(err) = Self::probe_ready(&runtime, io.as_ref(), Some(generation)).await {
            warn!(?err, "tunnel readiness probe failed");
            return;
          }
        }
      }
    });
  }

  /// `generation` is `Some` for the supervisor, so a tick from a replaced connector cannot clobber
  /// the state of the one that succeeded it.
  async fn probe_ready(
    runtime: &Mutex<TunnelRuntime>,
    io: &dyn TunnelIo,
    generation: Option<u64>,
  ) -> Result<()> {
    let stale = |current: u64| generation.is_some_and(|expected| current != expected);
    let (metrics_addr, hostname) = {
      let mut guard = runtime.lock().map_err(|_| TunnelError::RuntimePoisoned)?;
      if stale(guard.generation) {
        return Ok(());
      }
      let Some(running) = guard.running.as_mut() else {
        return Ok(());
      };
      match running.handle.try_wait() {
        Ok(Some(code)) => {
          guard.mark_exited(code);
          return Ok(());
        }
        Ok(None) => (
          running
            .metrics_addr
            .lock()
            .ok()
            .and_then(|addr| addr.clone()),
          running.hostname.clone(),
        ),
        Err(err) => {
          warn!(?err, "unable to inspect cloudflared process");
          return Ok(());
        }
      }
    };
    let ready = match metrics_addr {
      Some(metrics_addr) => io
        .http(HttpRequest::get(format!("http://{metrics_addr}/ready")))
        .await
        .map(|response| response.is_success())
        .unwrap_or(false),
      None => false,
    };
    let mut guard = runtime.lock().map_err(|_| TunnelError::RuntimePoisoned)?;
    if stale(guard.generation) {
      return Ok(());
    }
    if guard.running.as_ref().map(|running| &running.hostname) == Some(&hostname) {
      guard.state = if ready {
        TunnelConnectionState::Connected
      } else {
        TunnelConnectionState::Connecting
      };
      guard.error_code = None;
      guard.error_message = None;
      guard.publish();
    }
    Ok(())
  }

  fn supervise_tick(runtime: &Mutex<TunnelRuntime>, generation: u64) -> bool {
    let Ok(mut runtime) = runtime.lock() else {
      return false;
    };
    if runtime.generation != generation {
      return false;
    }
    let Some(running) = runtime.running.as_mut() else {
      return false;
    };
    match running.handle.try_wait() {
      Ok(Some(code)) => {
        runtime.mark_exited(code);
        false
      }
      Ok(None) => true,
      Err(err) => {
        warn!(?err, "unable to inspect cloudflared process");
        false
      }
    }
  }

  fn record_runtime_failure(&self, code: TunnelErrorCode, message: String) -> Result<()> {
    let mut runtime = self.runtime()?;
    runtime.state = TunnelConnectionState::Failed;
    runtime.error_code = Some(code.to_string());
    runtime.error_message = Some(message.chars().take(500).collect());
    Ok(())
  }

  fn stop_connector(&self) -> Result<()> {
    let mut runtime = self.runtime()?;
    runtime.generation = runtime.generation.wrapping_add(1);
    if let Some(mut running) = runtime.running.take() {
      running.handle.stop()?;
    }
    Ok(())
  }

  async fn origin_cert(&self) -> Result<PathBuf> {
    let path = self
      .configured_cert()
      .await
      .map(|(path, _)| path)
      .ok_or(TunnelError::MissingOriginCertificate)?;
    if self.io.is_file(&path) {
      Ok(path)
    } else {
      Err(TunnelError::MissingOriginCertificate)
    }
  }

  async fn scratch_credentials(&self, name: &str) -> Result<PathBuf> {
    let directory = self.settings.bodhi_home().await.join("tmp").join("tunnels");
    self.io.prepare_scratch_dir(directory.clone()).await?;
    Ok(directory.join(format!("{name}.json")))
  }

  async fn command_output(
    &self,
    binary: PathBuf,
    cert: PathBuf,
    args: Vec<String>,
  ) -> Result<RawOutput> {
    Ok(
      self
        .cli
        .run(
          binary,
          args,
          vec![("TUNNEL_ORIGIN_CERT".to_string(), cert.display().to_string())],
          COMMAND_TIMEOUT,
        )
        .await?,
    )
  }

  async fn tunnel_id(&self, binary: PathBuf, cert: PathBuf, name: &str) -> Result<String> {
    let output = self
      .command_output(
        binary.clone(),
        cert.clone(),
        vec![
          "tunnel".into(),
          "list".into(),
          "--output".into(),
          "json".into(),
          "-n".into(),
          name.to_string(),
        ],
      )
      .await?;
    if output.success {
      if let Ok(tunnels) = serde_json::from_slice::<Vec<serde_json::Value>>(&output.stdout) {
        if let Some(id) = tunnels
          .iter()
          .find(|tunnel| tunnel.get("name").and_then(|name| name.as_str()) == Some(name))
          .and_then(|tunnel| tunnel.get("id"))
          .and_then(|id| id.as_str())
        {
          return Ok(id.to_string());
        }
      }
    }
    let scratch = self.scratch_credentials(name).await?;
    let output = self
      .command_output(
        binary,
        cert,
        vec![
          "tunnel".into(),
          "create".into(),
          "--output".into(),
          "json".into(),
          "--credentials-file".into(),
          scratch.display().to_string(),
          name.to_string(),
        ],
      )
      .await;
    self.io.discard_scratch(scratch).await;
    let output = output?;
    if !output.success {
      return Err(TunnelError::Provisioning(
        output.stderr_utf8().trim().to_string(),
      ));
    }
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
      .ok()
      .and_then(|tunnel| {
        tunnel
          .get("id")
          .and_then(|id| id.as_str())
          .map(ToOwned::to_owned)
      })
      .ok_or_else(|| {
        TunnelError::Provisioning("cloudflared did not return a tunnel id".to_string())
      })
  }

  async fn connector_token(&self, binary: PathBuf, cert: PathBuf, name: &str) -> Result<String> {
    let output = self
      .command_output(
        binary,
        cert,
        vec!["tunnel".into(), "token".into(), name.to_string()],
      )
      .await?;
    if !output.success {
      return Err(TunnelError::Provisioning(
        output.stderr_utf8().trim().chars().take(500).collect(),
      ));
    }
    let token = output.stdout_utf8().trim().to_string();
    if token.is_empty() {
      return Err(TunnelError::Provisioning(
        "cloudflared did not return a tunnel token".to_string(),
      ));
    }
    Ok(token)
  }

  fn connector_credentials_env(token: &str) -> (&'static str, String) {
    ("TUNNEL_TOKEN", token.to_string())
  }

  fn consume_output(
    lines: Receiver<(OutputStream, String)>,
    metrics_addr: Arc<Mutex<Option<String>>>,
  ) {
    thread::spawn(move || {
      for (stream, line) in lines {
        if let Some(addr) = Self::parse_metrics_addr(&line) {
          if let Ok(mut slot) = metrics_addr.lock() {
            slot.get_or_insert(addr);
          }
        }
        // cloudflared's own stdout/stderr only; no proxied request payload passes through here.
        info!(stream = stream.as_str(), message = %scrub_secrets(&line), "cloudflared");
      }
    });
  }

  fn classify_sync_failure(error: &crate::AuthServiceError) -> TunnelAuthSyncState {
    use crate::AuthServiceError as E;
    match error {
      E::Reqwest(_) | E::ReqwestMiddlewareError(_) => TunnelAuthSyncState::Unreachable,
      E::AuthServiceApiError { status, .. } if *status >= 500 => TunnelAuthSyncState::Unreachable,
      _ => TunnelAuthSyncState::Rejected,
    }
  }

  fn parse_metrics_addr(line: &str) -> Option<String> {
    let rest = line.split_once(METRICS_LOG_MARKER)?.1.trim();
    let addr = rest
      .split_whitespace()
      .next()?
      .split('/')
      .next()?
      .trim_end_matches(['"', ',']);
    addr.rsplit_once(':')?;
    Some(addr.to_string())
  }

  fn run_args(origin: String) -> Vec<String> {
    // cloudflared parses connector flags at the `tunnel` level, before the `run` subcommand.
    vec![
      "tunnel".to_string(),
      "--no-autoupdate".to_string(),
      "--metrics".to_string(),
      "127.0.0.1:0".to_string(),
      "--loglevel".to_string(),
      "info".to_string(),
      "--protocol".to_string(),
      "auto".to_string(),
      "run".to_string(),
      "--url".to_string(),
      origin,
    ]
  }

  async fn build_status(&self) -> Result<TunnelStatus> {
    let available = self.settings.tunnel_enabled().await;
    let binary = if available {
      self.binary_status().await
    } else {
      TunnelBinaryStatus {
        state: TunnelCheckState::Waiting,
        path: None,
        source: None,
        version: None,
        minimum_version: Self::MINIMUM_VERSION.to_string(),
        error: None,
      }
    };
    let login = self
      .login_status(available && binary.state == TunnelCheckState::Ready)
      .await;
    let hostname = self.settings.get_setting(BODHI_TUNNEL_HOST).await;
    let auto_reconnect = self.auto_reconnect().await;
    let public_url = hostname.as_ref().map(|host| format!("https://{host}"));
    let subdomain = hostname.as_ref().and_then(|hostname| {
      login.zone.as_ref().and_then(|zone| {
        hostname
          .strip_suffix(&format!(".{zone}"))
          .map(ToOwned::to_owned)
      })
    });
    let runtime = self.runtime()?;
    Ok(TunnelStatus {
      available,
      unavailable_reason: (!available)
        .then(|| "Remote access is disabled for this deployment.".to_string()),
      enabled: runtime.running.is_some(),
      state: runtime.state.clone(),
      binary,
      login,
      hostname,
      subdomain,
      auto_reconnect,
      oauth_redirect_uri: public_url
        .as_ref()
        .map(|url| format!("{url}{LOGIN_CALLBACK_PATH}")),
      public_url,
      auth_sync: runtime.auth_sync.clone(),
      error_code: runtime.error_code.clone(),
      error_message: runtime.error_message.clone(),
    })
  }

  /// Best-effort: the new address already resolves, so a failure here is untidy rather than broken.
  /// Only a record pointing at *this* tunnel is touched, so a foreign one sharing the name survives.
  async fn delete_stale_cname(&self, cert: &OriginCertificate, hostname: &str, tunnel_id: &str) {
    let expected = format!("{tunnel_id}.cfargotunnel.com");
    let lookup = self
      .io
      .http(
        HttpRequest::get(format!(
          "{}/zones/{}/dns_records",
          self.cloudflare_api, cert.zone_id
        ))
        .bearer(&cert.api_token)
        .query("name", hostname),
      )
      .await;
    let records = match lookup {
      Ok(response) => match serde_json::from_str::<CloudflareDnsResponse>(&response.body) {
        Ok(body) if response.is_success() && body.success => body.result,
        _ => {
          warn!(
            hostname,
            "could not read the previous DNS record; leaving it in place"
          );
          return;
        }
      },
      Err(err) => {
        warn!(
          ?err,
          hostname, "could not reach Cloudflare to remove the previous DNS record"
        );
        return;
      }
    };
    let Some(record) = records.iter().find(|record| record.content == expected) else {
      return;
    };
    let deleted = self
      .io
      .http(
        HttpRequest::delete(format!(
          "{}/zones/{}/dns_records/{}",
          self.cloudflare_api, cert.zone_id, record.id
        ))
        .bearer(&cert.api_token),
      )
      .await;
    match deleted {
      Ok(response) if response.is_success() => {
        info!(hostname, "removed the DNS record for the previous address")
      }
      Ok(response) => warn!(
        status = response.status,
        hostname, "Cloudflare refused to remove the previous DNS record; remove it by hand"
      ),
      Err(err) => warn!(?err, hostname, "could not remove the previous DNS record"),
    }
  }

  async fn dns_conflicts(
    &self,
    cert: &OriginCertificate,
    hostname: &str,
    tunnel_id: &str,
  ) -> Result<bool> {
    let response = self
      .io
      .http(
        HttpRequest::get(format!(
          "{}/zones/{}/dns_records",
          self.cloudflare_api, cert.zone_id
        ))
        .bearer(&cert.api_token)
        .query("name", hostname),
      )
      .await
      .map_err(|error| TunnelError::Provisioning(error.to_string()))?;
    if !response.is_success() {
      return Err(TunnelError::Provisioning(format!(
        "Cloudflare DNS lookup failed ({}).",
        response.status
      )));
    }
    let records: CloudflareDnsResponse = serde_json::from_str(&response.body).map_err(|_| {
      TunnelError::Provisioning("Cloudflare returned invalid DNS data.".to_string())
    })?;
    if !records.success {
      return Err(TunnelError::Provisioning(
        "Cloudflare rejected the DNS lookup.".to_string(),
      ));
    }
    let expected = format!("{tunnel_id}.cfargotunnel.com");
    Ok(
      records
        .result
        .iter()
        .any(|record| !record.content.eq_ignore_ascii_case(&expected)),
    )
  }

  async fn sync_redirect(&self, redirect_uri: &str) {
    {
      let mut runtime = match self.runtime() {
        Ok(runtime) => runtime,
        Err(_) => return,
      };
      runtime.auth_sync = TunnelAuthSyncStatus {
        state: TunnelAuthSyncState::Syncing,
        error: None,
      };
    }
    let rejected = TunnelAuthSyncState::Rejected;
    let result = match (&self.auth_service, &self.tenant_service) {
      (Some(auth_service), Some(tenant_service)) => match tenant_service.get_standalone_app().await
      {
        Ok(Some(tenant)) => auth_service
          .update_tunnel_redirect_uri(
            &tenant.client_id,
            &tenant.client_secret,
            TUNNEL_GATEWAY,
            redirect_uri,
          )
          .await
          .map_err(|error| (Self::classify_sync_failure(&error), error.to_string())),
        Ok(None) => Err((
          rejected,
          "No standalone authorization client is configured.".to_string(),
        )),
        Err(error) => Err((rejected, error.to_string())),
      },
      _ => Err((
        rejected,
        "Authorization synchronization is unavailable.".to_string(),
      )),
    };
    if let Ok(mut runtime) = self.runtime() {
      runtime.auth_sync = match result {
        Ok(()) => TunnelAuthSyncStatus {
          state: TunnelAuthSyncState::Synced,
          error: None,
        },
        Err((state, error)) => TunnelAuthSyncStatus {
          state,
          error: Some(scrub_secrets(&error).chars().take(500).collect()),
        },
      };
    }
  }

  async fn resolved_binary(&self) -> Result<PathBuf> {
    let binary = self.binary_status().await;
    if binary.state == TunnelCheckState::Ready {
      return binary
        .path
        .map(PathBuf::from)
        .ok_or(TunnelError::MissingBinary);
    }
    Err(TunnelError::MissingBinary)
  }

  async fn validate_binary_override(&self, path: &str) -> Result<()> {
    let path = PathBuf::from(path);
    if !self.io.is_file(&path) {
      return Err(TunnelError::MissingBinary);
    }
    let output = self.version_output(path).await?;
    let supported = output.success
      && Self::version_tuple(&output.stdout_utf8()).is_some_and(|version| version >= (2025, 2, 0));
    if supported {
      Ok(())
    } else {
      Err(TunnelError::MissingBinary)
    }
  }

  async fn refresh_status(&self) -> Result<()> {
    Self::probe_ready(&self.runtime, self.io.as_ref(), None).await
  }
}

impl Drop for DefaultTunnelService {
  fn drop(&mut self) {
    match self.runtime.lock() {
      Ok(mut runtime) => {
        runtime.generation = runtime.generation.wrapping_add(1);
        if let Some(mut running) = runtime.running.take() {
          if let Err(err) = running.handle.stop() {
            warn!(
              ?err,
              "failed to stop Cloudflare Tunnel connector during drop"
            );
          }
        }
      }
      Err(_) => warn!("Cloudflare Tunnel runtime mutex poisoned during drop"),
    }
  }
}

#[async_trait::async_trait]
impl TunnelService for DefaultTunnelService {
  async fn status(&self) -> Result<TunnelStatus> {
    self.refresh_status().await?;
    self.build_status().await
  }

  async fn remote_access_info(&self) -> Option<RemoteAccessInfo> {
    if !self.settings.tunnel_enabled().await {
      return None;
    }
    // No hostname means remote access was never set up, so there is no address to advertise.
    let url = format!(
      "https://{}",
      self.settings.get_setting(BODHI_TUNNEL_HOST).await?
    );
    let runtime = self.runtime().ok()?;
    Some(RemoteAccessInfo {
      provider: RemoteAccessProvider::Cloudflared,
      url,
      enabled: runtime.running.is_some(),
      status: RemoteAccessState::from_connection(&runtime.state, runtime.error_code.is_some()),
      auth_status: RemoteAccessState::from_auth_sync(&runtime.auth_sync.state),
    })
  }

  async fn setup(&self, request: TunnelSetupRequest) -> Result<TunnelStatus> {
    if let Some(path) = request.cloudflared_path {
      let path = path.trim();
      if path.is_empty() {
        self
          .settings
          .delete_setting(BODHI_TUNNEL_CLOUDFLARED_PATH)
          .await
          .map_err(|error| TunnelError::Command(error.to_string()))?;
      } else {
        self.validate_binary_override(path).await?;
        self
          .settings
          .set_setting(BODHI_TUNNEL_CLOUDFLARED_PATH, path)
          .await
          .map_err(|error| TunnelError::Command(error.to_string()))?;
      }
    }
    if let Some(path) = request.origin_cert_path {
      let path = path.trim();
      if path.is_empty() {
        self
          .settings
          .delete_setting(BODHI_TUNNEL_ORIGIN_CERT)
          .await
          .map_err(|error| TunnelError::Command(error.to_string()))?;
      } else {
        let certificate = self
          .read_origin_cert(Path::new(path))
          .await
          .map_err(TunnelError::Provisioning)?;
        let zone = self.zone_name(&certificate).await;
        self.store_zone(Self::zone_cache_key(&certificate), zone.clone());
        zone.map_err(TunnelError::Provisioning)?;
        self
          .settings
          .set_setting(BODHI_TUNNEL_ORIGIN_CERT, path)
          .await
          .map_err(|error| TunnelError::Command(error.to_string()))?;
      }
    }
    self.status().await
  }

  async fn enable(&self, request: EnableTunnelRequest) -> Result<TunnelStatus> {
    let _operation = self.operation.lock().await;
    if !self.settings.tunnel_enabled().await {
      return Err(TunnelError::Disabled);
    }
    let binary = self.resolved_binary().await?;
    let login = self.login_status(true).await;
    let zone = login
      .zone
      .filter(|_| login.state == TunnelCheckState::Ready)
      .ok_or(TunnelError::MissingOriginCertificate)?;
    let subdomain = Self::validate_subdomain(&request.subdomain)?;
    let hostname = Self::validate_hostname(&format!("{subdomain}.{zone}"))?;
    // Read before the new address is saved, so a subdomain change can clean up the old record.
    let previous_hostname = self.settings.get_setting(BODHI_TUNNEL_HOST).await;
    self.stop_connector()?;
    let cert = self.origin_cert().await?;
    let certificate = self
      .read_origin_cert(&cert)
      .await
      .map_err(TunnelError::Provisioning)?;
    let name = self.tunnel_name().await?.to_string();
    let tunnel_id = self.tunnel_id(binary.clone(), cert.clone(), &name).await?;
    if self
      .dns_conflicts(&certificate, &hostname, &tunnel_id)
      .await?
      && !request.replace_dns
    {
      let mut runtime = self.runtime()?;
      runtime.error_code = Some(TunnelErrorCode::DnsConflict.to_string());
      runtime.error_message = Some("A DNS record already exists for this address.".to_string());
      return Err(TunnelError::DnsConflict);
    }
    let mut route_args = vec!["tunnel".into(), "route".into(), "dns".into()];
    if request.replace_dns {
      route_args.push("--overwrite-dns".into());
    }
    route_args.push(tunnel_id.clone());
    route_args.push(hostname.clone());
    let route = self
      .command_output(binary.clone(), cert.clone(), route_args)
      .await?;
    if !route.success {
      return Err(TunnelError::Provisioning(
        scrub_secrets(route.stderr_utf8().trim())
          .chars()
          .take(500)
          .collect(),
      ));
    }
    if let Some(previous) = previous_hostname {
      if previous != hostname {
        self
          .delete_stale_cname(&certificate, &previous, &tunnel_id)
          .await;
      }
    }
    let token = self.connector_token(binary.clone(), cert, &name).await?;
    let origin = format!("http://127.0.0.1:{}", self.settings.port().await);
    let run_args = Self::run_args(origin);
    let (credential_key, credential_value) = Self::connector_credentials_env(&token);
    let mut handle = self.connector.spawn(
      binary,
      run_args,
      vec![(credential_key.to_string(), credential_value)],
    )?;
    let metrics_addr = Arc::new(Mutex::new(None));
    if let Some(lines) = handle.take_output() {
      Self::consume_output(lines, metrics_addr.clone());
    }
    let generation = {
      let mut runtime = self.runtime()?;
      let generation = runtime.generation.wrapping_add(1);
      runtime.generation = generation;
      runtime.running = Some(RunningTunnel {
        handle,
        metrics_addr,
        hostname: hostname.clone(),
      });
      runtime.state = TunnelConnectionState::Connecting;
      runtime.error_code = None;
      runtime.error_message = None;
      generation
    };
    Self::supervise(
      self.runtime.clone(),
      generation,
      self.io.clone(),
      self.supervisor_interval,
      self.ready_probe_interval,
    );
    self
      .settings
      .set_setting(BODHI_TUNNEL_HOST, &hostname)
      .await
      .map_err(|err| TunnelError::Command(err.to_string()))?;
    self
      .settings
      .set_setting(
        BODHI_TUNNEL_AUTO_RECONNECT,
        &request.auto_reconnect.to_string(),
      )
      .await
      .map_err(|err| TunnelError::Command(err.to_string()))?;
    let redirect_uri = format!("https://{hostname}{LOGIN_CALLBACK_PATH}");
    self.sync_redirect(&redirect_uri).await;
    self.status().await
  }

  async fn update_preferences(
    &self,
    request: UpdateTunnelPreferencesRequest,
  ) -> Result<TunnelStatus> {
    self
      .settings
      .set_setting(
        BODHI_TUNNEL_AUTO_RECONNECT,
        &request.auto_reconnect.to_string(),
      )
      .await
      .map_err(|error| TunnelError::Command(error.to_string()))?;
    self.status().await
  }

  async fn sync_authorization(&self) -> Result<TunnelStatus> {
    let hostname = self
      .settings
      .get_setting(BODHI_TUNNEL_HOST)
      .await
      .ok_or(TunnelError::InvalidHostname)?;
    self
      .sync_redirect(&format!("https://{hostname}{LOGIN_CALLBACK_PATH}"))
      .await;
    self.status().await
  }

  async fn reconnect(&self) -> Result<()> {
    if !self.settings.tunnel_enabled().await || !self.auto_reconnect().await {
      return Ok(());
    }
    let Some(hostname) = self.settings.get_setting(BODHI_TUNNEL_HOST).await else {
      return Ok(());
    };
    let login = self.login_status(true).await;
    let Some(zone) = login.zone else {
      self.record_runtime_failure(
        TunnelErrorCode::ReconnectZoneUnavailable,
        login
          .error
          .unwrap_or_else(|| "Cloudflare sign-in needs attention before reconnecting.".to_string()),
      )?;
      return Ok(());
    };
    let Some(subdomain) = hostname.strip_suffix(&format!(".{zone}")) else {
      self.record_runtime_failure(
        TunnelErrorCode::ReconnectHostnameStale,
        format!("The saved address {hostname} no longer belongs to the Cloudflare zone {zone}."),
      )?;
      return Ok(());
    };
    let result = self
      .enable(EnableTunnelRequest {
        subdomain: subdomain.to_string(),
        auto_reconnect: true,
        replace_dns: false,
      })
      .await
      .map(|_| ());
    if let Err(error) = &result {
      let code = match error {
        TunnelError::DnsConflict => TunnelErrorCode::DnsConflict,
        _ => TunnelErrorCode::ProvisioningFailed,
      };
      self.record_runtime_failure(code, error.to_string())?;
    }
    result
  }

  async fn disable(&self) -> Result<TunnelStatus> {
    let _operation = self.operation.lock().await;
    self.stop_connector()?;
    {
      let mut runtime = self.runtime()?;
      runtime.state = TunnelConnectionState::Disabled;
      runtime.error_code = None;
      runtime.error_message = None;
    }
    self.status().await
  }
}

#[cfg(test)]
#[path = "test_tunnel_service.rs"]
mod tests;
