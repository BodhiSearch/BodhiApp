#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod error;
mod listener_keep_alive;
mod listener_variant;
mod serve;
mod server;
mod shutdown;

pub use error::*;
pub use listener_keep_alive::*;
pub use listener_variant::*;
pub use serve::*;
pub use server::*;
pub use shutdown::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
