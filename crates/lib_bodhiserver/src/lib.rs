#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod app_dirs_builder;
mod app_options;
mod app_service_builder;
mod bootstrap_service;
mod error;
mod ui_assets;

/// Git commit SHA captured at build time. "undefined" if not available.
pub const BUILD_COMMIT_SHA: &str = match option_env!("BODHI_BUILD_COMMIT_SHA") {
  Some(sha) => sha,
  None => "undefined",
};

pub use app_dirs_builder::*;
pub use app_options::*;
pub use app_service_builder::*;
pub use bootstrap_service::BootstrapService;
pub use error::*;
pub use services::BootstrapParts;
pub use ui_assets::EMBEDDED_UI_ASSETS;

// Re-exports for crates/bodhi dependency isolation
// Domain objects from objs crate
pub use objs::{
  ApiError, AppCommand, AppError, AppType, EnvType, ErrorType, LogLevel, OpenAIApiError,
};

// Service interfaces and implementations from services crate
pub use services::{
  AppInstance,
  AppService,
  AppStatus,
  DefaultAppService,
  DefaultEnvWrapper,
  DefaultSettingService,
  EnvWrapper,
  SettingService,
  BODHI_APP_TYPE,
  BODHI_AUTH_REALM,
  BODHI_AUTH_URL,
  BODHI_COMMIT_SHA,
  BODHI_ENCRYPTION_KEY,
  BODHI_ENV_TYPE,
  BODHI_EXEC_LOOKUP_PATH,
  BODHI_EXEC_VARIANT,
  // Setting constants for unified configuration
  BODHI_HOME,
  BODHI_HOST,
  BODHI_KEEP_ALIVE_SECS,
  BODHI_LOGS,
  BODHI_LOG_LEVEL,
  BODHI_LOG_STDOUT,
  BODHI_PORT,
  BODHI_PUBLIC_HOST,
  BODHI_PUBLIC_PORT,
  BODHI_PUBLIC_SCHEME,
  BODHI_SCHEME,
  BODHI_VERSION,
  DEFAULT_HOST,
  DEFAULT_PORT,
  DEFAULT_SCHEME,
  HF_HOME,
};

// Re-export services module for external access
pub use services;

// External dependencies needed for JWT
pub use jsonwebtoken;

// Server management from server_app crate
pub use server_app::{ServeCommand, ServeError, ServerShutdownHandle};
