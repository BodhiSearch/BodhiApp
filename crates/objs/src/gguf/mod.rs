mod constants;
mod error;
mod metadata;

pub use constants::*;
pub use error::*;
pub use metadata::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir =
    &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/gguf/resources");
}
