#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod alias;
mod api_model_alias;
mod api_tags;
mod envs;
mod error;
pub mod gguf;
mod model_alias;
mod model_metadata;
mod user_alias;

mod hub_file;
pub mod log;
mod oai;
mod remote_file;
mod repo;
mod resource_role;
mod resource_scope;
mod token_scope;
mod toolsets;
mod user;
mod user_scope;
mod utils;

pub use alias::*;
pub use api_model_alias::*;
pub use api_tags::*;
pub use envs::*;
pub use error::*;
pub use model_alias::*;
pub use model_metadata::*;
pub use user_alias::*;

pub use hub_file::*;
pub use oai::*;
pub use remote_file::*;
pub use repo::*;
pub use resource_role::*;
pub use resource_scope::*;
pub use token_scope::*;
pub use toolsets::*;
pub use user::*;
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
