#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod auth_middleware;
mod token_service;
mod utils;
mod api_auth_middleware;

pub use auth_middleware::*;
pub use token_service::*;
pub use utils::*;
pub use api_auth_middleware::api_auth_middleware;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
