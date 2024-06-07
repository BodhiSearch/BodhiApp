pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
  t == &T::default()
}
