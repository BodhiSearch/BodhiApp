#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod bodhi_err_exts;
mod bodhi_server_ctx;
mod error;
mod objs;

pub use bodhi_server_ctx::*;
pub use error::*;
pub use objs::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
