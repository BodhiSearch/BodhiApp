#[macro_export]
macro_rules! asref_impl {
  ($trait:path, $t:ty) => {
    impl ::core::convert::AsRef<dyn $trait> for $t {
      fn as_ref(&self) -> &(dyn $trait + 'static) {
        self
      }
    }

    impl ::core::convert::AsRef<dyn $trait> for ::std::sync::Arc<$t> {
      fn as_ref(&self) -> &(dyn $trait + 'static) {
        &**self
      }
    }
  };
}
