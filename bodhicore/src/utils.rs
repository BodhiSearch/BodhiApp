use regex::Regex;

pub(crate) fn to_safe_filename(input: &str) -> String {
  let illegal_chars = Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
  let mut sanitized = illegal_chars.replace_all(input, "--").to_string();
  sanitized = sanitized
    .chars()
    .filter(|c| !c.is_control() && !c.is_whitespace())
    .collect();
  if sanitized.len() > 255 {
    sanitized.truncate(255);
  }
  sanitized
}
