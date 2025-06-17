use crate::config::AppConfig;
use lib_bodhiserver::{
  build_app_service, setup_app_dirs, update_with_option, AppService, AppStateOption, ServeCommand,
  ServerShutdownHandle, EMBEDDED_UI_ASSETS,
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

#[napi]
/// Main NAPI wrapper for BodhiApp server functionality
pub struct BodhiApp {
  state: NapiAppState,
  app_service: Option<Arc<dyn AppService>>,
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

    // Use lib_bodhiserver's interface
    let setting_service =
      setup_app_dirs(&app_options).map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let app_service = build_app_service(Arc::new(setting_service))
      .await
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let app_service: Arc<dyn AppService> = Arc::new(app_service);
    update_with_option(&app_service, AppStateOption::from(&app_options))
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    self.app_service = Some(app_service);
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::AppConfig;
  use lib_bodhiserver::FluentLocalizationService;
  use objs::test_utils::{setup_l10n, temp_dir};
  use rstest::rstest;
  use std::{collections::HashMap, sync::Arc};
  use tempfile::TempDir;

  #[rstest]
  #[tokio::test]
  async fn test_bodhi_app_initialize_with_enhanced_config(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    temp_dir: TempDir,
  ) -> Result<(), napi::Error> {
    // Create enhanced config
    let mut env_vars = HashMap::new();
    env_vars.insert(
      "BODHI_ENCRYPTION_KEY".to_string(),
      "test-encryption-key-enhanced".to_string(),
    );
    env_vars.insert("BODHI_EXEC_LOOKUP_PATH".to_string(), "/tmp".to_string());
    env_vars.insert("BODHI_PORT".to_string(), "54322".to_string());
    env_vars.insert(
      "BODHI_HOME".to_string(),
      temp_dir.path().display().to_string(),
    );

    let app_settings = HashMap::new();

    let config = AppConfig {
      env_type: "development".to_string(),
      app_type: "container".to_string(),
      app_version: "1.0.0-test".to_string(),
      auth_url: "https://dev-id.getbodhi.app".to_string(),
      auth_realm: "bodhi".to_string(),
      environment_vars: Some(env_vars),
      app_settings: Some(app_settings),
      oauth_client_id: Some("test_client_id".to_string()),
      oauth_client_secret: Some("test_client_secret".to_string()),
      app_status: Some("ready".to_string()),
    };

    let mut app = BodhiApp::new();
    unsafe {
      app.initialize(config).await?;
    }

    // Verify the app is in Ready state
    assert_eq!(app.state, NapiAppState::Ready);
    assert!(app.app_service.is_some());

    Ok(())
  }

  #[rstest]
  #[tokio::test]
  async fn test_bodhi_app_lifecycle(
    #[from(setup_l10n)] _l10n: &Arc<FluentLocalizationService>,
    temp_dir: TempDir,
  ) -> Result<(), napi::Error> {
    let mut config = AppConfig::development();
    let mut env_vars = config.environment_vars.unwrap_or_default();
    env_vars.insert(
      "BODHI_ENCRYPTION_KEY".to_string(),
      "test-encryption-key-lifecycle".to_string(),
    );
    env_vars.insert(
      "BODHI_HOME".to_string(),
      temp_dir.path().display().to_string(),
    );
    config.environment_vars = Some(env_vars);

    let mut app = BodhiApp::new();

    // Test initialization
    unsafe {
      app.initialize(config).await?;
    }
    assert_eq!(app.state, NapiAppState::Ready);
    assert!(app.app_service.is_some());

    // Test status getter
    assert_eq!(app.get_status(), 1); // Ready state

    // Test shutdown without starting server
    unsafe {
      app.shutdown().await?;
    }
    assert_eq!(app.state, NapiAppState::Shutdown);
    assert_eq!(app.get_status(), 3); // Shutdown state
    assert!(app.app_service.is_none());

    Ok(())
  }
}
