#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod app_dirs_builder;
mod app_service_builder;
mod error;

pub use app_dirs_builder::*;
pub use app_service_builder::*;
pub use error::*;

// Re-exports for crates/bodhi dependency isolation
// Domain objects from objs crate
pub use objs::{AppError, AppType, EnvType, ErrorMessage, ErrorType, LogLevel};

// Service interfaces and implementations from services crate
pub use services::{
  AppService, DefaultEnvWrapper, DefaultSettingService, EnvWrapper, SettingService,
  BODHI_EXEC_LOOKUP_PATH, BODHI_LOGS, BODHI_LOG_STDOUT, DEFAULT_HOST, DEFAULT_PORT,
};

// Server management from server_app crate
pub use server_app::{ServeCommand, ServeError, ServerShutdownHandle};

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
