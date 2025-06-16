use crate::config::AppConfig;
use lib_bodhiserver::{
  build_app_service, create_static_dir_from_path, setup_app_dirs, ServeCommand,
  ServerShutdownHandle,
};
use napi_derive::napi;
use objs::{test_utils::set_mock_localization_service, FluentLocalizationService};
use std::sync::Arc;

/// Application state enumeration for tracking lifecycle
#[napi]
#[derive(Debug, PartialEq)]
pub enum AppState {
  Uninitialized,
  Ready,
  Running,
  Shutdown,
}

/// Main NAPI wrapper for BodhiApp server functionality
#[napi]
pub struct BodhiApp {
  state: AppState,
  app_service: Option<Arc<lib_bodhiserver::DefaultAppService>>,
  server_handle: Option<ServerShutdownHandle>,
}

#[napi]
impl BodhiApp {
  /// Creates a new uninitialized BodhiApp instance
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      state: AppState::Uninitialized,
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
    if self.state != AppState::Uninitialized {
      return Err(napi::Error::from_reason("App already initialized"));
    }

    // Initialize localization service for test environment
    let localization_service = Arc::new(FluentLocalizationService::new_standalone());
    set_mock_localization_service(localization_service.clone());

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

    self.app_service = Some(Arc::new(app_service));
    self.state = AppState::Ready;

    Ok(())
  }

  /// Starts the HTTP server with the specified configuration
  ///
  /// # Arguments
  /// * `host` - The host address to bind to (e.g., "127.0.0.1")
  /// * `port` - The port number to bind to (0 for automatic port selection)
  /// * `assets_path` - Optional path to static assets directory
  ///
  /// # Returns
  /// The server URL (e.g., "http://127.0.0.1:3000")
  #[napi]
  pub async unsafe fn start_server(
    &mut self,
    host: String,
    port: u16,
    assets_path: Option<String>,
  ) -> napi::Result<String> {
    if self.state != AppState::Ready {
      return Err(napi::Error::from_reason(
        "App not initialized or already running",
      ));
    }

    let app_service = self
      .app_service
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("App service not available"))?;

    // Handle assets using the utility function
    let assets_dir = if let Some(path) = assets_path {
      Some(create_static_dir_from_path(&path).map_err(|e| napi::Error::from_reason(e.to_string()))?)
    } else {
      None
    };

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
    self.state = AppState::Running;

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
    self.state = AppState::Shutdown;

    Ok(())
  }

  /// Gets the current application state
  #[napi]
  pub fn get_status(&self) -> u32 {
    match self.state {
      AppState::Uninitialized => 0,
      AppState::Ready => 1,
      AppState::Running => 2,
      AppState::Shutdown => 3,
    }
  }
}
