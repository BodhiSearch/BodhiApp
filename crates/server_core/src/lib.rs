#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod direct_sse;
mod error;
mod fwd_sse;
mod objs;
mod router_state;
mod shared_rw;

pub use direct_sse::*;
pub use error::*;
pub use fwd_sse::*;
pub use objs::*;
pub use router_state::*;
pub use shared_rw::*;

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
