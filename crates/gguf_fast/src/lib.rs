mod constants;
mod error;
mod reader;

pub use constants::*;
pub use error::*;
pub use reader::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
