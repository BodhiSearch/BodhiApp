use once_cell::sync::Lazy;
use regex::Regex;

pub(crate) fn is_default<T: Default + PartialEq>(t: &T) -> bool {
  t == &T::default()
}

pub static ILLEGAL_CHARS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[<>:"/\\|?*]"#).unwrap());

pub(crate) fn to_safe_filename(input: &str) -> String {
  let mut sanitized = ILLEGAL_CHARS.replace_all(input, "--").to_string();
  sanitized = sanitized
    .chars()
    .filter(|c| !c.is_control() && !c.is_whitespace())
    .collect();
  if sanitized.len() > 255 {
    sanitized.truncate(255);
  }
  sanitized
}

#[cfg(test)]
mod tests {
  use super::to_safe_filename;
  use rstest::rstest;

  #[rstest]
  #[case("normal_filename.txt", "normal_filename.txt")]
  #[case("file with spaces.txt", "filewithspaces.txt")]
  #[case("file:with:colons.txt", "file--with--colons.txt")]
  #[case("file<with>brackets.txt", "file--with--brackets.txt")]
  #[case("file\"with\"quotes.txt", "file--with--quotes.txt")]
  #[case("file\\with/slashes.txt", "file--with--slashes.txt")]
  #[case("file|with|pipes.txt", "file--with--pipes.txt")]
  #[case("file?with?questions.txt", "file--with--questions.txt")]
  #[case("file*with*stars.txt", "file--with--stars.txt")]
  #[case("file\nwith\nnewlines.txt", "filewithnewlines.txt")]
  #[case("file\twith\ttabs.txt", "filewithtabs.txt")]
  fn test_to_safe_filename(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(expected, to_safe_filename(input));
  }
}
