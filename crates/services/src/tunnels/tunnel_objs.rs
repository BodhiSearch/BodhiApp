use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct EnableTunnelRequest {
  pub subdomain: String,
  #[serde(default = "default_true")]
  pub auto_reconnect: bool,
  #[serde(default)]
  pub replace_dns: bool,
}

fn default_true() -> bool {
  true
}

#[derive(Debug, Clone, Deserialize, ToSchema, Default)]
pub struct TunnelSetupRequest {
  pub cloudflared_path: Option<String>,
  pub origin_cert_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateTunnelPreferencesRequest {
  pub auto_reconnect: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TunnelStatus {
  pub available: bool,
  pub unavailable_reason: Option<String>,
  pub enabled: bool,
  pub state: TunnelConnectionState,
  pub binary: TunnelBinaryStatus,
  pub login: TunnelLoginStatus,
  pub hostname: Option<String>,
  pub subdomain: Option<String>,
  pub auto_reconnect: bool,
  pub public_url: Option<String>,
  pub oauth_redirect_uri: Option<String>,
  pub auth_sync: TunnelAuthSyncStatus,
  pub error_code: Option<String>,
  pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TunnelBinaryStatus {
  pub state: TunnelCheckState,
  pub path: Option<String>,
  pub source: Option<TunnelPathSource>,
  pub version: Option<String>,
  pub minimum_version: String,
  pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TunnelLoginStatus {
  pub state: TunnelCheckState,
  pub cert_path: Option<String>,
  pub source: Option<TunnelPathSource>,
  pub zone: Option<String>,
  pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TunnelAuthSyncStatus {
  pub state: TunnelAuthSyncState,
  pub error: Option<String>,
}

impl Default for TunnelAuthSyncStatus {
  fn default() -> Self {
    Self {
      state: TunnelAuthSyncState::NotAttempted,
      error: None,
    }
  }
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelPathSource {
  Configured,
  Environment,
  Path,
  StandardLocation,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelCheckState {
  Waiting,
  Missing,
  Invalid,
  Unsupported,
  Ready,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelAuthSyncState {
  NotAttempted,
  Syncing,
  Synced,
  /// Transport failure or 5xx; retrying is worthwhile on its own.
  Unreachable,
  /// The server answered and refused; permissions have to change before a retry can help.
  Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum TunnelErrorCode {
  DnsConflict,
  CloudflaredExited,
  ProvisioningFailed,
  ReconnectZoneUnavailable,
  ReconnectHostnameStale,
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelConnectionState {
  Disabled,
  Connecting,
  Connected,
  Failed,
}

/// The anonymous counterpart of [`TunnelStatus`]. No field here can carry a path, the zone or raw
/// stderr, so the admin-only split holds by construction rather than by remembering to strip fields.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct RemoteAccessInfo {
  pub provider: RemoteAccessProvider,
  /// The public base URL this route answers on.
  pub url: String,
  /// True while still connecting; this is not "the user switched it on".
  pub enabled: bool,
  /// Absent means neither ready nor failed — off, or still coming up.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<RemoteAccessState>,
  /// Absent means not attempted yet, or in flight.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_status: Option<RemoteAccessState>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemoteAccessProvider {
  Cloudflared,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemoteAccessState {
  Ready,
  Error,
}

impl RemoteAccessState {
  /// `error_code` wins: a DNS conflict from `enable` records the code without moving state off
  /// `Disabled`, which would otherwise be indistinguishable from remote access being switched off.
  pub fn from_connection(state: &TunnelConnectionState, has_error_code: bool) -> Option<Self> {
    if has_error_code {
      return Some(Self::Error);
    }
    match state {
      TunnelConnectionState::Connected => Some(Self::Ready),
      TunnelConnectionState::Failed => Some(Self::Error),
      TunnelConnectionState::Connecting | TunnelConnectionState::Disabled => None,
    }
  }

  pub fn from_auth_sync(state: &TunnelAuthSyncState) -> Option<Self> {
    match state {
      TunnelAuthSyncState::Synced => Some(Self::Ready),
      TunnelAuthSyncState::Unreachable | TunnelAuthSyncState::Rejected => Some(Self::Error),
      TunnelAuthSyncState::NotAttempted | TunnelAuthSyncState::Syncing => None,
    }
  }
}
