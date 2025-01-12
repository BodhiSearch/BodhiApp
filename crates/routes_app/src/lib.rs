#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod error;
mod routes_api_token;
mod routes_create;
mod routes_dev;
mod routes_login;
mod routes_pull;
mod routes_setup;
mod routes_ui;

pub use error::*;
pub use routes_api_token::*;
pub use routes_create::*;
pub use routes_dev::*;
pub use routes_login::*;
pub use routes_pull::*;
pub use routes_setup::*;
pub use routes_ui::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
