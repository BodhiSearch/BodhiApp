pub mod capabilities;
mod constants;
mod error;
mod metadata;
mod value;

pub use capabilities::*;
pub use constants::*;
pub use error::*;
pub use metadata::*;
pub use value::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir =
    &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/gguf/resources");
}
