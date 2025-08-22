#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod app_service;
mod auth_service;
mod cache_service;
mod data_service;
pub mod db;
mod env_wrapper;
mod hub_service;
mod keyring_service;
mod macros;
mod obj_exts;
mod objs;
mod progress_tracking;
mod secret_service;
mod service_ext;
mod session_service;
mod setting_service;
mod token;

pub use app_service::*;
pub use auth_service::*;
pub use cache_service::*;
pub use data_service::*;
pub use env_wrapper::*;
pub use hub_service::*;
pub use keyring_service::*;
// obj_exts module is currently empty after chat template removal
pub use objs::*;
pub use progress_tracking::*;
pub use secret_service::*;
pub use service_ext::*;
pub use session_service::*;
pub use setting_service::*;
pub use token::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
