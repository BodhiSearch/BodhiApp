mod build_envs;
mod error;
mod server;

pub use build_envs::*;
pub use error::*;
pub use server::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
