mod app_error;
mod entity_error;
mod error_type;
mod io_error;
mod rwlock_error;

#[cfg(test)]
#[path = "test_impl_error_from.rs"]
mod test_impl_error_from;

pub use app_error::*;
pub use entity_error::*;
pub use error_type::*;
pub use io_error::*;
pub use rwlock_error::*;

pub use errmeta_derive::ErrorMeta;

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
