#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod user_alias;
mod api_model_alias;
mod api_tags;
mod envs;
mod error;
pub mod gguf;
mod alias;

mod hub_file;
mod localization_service;
pub mod log;
mod oai;
mod remote_file;
mod repo;
mod resource_scope;
mod role;
mod token_scope;
mod user_scope;
mod utils;

pub use user_alias::*;
pub use api_model_alias::*;
pub use api_tags::*;
pub use envs::*;
pub use error::*;
pub use alias::*;

pub use hub_file::*;
pub use localization_service::*;
pub use oai::*;
pub use remote_file::*;
pub use repo::*;
pub use resource_scope::*;
pub use role::*;
pub use token_scope::*;
pub use user_scope::*;
pub use utils::*;

#[macro_export]
macro_rules! impl_error_from {
  ($source_error:ty, $target_error:ident :: $variant:ident, $intermediate_error:ty) => {
    impl From<$source_error> for $target_error {
      fn from(err: $source_error) -> Self {
        $target_error::$variant(<$intermediate_error>::from(err))
      }
    }
  };
}

pub mod l10n {
  use include_dir::Dir;

  pub const L10N_RESOURCES: &Dir = &include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/resources");
}
