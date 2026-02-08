// -- Test utilities
#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

// -- Error handling
mod error;

// -- Model & alias system
mod alias;
mod api_model_alias;
mod model_alias;
mod model_metadata;
mod user_alias;
mod hub_file;
pub mod gguf;
mod remote_file;
mod repo;

// -- OpenAI API compatibility
mod oai;

// -- Authentication & access control
mod resource_role;
mod resource_scope;
mod token_scope;
mod user;
mod user_scope;

// -- Toolsets
mod toolsets;

// -- Configuration & environment
mod envs;
pub mod log;

// -- API organization
mod api_tags;

// -- Utilities
mod utils;

// -- Re-exports: error handling
pub use error::*;

// -- Re-exports: model & alias system
pub use alias::*;
pub use api_model_alias::*;
pub use model_alias::*;
pub use model_metadata::*;
pub use user_alias::*;
pub use hub_file::*;
pub use remote_file::*;
pub use repo::*;

// -- Re-exports: OpenAI API compatibility
pub use oai::*;

// -- Re-exports: authentication & access control
pub use resource_role::*;
pub use resource_scope::*;
pub use token_scope::*;
pub use user::*;
pub use user_scope::*;

// -- Re-exports: toolsets
pub use toolsets::*;

// -- Re-exports: configuration & environment
pub use envs::*;

// -- Re-exports: API organization
pub use api_tags::*;

// -- Re-exports: utilities
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
