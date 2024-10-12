#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod routes_chat;
mod routes_models;
mod routes_oai_models;
mod routes_ollama;

pub use routes_chat::*;
pub use routes_models::*;
pub use routes_oai_models::*;
pub use routes_ollama::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
