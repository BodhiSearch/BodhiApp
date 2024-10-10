#[cfg(feature = "test-utils")]
pub mod test_utils;
#[cfg(all(not(feature = "test-utils"), test))]
pub mod test_utils;

mod alias;
mod app_error;
mod chat_template;
mod envs;
mod error;
mod gpt_params;
mod hub_file;
mod l10n;
mod oai;
mod remote_file;
mod repo;
mod utils;

pub use alias::*;
pub use app_error::*;
pub use chat_template::*;
pub use envs::*;
pub use error::*;
pub use gpt_params::*;
pub use hub_file::*;
pub use l10n::*;
pub use oai::*;
pub use remote_file::*;
pub use repo::*;
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
