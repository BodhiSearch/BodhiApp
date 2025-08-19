use crate::{
  config::{try_build_app_options_internal, NapiAppOptions},
  BODHI_HOST, BODHI_PORT,
};
use lib_bodhiserver::{
  build_app_service, setup_app_dirs, update_with_option, ApiError, AppService, OpenAIApiError,
  ServeCommand, ServerShutdownHandle, SettingService, BODHI_LOG_STDOUT, DEFAULT_HOST, DEFAULT_PORT,
  EMBEDDED_UI_ASSETS,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// The main Bodhi server wrapper for NAPI
#[napi]
pub struct BodhiServer {
  config: NapiAppOptions,
  shutdown_handle: Arc<Mutex<Option<ServerShutdownHandle>>>,
  temp_dir: Option<TempDir>,
  log_guard: Option<WorkerGuard>,
}

#[napi]
impl BodhiServer {
  /// Create a new Bodhi server instance with the given configuration
  #[napi(constructor)]
  pub fn new(config: NapiAppOptions) -> Result<Self> {
    Ok(Self {
      config,
      shutdown_handle: Arc::new(Mutex::new(None)),
      temp_dir: None,
      log_guard: None,
    })
  }

  /// Get the server configuration
  #[napi(getter)]
  pub fn config(&self) -> NapiAppOptions {
    self.config.clone()
  }

  /// Get the server URL
  #[napi]
  pub fn server_url(&self) -> String {
    let host = self.host();
    let port = self.port();
    format!("http://{}:{}", host, port)
  }

  /// Get the server host
  #[napi]
  pub fn host(&self) -> String {
    self
      .config
      .env_vars
      .get(BODHI_HOST)
      .cloned()
      .unwrap_or_else(|| DEFAULT_HOST.to_string())
  }

  /// Get the server port
  #[napi]
  pub fn port(&self) -> u16 {
    self
      .config
      .env_vars
      .get(BODHI_PORT)
      .and_then(|port_str| port_str.parse().ok())
      .unwrap_or(DEFAULT_PORT)
  }

  /// Start the Bodhi server
  ///
  /// # Safety
  /// Safe to call from JavaScript/Node.js context via NAPI bindings.
  #[napi]
  pub async unsafe fn start(&mut self) -> Result<()> {
    // Check if server is already running
    {
      let handle_guard = self.shutdown_handle.lock().await;
      if handle_guard.is_some() {
        return Err(Error::new(
          Status::InvalidArg,
          "Server is already running".to_string(),
        ));
      }
    }
    // Build app options from the config
    let builder = try_build_app_options_internal(self.config.clone()).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to build app options: {}", e),
      )
    })?;
    let app_options = builder.build().map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to build app options: {}", e),
      )
    })?;

    // Setup app directories and settings
    let setting_service = Arc::new(setup_app_dirs(&app_options).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to setup app dirs: {}", e),
      )
    })?);

    // Setup logging
    let log_guard = setup_logs(&setting_service);
    self.log_guard = Some(log_guard);

    // Build the app service
    let app_service: Arc<dyn AppService> = Arc::new(
      build_app_service(setting_service.clone())
        .await
        .map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to build app service: {}", e),
          )
        })?,
    );
    update_with_option(&app_service, (&app_options).into())
      .map_err(|err| Error::new(Status::GenericFailure, err))?;
    // Create and start the server
    let serve_command = ServeCommand::ByParams {
      host: self.host(),
      port: self.port(),
    };

    let handle = serve_command
      .get_server_handle(app_service, Some(&EMBEDDED_UI_ASSETS))
      .await
      .map_err(|e| {
        let err: ApiError = e.into();
        let err: OpenAIApiError = err.into();
        Error::new(
          Status::GenericFailure,
          format!("Failed to start server: {}", err),
        )
      })?;

    // Store the shutdown handle
    {
      let mut handle_guard = self.shutdown_handle.lock().await;
      *handle_guard = Some(handle);
    }

    Ok(())
  }

  /// Stop the Bodhi server
  ///
  /// # Safety
  /// Safe to call from JavaScript/Node.js context via NAPI bindings.
  #[napi]
  pub async unsafe fn stop(&mut self) -> Result<()> {
    let handle = {
      let mut handle_guard = self.shutdown_handle.lock().await;
      handle_guard.take()
    };

    if let Some(handle) = handle {
      handle.shutdown().await.map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to shutdown server: {}", e),
        )
      })?;
    }
    // Clean up log guard
    if let Some(guard) = self.log_guard.take() {
      drop(guard);
    }
    Ok(())
  }

  /// Check if the server is running
  ///
  /// # Safety
  /// Safe to call from JavaScript/Node.js context via NAPI bindings.
  #[napi]
  pub async unsafe fn is_running(&self) -> Result<bool> {
    let handle_guard = self.shutdown_handle.lock().await;
    Ok(handle_guard.is_some())
  }

  /// Get server ping status
  ///
  /// # Safety
  /// Safe to call from JavaScript/Node.js context via NAPI bindings.
  #[napi]
  pub async unsafe fn ping(&self) -> Result<bool> {
    let is_running = {
      let handle_guard = self.shutdown_handle.lock().await;
      handle_guard.is_some()
    };

    if !is_running {
      return Ok(false);
    }

    // Try to make a simple HTTP request to the server
    let url = format!("{}/ping", self.server_url());
    match reqwest::get(&url).await {
      Ok(response) => Ok(response.status().is_success()),
      Err(_) => Ok(false),
    }
  }
}

fn setup_logs(setting_service: &lib_bodhiserver::DefaultSettingService) -> WorkerGuard {
  let logs_dir = setting_service.logs_dir();
  let file_appender = tracing_appender::rolling::daily(logs_dir, "bodhi.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  let log_level: LevelFilter = setting_service.log_level().into();
  let log_level = log_level.to_string();
  let filter = EnvFilter::new(&log_level);
  let filter = filter.add_directive("hf_hub=error".parse().expect("is a valid directive"));
  // Reduce verbose middleware logging noise
  let filter = filter.add_directive("tower_sessions=warn".parse().expect("is a valid directive"));
  let filter = filter.add_directive("tower_http=warn".parse().expect("is a valid directive"));
  let filter = filter.add_directive(
    "tower_sessions_core=warn"
      .parse()
      .expect("is a valid directive"),
  );

  // Check if we should output to stdout
  let enable_stdout = cfg!(debug_assertions)
    || setting_service
      .get_setting(BODHI_LOG_STDOUT)
      .map(|v| v == "1" || v.to_lowercase() == "true")
      .unwrap_or(false);

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
  // Handle the case where subscriber is already set (e.g., in tests)
  if result.is_err() {
    #[cfg(debug_assertions)]
    {
      println!("logging subscriber already set, continuing with existing setup");
    }
  } else {
    #[cfg(debug_assertions)]
    {
      println!(
        "logging to stdout: {}, log_level: {}",
        enable_stdout, log_level
      );
    }
  }
  guard
}

impl Drop for BodhiServer {
  fn drop(&mut self) {
    // We can't use async in Drop, but we can at least handle cleanup
    // The server shutdown should happen explicitly via stop()
    if let Some(_temp_dir) = self.temp_dir.take() {
      // temp_dir will be automatically cleaned up when dropped
    }
    if let Some(_log_guard) = self.log_guard.take() {
      // log_guard will be automatically cleaned up when dropped
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    test_utils::test_config, BodhiServer, NapiAppOptions, BODHI_HOME, BODHI_HOST, BODHI_PORT,
  };
  use objs::{test_utils::setup_l10n, FluentLocalizationService};
  use rstest::rstest;
  use std::sync::Arc;
  use tempfile::TempDir;
  use tokio::time::{sleep, Duration};

  #[rstest]
  #[tokio::test]
  async fn test_server_lifecycle(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    test_config: (NapiAppOptions, TempDir),
  ) {
    let (config, _temp_dir) = test_config;
    let mut server = BodhiServer::new(config).expect("Failed to create server");

    // Test initial state
    let is_running = unsafe {
      server
        .is_running()
        .await
        .expect("Failed to check if running")
    };
    assert!(!is_running);

    // Start the server
    unsafe {
      server.start().await.expect("Failed to start server");
    }
    let is_running = unsafe {
      server
        .is_running()
        .await
        .expect("Failed to check if running")
    };
    assert!(is_running);

    // Give the server a moment to fully start
    sleep(Duration::from_millis(1000)).await;

    // Test ping
    let ping_response = unsafe { server.ping().await.expect("Failed to ping server") };
    assert!(ping_response);

    // Stop the server
    unsafe {
      server.stop().await.expect("Failed to stop server");
    }
    let is_running = unsafe {
      server
        .is_running()
        .await
        .expect("Failed to check if running")
    };
    assert!(!is_running);
  }

  #[rstest]
  #[tokio::test]
  async fn test_server_config_access(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    test_config: (NapiAppOptions, TempDir),
  ) {
    let (config, _temp_dir) = test_config;
    let server = BodhiServer::new(config).expect("Failed to create server");

    // Test that we can access config values
    assert!(!server.config().env_vars.get(BODHI_HOME).unwrap().is_empty());
    assert_eq!(server.host(), "127.0.0.1");
    assert!(server.port() > 0);
  }

  #[tokio::test]
  #[rstest]
  async fn test_server_already_running_error(
    #[from(setup_l10n)] _setup_l10n: &Arc<FluentLocalizationService>,
    test_config: (NapiAppOptions, TempDir),
  ) {
    let (config, _temp_dir) = test_config;
    let mut server = BodhiServer::new(config).expect("Failed to create server");

    // Start the server
    unsafe {
      server.start().await.expect("Failed to start server");
    }

    // Try to start again - should fail
    let result = unsafe { server.start().await };
    assert!(result.is_err());

    // Clean up
    unsafe {
      server.stop().await.expect("Failed to stop server");
    }
  }

  #[rstest]
  fn test_server_creation(test_config: (NapiAppOptions, TempDir)) {
    let (mut config, _temp_dir) = test_config;
    config = crate::config::set_env_var(config, BODHI_HOME.to_string(), "/tmp/bodhi".to_string());
    config = crate::config::set_env_var(config, BODHI_HOST.to_string(), "127.0.0.1".to_string());
    config = crate::config::set_env_var(config, BODHI_PORT.to_string(), "25000".to_string());

    let server = BodhiServer::new(config.clone()).expect("Failed to create server");

    assert_eq!(
      server.config().env_vars.get(BODHI_HOME),
      Some(&"/tmp/bodhi".to_string())
    );
    assert_eq!(server.host(), "127.0.0.1");
    assert_eq!(server.port(), 25000);
  }

  #[rstest]
  fn test_server_config_values(test_config: (NapiAppOptions, TempDir)) {
    let (config, _temp_dir) = test_config;
    let server = BodhiServer::new(config).expect("Failed to create server");

    assert!(!server.config().env_vars.get(BODHI_HOME).unwrap().is_empty());
    assert_eq!(server.host(), "127.0.0.1");
    assert!(server.port() > 0);
  }

  #[test]
  fn test_server_url() {
    let mut config = crate::config::create_napi_app_options();
    config = crate::config::set_env_var(config, BODHI_HOST.to_string(), "localhost".to_string());
    config = crate::config::set_env_var(config, BODHI_PORT.to_string(), "8080".to_string());

    let server = BodhiServer::new(config.clone()).expect("Failed to create server");

    let expected_url = "http://localhost:8080";
    assert_eq!(server.server_url(), expected_url);
  }
}
