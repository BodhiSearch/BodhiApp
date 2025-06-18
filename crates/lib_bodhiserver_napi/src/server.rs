use crate::{
  config::{try_build_app_options_internal, NapiAppOptions},
  BODHI_HOST, BODHI_PORT,
};
use lib_bodhiserver::{
  build_app_service, setup_app_dirs, update_with_option, ApiError, AppService, OpenAIApiError,
  ServeCommand, ServerShutdownHandle, DEFAULT_HOST, DEFAULT_PORT, EMBEDDED_UI_ASSETS,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

/// The main Bodhi server wrapper for NAPI
#[napi]
pub struct BodhiServer {
  config: NapiAppOptions,
  shutdown_handle: Arc<Mutex<Option<ServerShutdownHandle>>>,
  temp_dir: Option<TempDir>,
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

    Ok(())
  }

  /// Check if the server is running
  #[napi]
  pub async unsafe fn is_running(&self) -> Result<bool> {
    let handle_guard = self.shutdown_handle.lock().await;
    Ok(handle_guard.is_some())
  }

  /// Get server ping status
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

impl Drop for BodhiServer {
  fn drop(&mut self) {
    // We can't use async in Drop, but we can at least handle temp_dir cleanup
    // The server shutdown should happen explicitly via stop()
    if let Some(_temp_dir) = self.temp_dir.take() {
      // temp_dir will be automatically cleaned up when dropped
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
