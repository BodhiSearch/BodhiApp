#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod bindings;
mod direct_sse;
mod error;
mod fwd_sse;
pub mod obj_exts;
mod objs;
mod router_state;
mod shared_rw;
mod tokenizer_config;

pub use bindings::*;
pub use direct_sse::*;
pub use error::*;
pub use fwd_sse::*;
pub use objs::*;
pub use router_state::*;
pub use shared_rw::*;
pub use tokenizer_config::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
