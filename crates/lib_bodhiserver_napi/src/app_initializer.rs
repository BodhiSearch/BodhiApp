use crate::config::AppConfig;
use lib_bodhiserver::{
  build_app_service, setup_app_dirs, AppRegInfo, AppService, AppStatus,
  SecretServiceExt, ServeCommand, ServerShutdownHandle, EMBEDDED_UI_ASSETS,
};
use napi_derive::napi;
use std::sync::Arc;

/// NAPI application state enumeration for tracking lifecycle
/// Renamed from AppState to avoid confusion with objs::AppStatus
#[napi]
#[derive(Debug, PartialEq)]
pub enum NapiAppState {
  Uninitialized,
  Ready,
  Running,
  Shutdown,
}

/// Main NAPI wrapper for BodhiApp server functionality
#[napi]
pub struct BodhiApp {
  state: NapiAppState,
  app_service: Option<Arc<lib_bodhiserver::DefaultAppService>>,
  server_handle: Option<ServerShutdownHandle>,
}

#[napi]
impl BodhiApp {
  /// Creates a new uninitialized BodhiApp instance
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      state: NapiAppState::Uninitialized,
      app_service: None,
      server_handle: None,
    }
  }

  /// Initializes the BodhiApp with the provided configuration
  ///
  /// This function sets up application directories, loads settings,
  /// and builds all required services using the isolated lib_bodhiserver interface.
  #[napi]
  pub async unsafe fn initialize(&mut self, config: AppConfig) -> napi::Result<()> {
    if self.state != NapiAppState::Uninitialized {
      return Err(napi::Error::from_reason("App already initialized"));
    }

    // Convert FFI config to lib_bodhiserver AppOptions
    let app_options: lib_bodhiserver::AppOptions = config
      .try_into()
      .map_err(|e: String| napi::Error::from_reason(e))?;

    // Use lib_bodhiserver's isolated interface
    let setting_service =
      setup_app_dirs(app_options).map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let app_service = build_app_service(Arc::new(setting_service))
      .await
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    // Configure authentication using integration test values (similar to live_server_utils.rs)
    let app_reg_info = AppRegInfo {
      client_id: "resource-28f0cef6-cd2d-45c3-a162-f7a6a9ff30ce".to_string(),
      client_secret: "WxfJHaMUfqwcE8dUmaqvsZWqwq4TonlS".to_string(),
    };

    // Set app registration info and status to Ready for testing
    app_service
      .secret_service()
      .set_app_reg_info(&app_reg_info)
      .map_err(|e| napi::Error::from_reason(format!("Failed to set app reg info: {}", e)))?;

    app_service
      .secret_service()
      .set_app_status(&AppStatus::Ready)
      .map_err(|e| napi::Error::from_reason(format!("Failed to set app status: {}", e)))?;

    self.app_service = Some(Arc::new(app_service));
    self.state = NapiAppState::Ready;

    Ok(())
  }

  /// Starts the HTTP server with the specified configuration
  ///
  /// # Arguments
  /// * `host` - The host address to bind to (e.g., "127.0.0.1")
  /// * `port` - The port number to bind to (0 for automatic port selection)
  ///
  /// # Returns
  /// The server URL (e.g., "http://127.0.0.1:3000")
  #[napi]
  pub async unsafe fn start_server(&mut self, host: String, port: u16) -> napi::Result<String> {
    if self.state != NapiAppState::Ready {
      return Err(napi::Error::from_reason(
        "App not initialized or already running",
      ));
    }

    let app_service = self
      .app_service
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("App service not available"))?;

    // Use embedded UI assets from lib_bodhiserver
    let assets_dir = Some(&EMBEDDED_UI_ASSETS);

    let command = ServeCommand::ByParams {
      host: host.clone(),
      port,
    };

    let handle = command
      .get_server_handle(app_service.clone(), assets_dir)
      .await
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    // Extract the actual port from the server handle if port was 0
    let actual_port = if port == 0 {
      // TODO: Extract actual port from server handle
      // For now, we'll use the provided port
      port
    } else {
      port
    };

    self.server_handle = Some(handle);
    self.state = NapiAppState::Running;

    Ok(format!("http://{}:{}", host, actual_port))
  }

  /// Shuts down the server and cleans up resources
  #[napi]
  pub async unsafe fn shutdown(&mut self) -> napi::Result<()> {
    if let Some(handle) = self.server_handle.take() {
      handle
        .shutdown()
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    }

    self.app_service = None;
    self.state = NapiAppState::Shutdown;

    Ok(())
  }

  /// Gets the current NAPI application state
  #[napi]
  pub fn get_status(&self) -> u32 {
    match self.state {
      NapiAppState::Uninitialized => 0,
      NapiAppState::Ready => 1,
      NapiAppState::Running => 2,
      NapiAppState::Shutdown => 3,
    }
  }
}
