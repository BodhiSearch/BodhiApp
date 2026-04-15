//! Dev/E2E launcher for BodhiApp.
//!
//! Configures [`lib_bodhiserver`] entirely from environment variables, forces
//! `BODHI_DEV_PROXY_UI=true`, and starts the HTTP server without any embedded
//! UI assets — the Rust server proxies `/ui/*` to a separately-running Vite
//! dev server at `http://localhost:3000`. Intended to be spawned from the
//! Playwright E2E suite in place of the NAPI bindings.

use std::{env, env::VarError, fs, io::Write, process::ExitCode, sync::Arc};

use lib_bodhiserver::{
  build_app_service, services::new_ulid, setup_app_dirs, setup_bootstrap_service, AppCommand,
  AppOptionsBuilder, AppService, AppStatus, BootstrapError, BootstrapService, ServeCommand, Tenant,
  BODHI_APP_TYPE, BODHI_AUTH_REALM, BODHI_AUTH_URL, BODHI_DEPLOYMENT, BODHI_ENV_TYPE, BODHI_HOST,
  BODHI_PORT, BODHI_VERSION, DEFAULT_HOST, DEFAULT_PORT,
};
use tokio::signal;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const BODHI_DEV_PROXY_UI: &str = "BODHI_DEV_PROXY_UI";
// Extra env vars mirroring the NAPI config surface. Kept here so the test
// harness contract lives with the binary, not in the library.
const BODHI_CLIENT_ID: &str = "BODHI_CLIENT_ID";
const BODHI_CLIENT_SECRET: &str = "BODHI_CLIENT_SECRET";
const BODHI_APP_STATUS: &str = "BODHI_APP_STATUS";
const BODHI_TENANT_NAME: &str = "BODHI_TENANT_NAME";
const BODHI_CREATED_BY: &str = "BODHI_CREATED_BY";
const BODHI_EXEC_LOOKUP_PATH: &str = "BODHI_EXEC_LOOKUP_PATH";
const BODHI_LOG_LEVEL: &str = "BODHI_LOG_LEVEL";
const BODHI_LOG_STDOUT: &str = "BODHI_LOG_STDOUT";

// System settings propagated from process env when present.
const SYSTEM_SETTING_KEYS: &[&str] = &[
  BODHI_ENV_TYPE,
  BODHI_APP_TYPE,
  BODHI_VERSION,
  BODHI_AUTH_URL,
  BODHI_AUTH_REALM,
  BODHI_DEPLOYMENT,
];

// App settings propagated from process env when present.
const APP_SETTING_KEYS: &[&str] = &[BODHI_EXEC_LOOKUP_PATH, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT];

fn main() -> ExitCode {
  match run() {
    Ok(()) => ExitCode::SUCCESS,
    Err(err) => {
      eprintln!("bodhiserver_dev: fatal: {err:#}");
      ExitCode::from(1)
    }
  }
}

fn run() -> anyhow::Result<()> {
  if env::var(crate_consts::BODHI_HOME).is_err() {
    anyhow::bail!("BODHI_HOME must be set (dev binary does not create temp dirs)");
  }

  // The routes layer only honours BODHI_DEV_PROXY_UI under debug_assertions
  // via SettingService::get_dev_env. Dev binary is always compiled with
  // debug_assertions (cargo build/run default), so this works.
  env::set_var(BODHI_DEV_PROXY_UI, "true");

  let host = env_or(BODHI_HOST, DEFAULT_HOST);
  let port: u16 = env::var(BODHI_PORT)
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(DEFAULT_PORT);

  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;

  runtime.block_on(async move {
    let builder = build_options_from_env()?;
    let app_options = builder.build()?;

    let (bodhi_home, source, file_defaults) = setup_app_dirs(&app_options)?;
    let bootstrap = setup_bootstrap_service(
      &app_options,
      bodhi_home,
      source,
      file_defaults,
      AppCommand::Default,
    )?;

    let _log_guard = setup_logs(&bootstrap)?;
    let parts = bootstrap.into_parts();

    let app_service_inner = build_app_service(parts).await?;
    let app_service: Arc<dyn AppService> = Arc::new(app_service_inner);
    ensure_tenant(&app_service, app_options.tenant.as_ref()).await?;

    let serve_command = ServeCommand::ByParams {
      host: host.clone(),
      port,
    };
    let handle = serve_command.get_server_handle(app_service, None).await?;

    // Signal readiness on stdout so the JS launcher can begin its checks.
    let url = format!("http://{host}:{port}");
    println!("bodhiserver_dev: listening on {url}");
    let _ = std::io::stdout().flush();

    signal::ctrl_c().await.ok();
    handle.shutdown().await?;
    Ok::<_, anyhow::Error>(())
  })
}

fn build_options_from_env() -> Result<AppOptionsBuilder, BootstrapError> {
  let mut builder = AppOptionsBuilder::default();

  // Propagate every known env var into the EnvWrapper so downstream
  // SettingService lookups see consistent values.
  for key in env::vars().filter_map(|(k, _)| {
    if k.starts_with("BODHI_") || k == "HOME" || k == "HF_HOME" {
      Some(k)
    } else {
      None
    }
  }) {
    if let Ok(value) = env::var(&key) {
      builder = builder.set_env(&key, &value);
    }
  }

  for key in APP_SETTING_KEYS {
    if let Some(value) = env_opt(key) {
      builder = builder.set_app_setting(key, &value);
    }
  }

  for key in SYSTEM_SETTING_KEYS {
    if let Some(value) = env_opt(key) {
      builder = builder.set_system_setting(key, &value)?;
    }
  }

  if let (Some(client_id), Some(client_secret)) =
    (env_opt(BODHI_CLIENT_ID), env_opt(BODHI_CLIENT_SECRET))
  {
    let status = match env_opt(BODHI_APP_STATUS) {
      Some(value) => value.parse::<AppStatus>().map_err(|_| {
        BootstrapError::ValidationError(format!("Invalid {BODHI_APP_STATUS}: {value}"))
      })?,
      None => AppStatus::Ready,
    };
    let tenant_name = env_opt(BODHI_TENANT_NAME).unwrap_or_else(|| "BodhiApp".to_string());
    let now = chrono::Utc::now();
    let tenant = Tenant {
      id: new_ulid(),
      client_id,
      client_secret,
      name: tenant_name,
      description: None,
      status,
      created_by: env_opt(BODHI_CREATED_BY),
      created_at: now,
      updated_at: now,
    };
    builder = builder.set_tenant(tenant);
  }

  Ok(builder)
}

fn env_opt(key: &str) -> Option<String> {
  match env::var(key) {
    Ok(value) if !value.is_empty() => Some(value),
    Ok(_) | Err(VarError::NotPresent) => None,
    Err(VarError::NotUnicode(_)) => None,
  }
}

fn env_or(key: &str, default: &str) -> String {
  env_opt(key).unwrap_or_else(|| default.to_string())
}

async fn ensure_tenant(
  service: &Arc<dyn AppService>,
  tenant: Option<&Tenant>,
) -> Result<(), BootstrapError> {
  if let Some(tenant) = tenant {
    service.db_service().create_tenant_test(tenant).await?;
  }
  Ok(())
}

// Logging setup copied from lib_bodhiserver_napi::server::setup_logs. Test-infra
// logic intentionally kept inside the bin so the library surface stays clean.
fn setup_logs(bootstrap_service: &BootstrapService) -> Result<WorkerGuard, std::io::Error> {
  let logs_dir = bootstrap_service.logs_dir();
  fs::create_dir_all(&logs_dir)?;
  let file_appender = tracing_appender::rolling::daily(logs_dir, "bodhi.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  let log_level: LevelFilter = bootstrap_service.log_level().into();
  let filter = EnvFilter::new(log_level.to_string())
    .add_directive("hf_hub=error".parse().expect("valid directive"))
    .add_directive("tower_sessions=warn".parse().expect("valid directive"))
    .add_directive("tower_http=warn".parse().expect("valid directive"))
    .add_directive("tower_sessions_core=warn".parse().expect("valid directive"))
    .add_directive("sqlx::query=warn".parse().expect("valid directive"));
  let enable_stdout = cfg!(debug_assertions) || bootstrap_service.log_stdout();

  let subscriber = tracing_subscriber::registry().with(filter);
  let result = if enable_stdout {
    subscriber
      .with(
        fmt::layer()
          .with_writer(std::io::stdout)
          .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
          .with_target(true),
      )
      .with(
        fmt::layer()
          .with_writer(non_blocking)
          .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
          .with_target(true),
      )
      .try_init()
  } else {
    subscriber
      .with(
        fmt::layer()
          .with_writer(non_blocking)
          .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
          .with_target(true),
      )
      .try_init()
  };
  if result.is_err() {
    #[cfg(debug_assertions)]
    eprintln!("bodhiserver_dev: logging subscriber already initialised");
  }
  Ok(guard)
}

mod crate_consts {
  // Re-exported constant name so the top-level `env::var` call reads the same
  // key as everything else without importing the full lib_bodhiserver surface.
  pub const BODHI_HOME: &str = lib_bodhiserver::BODHI_HOME;
}
