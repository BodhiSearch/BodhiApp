use super::{DefaultTunnelService, RuntimeSnapshot, TunnelError};
use crate::{
  shared_objs::log::scrub_secrets,
  test_utils::{
    FakeCloudflared, SettingServiceStub, FAKE_BINARY, FAKE_METRICS_ADDR, FAKE_ORIGIN_CERT,
    FAKE_TUNNEL_ID, FAKE_TUNNEL_TOKEN, FAKE_ZONE,
  },
  AppStatus, EnableTunnelRequest, MockAuthService, MockTenantService, RemoteAccessProvider,
  RemoteAccessState, SettingService, Tenant, TunnelAuthSyncState, TunnelConnectionState,
  TunnelErrorCode, TunnelService, BODHI_TUNNEL, BODHI_TUNNEL_CLOUDFLARED_PATH,
  BODHI_TUNNEL_ORIGIN_CERT, RESOURCE_CLIENT_PREFIX,
};
use anyhow_trace::anyhow_trace;
use base64::Engine;
use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
  time::Duration,
};

const INSTANCE_ID: &str = "0cb93fdd-f8f4-471e-aaa8-54feb41c073d";
const TUNNEL_NAME: &str = "bodhi-app-tunnel-0cb93fdd-f8f4-471e-aaa8-54feb41c073d";
const SUBDOMAIN: &str = "amir";

fn client_id() -> String {
  format!("{RESOURCE_CLIENT_PREFIX}{INSTANCE_ID}")
}
const TEST_INTERVAL: Duration = Duration::from_millis(5);

#[test]
fn validates_cloudflare_hostnames_without_accepting_urls_or_ports() {
  assert_eq!(
    "tunnel.example.com",
    DefaultTunnelService::validate_hostname("Tunnel.Example.Com.").unwrap()
  );
  assert!(DefaultTunnelService::validate_hostname("https://tunnel.example.com").is_err());
  assert!(DefaultTunnelService::validate_hostname("tunnel.example.com:443").is_err());
}

#[test]
fn tunnel_name_identifies_the_instance_by_its_oauth_client() {
  assert_eq!(
    TUNNEL_NAME,
    DefaultTunnelService::tunnel_name_for(&client_id())
  );
  assert_eq!(
    "bodhi-app-tunnel-other-client",
    DefaultTunnelService::tunnel_name_for("other-client")
  );
}

#[test]
fn connector_flags_precede_the_run_subcommand_and_carry_no_credentials() {
  assert_eq!(
    DefaultTunnelService::run_args("http://127.0.0.1:11135".to_string()),
    vec![
      "tunnel",
      "--no-autoupdate",
      "--metrics",
      "127.0.0.1:0",
      "--loglevel",
      "info",
      "--protocol",
      "auto",
      "run",
      "--url",
      "http://127.0.0.1:11135",
    ]
  );
}

#[test]
fn reads_the_metrics_address_cloudflared_chose() {
  assert_eq!(
    Some("127.0.0.1:42042".to_string()),
    DefaultTunnelService::parse_metrics_addr(
      "2026-09-18T00:00:00Z INF Starting metrics server on 127.0.0.1:42042/metrics"
    )
  );
  assert_eq!(
    None,
    DefaultTunnelService::parse_metrics_addr("INF Registered tunnel connection")
  );
}

#[test]
fn runtime_error_codes_are_stable_on_the_wire() {
  assert_eq!("dns_conflict", TunnelErrorCode::DnsConflict.to_string());
  assert_eq!(
    "cloudflared_exited",
    TunnelErrorCode::CloudflaredExited.to_string()
  );
  assert_eq!(
    "provisioning_failed",
    TunnelErrorCode::ProvisioningFailed.to_string()
  );
  assert_eq!(
    "reconnect_zone_unavailable",
    TunnelErrorCode::ReconnectZoneUnavailable.to_string()
  );
  assert_eq!(
    "reconnect_hostname_stale",
    TunnelErrorCode::ReconnectHostnameStale.to_string()
  );
}

#[test]
fn connector_credentials_travel_in_the_environment() {
  assert_eq!(
    ("TUNNEL_TOKEN", "a-token".to_string()),
    DefaultTunnelService::connector_credentials_env("a-token")
  );
}

#[test]
fn reads_cloudflare_origin_certificate_with_cloudflare_field_casing() {
  let payload = r#"{"zoneID":"zone-id","accountID":"account-id","apiToken":"secret"}"#;
  let encoded = base64::engine::general_purpose::STANDARD.encode(payload);
  let pem =
    format!("-----BEGIN ARGO TUNNEL TOKEN-----\n{encoded}\n-----END ARGO TUNNEL TOKEN-----\n");

  let certificate = DefaultTunnelService::parse_origin_cert(&pem).unwrap();
  assert_eq!("zone-id", certificate.zone_id);
  assert_eq!("secret", certificate.api_token);
}

struct TunnelHarness {
  service: DefaultTunnelService,
  fake: FakeCloudflared,
  bodhi_home: PathBuf,
  redirect_calls: Arc<Mutex<Vec<(String, String, String, String)>>>,
}

fn tenant() -> Tenant {
  Tenant {
    id: "tenant-id".to_string(),
    client_id: client_id(),
    client_secret: "client-secret".to_string(),
    name: "Bodhi".to_string(),
    description: None,
    status: AppStatus::Ready,
    created_by: None,
    created_at: DateTime::<Utc>::default(),
    updated_at: DateTime::<Utc>::default(),
  }
}

fn enable_request() -> EnableTunnelRequest {
  EnableTunnelRequest {
    subdomain: SUBDOMAIN.to_string(),
    auto_reconnect: true,
    replace_dns: false,
  }
}

async fn harness(fake: FakeCloudflared) -> TunnelHarness {
  let temp_home = Arc::new(tempfile::tempdir().unwrap());
  let settings = SettingServiceStub::with_defaults_in(temp_home.clone());
  settings.set_setting(BODHI_TUNNEL, "true").await.unwrap();
  settings
    .set_setting(BODHI_TUNNEL_CLOUDFLARED_PATH, FAKE_BINARY)
    .await
    .unwrap();
  settings
    .set_setting(BODHI_TUNNEL_ORIGIN_CERT, FAKE_ORIGIN_CERT)
    .await
    .unwrap();
  let bodhi_home = settings.bodhi_home().await;

  let mut tenant_service = MockTenantService::new();
  tenant_service
    .expect_get_standalone_app()
    .returning(|| Ok(Some(tenant())));
  let mut auth_service = MockAuthService::new();
  let redirect_calls: Arc<Mutex<Vec<(String, String, String, String)>>> =
    Arc::new(Mutex::new(Vec::new()));
  let recorder = redirect_calls.clone();
  auth_service.expect_update_tunnel_redirect_uri().returning(
    move |client_id, client_secret, gateway, redirect_uri| {
      recorder.lock().unwrap().push((
        client_id.to_string(),
        client_secret.to_string(),
        gateway.to_string(),
        redirect_uri.to_string(),
      ));
      Ok(())
    },
  );

  let service = DefaultTunnelService::with_auth(
    Arc::new(settings),
    Arc::new(auth_service),
    Arc::new(tenant_service),
  )
  .with_runtime(
    Arc::new(fake.clone()),
    Arc::new(fake.clone()),
    Arc::new(fake.clone()),
  )
  .with_intervals(TEST_INTERVAL, TEST_INTERVAL);

  TunnelHarness {
    service,
    fake,
    bodhi_home,
    redirect_calls,
  }
}

fn snapshot(service: &DefaultTunnelService) -> RuntimeSnapshot {
  let runtime = service.runtime.lock().unwrap();
  RuntimeSnapshot {
    state: runtime.state.clone(),
    error_code: runtime.error_code.clone(),
  }
}

async fn wait_for(
  service: &DefaultTunnelService,
  predicate: impl Fn(&RuntimeSnapshot) -> bool,
) -> RuntimeSnapshot {
  let mut events = service.runtime.lock().unwrap().events.subscribe();
  let current = snapshot(service);
  if predicate(&current) {
    return current;
  }
  tokio::time::timeout(Duration::from_secs(5), async {
    loop {
      match events.recv().await {
        Ok(snapshot) if predicate(&snapshot) => return snapshot,
        Ok(_) => continue,
        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
        Err(err) => panic!("runtime event channel closed: {err}"),
      }
    }
  })
  .await
  .expect("the runtime never published the expected state")
}

/// `/bodhi/v1/info` is anonymous and, once a tunnel is live, reachable from the open internet. The
/// summary it serves must therefore never drive a `cloudflared` subprocess or a Cloudflare API call
/// the way the admin-only `status()` legitimately does, or unauthenticated traffic could pump both.
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn the_public_summary_never_spawns_cloudflared_or_calls_cloudflare() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;

  // Before setup there is nothing configured, so nothing may be probed to discover that.
  let before_setup = harness.fake.calls();
  assert_eq!(None, harness.service.remote_access_info().await);
  assert_eq!(before_setup, harness.fake.calls());

  harness.service.enable(enable_request()).await?;
  let after_enable = harness.fake.calls();
  let zone_lookups = harness.fake.zone_lookups();
  for _ in 0..25 {
    harness.service.remote_access_info().await;
  }
  assert_eq!(
    after_enable,
    harness.fake.calls(),
    "25 anonymous reads must not invoke cloudflared once"
  );
  assert_eq!(
    zone_lookups,
    harness.fake.zone_lookups(),
    "nor reach the Cloudflare API"
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn advertises_no_remote_access_until_it_is_available_and_configured() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;
  // Available, but never set up — there is no address to hand out.
  assert_eq!(None, harness.service.remote_access_info().await);

  harness.service.enable(enable_request()).await?;
  assert!(harness.service.remote_access_info().await.is_some());

  // Feature switched off for this deployment: nothing is advertised even though a host is saved.
  harness
    .service
    .settings
    .set_setting(BODHI_TUNNEL, "false")
    .await?;
  assert_eq!(None, harness.service.remote_access_info().await);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn publishes_the_tunnel_address_without_any_of_its_diagnostics() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;
  harness.service.enable(enable_request()).await?;

  let info = harness.service.remote_access_info().await.unwrap();
  assert_eq!(RemoteAccessProvider::Cloudflared, info.provider);
  assert_eq!(format!("https://{SUBDOMAIN}.{FAKE_ZONE}"), info.url);
  assert!(info.enabled, "a connector is running");
  // Still coming up: neither ready nor failed, so the field is absent rather than guessed.
  assert_eq!(None, info.status);
  Ok(())
}

/// A DNS conflict raised by `enable` records an error code *without* moving the connection state off
/// `Disabled`, while the same conflict reached through `reconnect` goes via `record_runtime_failure`
/// and lands on `Failed`. Collapsing on state alone would therefore report the common UI-driven case
/// as though the user had simply switched remote access off.
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn a_blocked_tunnel_never_reads_as_merely_switched_off() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;
  harness.service.enable(enable_request()).await?;

  {
    let mut runtime = harness.service.runtime.lock().unwrap();
    runtime.running = None;
    runtime.state = TunnelConnectionState::Disabled;
    runtime.error_code = Some(TunnelErrorCode::DnsConflict.to_string());
  }
  let info = harness.service.remote_access_info().await.unwrap();
  assert_eq!(Some(RemoteAccessState::Error), info.status);
  assert!(!info.enabled);

  // The `reconnect` shape, where the state did move.
  {
    let mut runtime = harness.service.runtime.lock().unwrap();
    runtime.state = TunnelConnectionState::Failed;
  }
  assert_eq!(
    Some(RemoteAccessState::Error),
    harness.service.remote_access_info().await.unwrap().status
  );
  Ok(())
}

/// Nothing but an admin loading `GET /bodhi/v1/tunnel` used to promote a connector past
/// `Connecting`, so an unattended instance reported a healthy tunnel as never-ready for as long as
/// it ran. The supervisor now runs the readiness probe itself.
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn the_supervisor_notices_readiness_without_anyone_polling_status() -> anyhow::Result<()> {
  let harness =
    harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME).with_metrics_line()).await;
  harness.service.enable(enable_request()).await?;

  let observed = wait_for(&harness.service, |snapshot| {
    snapshot.state == TunnelConnectionState::Connected
  })
  .await;

  assert_eq!(TunnelConnectionState::Connected, observed.state);
  assert_eq!(
    Some(RemoteAccessState::Ready),
    harness.service.remote_access_info().await.unwrap().status
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn a_connector_that_announces_no_metrics_address_never_reads_as_ready() -> anyhow::Result<()>
{
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;
  harness.service.enable(enable_request()).await?;

  let status = harness.service.status().await?;
  assert_eq!(TunnelConnectionState::Connecting, status.state);
  Ok(())
}

#[rstest]
#[case::connected_is_ready(
  TunnelConnectionState::Connected,
  false,
  Some(RemoteAccessState::Ready)
)]
#[case::failed_is_error(TunnelConnectionState::Failed, false, Some(RemoteAccessState::Error))]
#[case::connecting_is_undecided(TunnelConnectionState::Connecting, false, None)]
#[case::disabled_is_undecided(TunnelConnectionState::Disabled, false, None)]
#[case::code_beats_disabled(TunnelConnectionState::Disabled, true, Some(RemoteAccessState::Error))]
#[case::code_beats_connecting(
  TunnelConnectionState::Connecting,
  true,
  Some(RemoteAccessState::Error)
)]
#[case::code_beats_connected(
  TunnelConnectionState::Connected,
  true,
  Some(RemoteAccessState::Error)
)]
#[case::code_beats_failed(TunnelConnectionState::Failed, true, Some(RemoteAccessState::Error))]
fn every_connection_state_collapses_to_the_public_enum(
  #[case] state: TunnelConnectionState,
  #[case] has_error_code: bool,
  #[case] expected: Option<RemoteAccessState>,
) {
  assert_eq!(
    expected,
    RemoteAccessState::from_connection(&state, has_error_code)
  );
}

#[rstest]
#[case::synced_is_ready(TunnelAuthSyncState::Synced, Some(RemoteAccessState::Ready))]
#[case::unreachable_is_error(TunnelAuthSyncState::Unreachable, Some(RemoteAccessState::Error))]
#[case::rejected_is_error(TunnelAuthSyncState::Rejected, Some(RemoteAccessState::Error))]
#[case::not_attempted_is_undecided(TunnelAuthSyncState::NotAttempted, None)]
#[case::syncing_is_undecided(TunnelAuthSyncState::Syncing, None)]
fn every_auth_sync_state_collapses_to_the_public_enum(
  #[case] state: TunnelAuthSyncState,
  #[case] expected: Option<RemoteAccessState>,
) {
  assert_eq!(expected, RemoteAccessState::from_auth_sync(&state));
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn creates_the_tunnel_once_and_keeps_no_credentials_under_bodhi_home() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::new()).await;

  let status = harness.service.enable(enable_request()).await?;

  assert_eq!(
    Some(format!("{SUBDOMAIN}.{FAKE_ZONE}")),
    status.hostname,
    "the hostname is composed from the resolved zone"
  );
  let created = harness
    .fake
    .calls()
    .into_iter()
    .filter(|call| call.starts_with("tunnel create "))
    .collect::<Vec<_>>();
  assert_eq!(1, created.len());
  assert!(created[0].ends_with(TUNNEL_NAME));

  let scratch = harness
    .bodhi_home
    .join("tmp")
    .join("tunnels")
    .join(format!("{TUNNEL_NAME}.json"));
  assert_eq!(vec![scratch.clone()], harness.fake.credentials_files());
  assert_eq!(
    vec![scratch],
    harness.fake.discarded_files(),
    "the scratch credentials file is discarded as soon as the tunnel exists"
  );
  assert!(
    !harness.bodhi_home.join("tunnels").exists(),
    "nothing durable is written under BODHI_HOME"
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn reuses_the_existing_tunnel_when_no_local_state_exists() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;

  harness.service.enable(enable_request()).await?;

  let calls = harness.fake.calls();
  assert!(
    !calls.iter().any(|call| call.starts_with("tunnel create ")),
    "an existing tunnel is adopted, never recreated: {calls:?}"
  );
  assert!(
    calls.contains(&format!("tunnel token {TUNNEL_NAME}")),
    "credentials are re-fetched from Cloudflare on every start: {calls:?}"
  );
  assert!(
    harness.fake.credentials_files().is_empty(),
    "no credentials file is written when the tunnel already exists"
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn hands_the_connector_token_through_the_environment() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;

  harness.service.enable(enable_request()).await?;

  assert_eq!(
    Some(FAKE_TUNNEL_TOKEN.to_string()),
    harness.fake.connector_token()
  );
  let args = harness.fake.connector_args().unwrap();
  assert!(
    !args.contains(FAKE_TUNNEL_TOKEN),
    "the token must never reach argv: {args}"
  );
  assert!(!args.contains("--credentials-file"), "{args}");
  assert!(args.contains("--url http://127.0.0.1:1135"), "{args}");
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn leaves_no_connector_behind_when_disable_races_enable() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;

  let (enabled, disabled) = tokio::join!(
    harness.service.enable(enable_request()),
    harness.service.disable()
  );
  enabled?;
  let status = disabled?;

  assert_eq!(TunnelConnectionState::Disabled, status.state);
  assert!(!status.enabled);
  assert!(
    harness.service.runtime.lock().unwrap().running.is_none(),
    "no connector is tracked once the racing disable returns"
  );
  assert!(
    harness.fake.connector_stopped(),
    "the connector enable() spawned was stopped, not orphaned"
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn shuts_down_while_a_provisioning_call_is_still_in_flight() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME).with_dns_gate()).await;

  let enabling = harness.service.enable(enable_request());
  let shutting_down = tokio::time::timeout(Duration::from_secs(5), harness.service.disable());
  let releasing = async {
    for _ in 0..16 {
      tokio::task::yield_now().await;
    }
    harness.fake.release_dns();
  };
  let (enabled, shutdown, ()) = tokio::join!(enabling, shutting_down, releasing);

  enabled?;
  let status =
    shutdown.expect("shutdown must not block indefinitely on the in-flight provisioning call")?;
  assert_eq!(TunnelConnectionState::Disabled, status.state);
  assert!(harness.fake.connector_stopped());
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn notices_a_connector_that_exits_without_waiting_for_a_poll() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;

  let status = harness.service.enable(enable_request()).await?;
  assert_eq!(TunnelConnectionState::Connecting, status.state);

  harness.fake.exit_connector(Some(1));
  let observed = wait_for(&harness.service, |snapshot| snapshot.error_code.is_some()).await;

  assert_eq!(
    Some(TunnelErrorCode::CloudflaredExited.to_string()),
    observed.error_code,
    "the supervisor records the exit without anyone calling status()"
  );
  assert_eq!(TunnelConnectionState::Failed, observed.state);
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn probes_cloudflared_and_cloudflare_once_across_repeated_polls() -> anyhow::Result<()> {
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;

  harness.service.enable(enable_request()).await?;
  harness.service.status().await?;
  harness.service.status().await?;

  let versions = harness
    .fake
    .calls()
    .into_iter()
    .filter(|call| call == "--version")
    .count();
  assert_eq!(
    1,
    versions,
    "the binary probe is cached across polls: {:?}",
    harness.fake.calls()
  );
  assert_eq!(
    1,
    harness.fake.zone_lookups(),
    "the zone lookup is cached across polls"
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn refuses_to_provision_without_an_authorization_client() -> anyhow::Result<()> {
  let fake = FakeCloudflared::new();
  let harness = harness(fake.clone()).await;
  let service = DefaultTunnelService::new(harness.service.settings.clone()).with_runtime(
    Arc::new(fake.clone()),
    Arc::new(fake.clone()),
    Arc::new(fake),
  );

  let error = service.enable(enable_request()).await.unwrap_err();

  assert!(matches!(error, TunnelError::UnidentifiedInstance));
  Ok(())
}

#[test]
fn scrubs_opaque_secrets_while_keeping_the_text_diagnosable() {
  let scrubbed = scrub_secrets(&format!(
    "failed to run {TUNNEL_NAME} with token {FAKE_TUNNEL_TOKEN} at /tmp/tunnels/x.json"
  ));

  assert!(
    !scrubbed.contains(FAKE_TUNNEL_TOKEN),
    "the token must not survive: {scrubbed}"
  );
  assert!(scrubbed.contains("<redacted>"), "{scrubbed}");
  assert!(
    scrubbed.contains(TUNNEL_NAME),
    "the tunnel name is not a secret and is what makes the error readable: {scrubbed}"
  );
  assert!(scrubbed.contains("/tmp/tunnels/x.json"), "{scrubbed}");
}

#[rstest]
#[case::transport(
  crate::AuthServiceError::ReqwestMiddlewareError("connection refused".to_string()),
  TunnelAuthSyncState::Unreachable
)]
#[case::server_error(
  crate::AuthServiceError::AuthServiceApiError { status: 503, body: "down".to_string() },
  TunnelAuthSyncState::Unreachable
)]
#[case::forbidden(
  crate::AuthServiceError::AuthServiceApiError { status: 403, body: "nope".to_string() },
  TunnelAuthSyncState::Rejected
)]
#[case::unauthorized(
  crate::AuthServiceError::AuthServiceApiError { status: 401, body: "nope".to_string() },
  TunnelAuthSyncState::Rejected
)]
#[case::bad_credentials(
  crate::AuthServiceError::TokenExchangeError("invalid_client".to_string()),
  TunnelAuthSyncState::Rejected
)]
fn classifies_sync_failures_by_whether_keycloak_answered(
  #[case] error: crate::AuthServiceError,
  #[case] expected: TunnelAuthSyncState,
) {
  assert_eq!(
    expected,
    DefaultTunnelService::classify_sync_failure(&error)
  );
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn sends_a_byte_identical_redirect_patch_across_enable_disable_cycles() -> anyhow::Result<()>
{
  let harness = harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME)).await;

  for _ in 0..3 {
    harness.service.enable(enable_request()).await?;
    harness.service.disable().await?;
  }

  let calls = harness.redirect_calls.lock().unwrap().clone();
  assert_eq!(3, calls.len(), "one registration per enable");
  assert!(
    calls.windows(2).all(|pair| pair[0] == pair[1]),
    "every cycle must send the same body, or Keycloak accumulates redirect URIs instead of replacing: {calls:?}"
  );
  assert_eq!(
    (
      client_id(),
      "client-secret".to_string(),
      "cloudflared".to_string(),
      format!("https://{SUBDOMAIN}.{FAKE_ZONE}/ui/auth/callback"),
    ),
    calls[0]
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn removes_the_previous_cname_when_the_subdomain_changes() -> anyhow::Result<()> {
  let harness = harness(
    FakeCloudflared::with_existing_tunnel(TUNNEL_NAME).with_dns_records(&format!(
      r#"{{"success":true,"result":[{{"id":"rec-1","content":"{FAKE_TUNNEL_ID}.cfargotunnel.com"}}]}}"#
    )),
  )
  .await;

  harness.service.enable(enable_request()).await?;
  harness
    .service
    .enable(EnableTunnelRequest {
      subdomain: "moved".to_string(),
      auto_reconnect: true,
      replace_dns: false,
    })
    .await?;

  assert_eq!(
    vec!["rec-1".to_string()],
    harness.fake.deleted_dns_records()
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn keeps_the_cname_when_the_subdomain_is_unchanged() -> anyhow::Result<()> {
  let harness = harness(
    FakeCloudflared::with_existing_tunnel(TUNNEL_NAME).with_dns_records(&format!(
      r#"{{"success":true,"result":[{{"id":"rec-1","content":"{FAKE_TUNNEL_ID}.cfargotunnel.com"}}]}}"#
    )),
  )
  .await;

  harness.service.enable(enable_request()).await?;
  harness.service.enable(enable_request()).await?;

  assert!(
    harness.fake.deleted_dns_records().is_empty(),
    "the record for the unchanged address must survive"
  );
  Ok(())
}

#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn learns_the_metrics_address_from_the_connectors_own_output() -> anyhow::Result<()> {
  let harness =
    harness(FakeCloudflared::with_existing_tunnel(TUNNEL_NAME).with_metrics_line()).await;
  harness.service.enable(enable_request()).await?;

  wait_for(&harness.service, |snapshot| {
    snapshot.state == TunnelConnectionState::Connected
  })
  .await;

  let observed = harness
    .service
    .runtime
    .lock()
    .unwrap()
    .running
    .as_ref()
    .and_then(|running| running.metrics_addr.lock().unwrap().clone());
  assert_eq!(Some(FAKE_METRICS_ADDR.to_string()), observed);
  Ok(())
}
