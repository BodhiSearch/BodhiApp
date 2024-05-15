pub(crate) fn bool(key: &str) -> bool {
  std::env::var(key)
    .unwrap_or(String::from("false"))
    .parse::<bool>()
    .unwrap_or_default()
}

#[cfg(test)]
mod test {
  use super::*;
  use rstest::rstest;
  use serial_test::serial;
  static TEST_BOOL: &str = "TEST_BOOL";

  #[rstest]
  #[case("false")]
  #[serial(env_utils)]
  fn test_env_utils_bool_false(#[case] input: &str) -> anyhow::Result<()> {
    std::env::set_var(TEST_BOOL, input);
    assert!(!bool(TEST_BOOL));
    Ok(())
  }

  #[rstest]
  #[case("true")]
  #[serial(env_utils)]
  fn test_env_utils_bool_true(#[case] input: &str) -> anyhow::Result<()> {
    std::env::set_var(TEST_BOOL, input);
    assert!(bool(TEST_BOOL));
    Ok(())
  }

  #[rstest]
  #[case("FALSE")]
  #[case("False")]
  #[case("0")]
  #[case("no")]
  #[case("NO")]
  #[case("No")]
  #[case("TRUE")]
  #[case("True")]
  #[case("1")]
  #[case("yes")]
  #[case("YES")]
  #[case("Yes")]
  #[serial(env_utils)]
  fn test_env_utils_bool_default(#[case] input: &str) -> anyhow::Result<()> {
    std::env::set_var(TEST_BOOL, input);
    assert!(!bool(TEST_BOOL));
    Ok(())
  }

  #[test]
  #[serial(env_utils)]
  fn test_env_utils_bool_not_set() -> anyhow::Result<()> {
    std::env::remove_var(TEST_BOOL);
    assert!(!bool(TEST_BOOL));
    Ok(())
  }
}
