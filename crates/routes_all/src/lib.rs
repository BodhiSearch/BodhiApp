mod routes;
mod routes_proxy;

pub use routes::*;
pub use routes_proxy::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
