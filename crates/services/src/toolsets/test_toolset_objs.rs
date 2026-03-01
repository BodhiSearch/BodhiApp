use super::{
  validate_toolset_description, validate_toolset_slug, MAX_TOOLSET_DESCRIPTION_LEN,
  MAX_TOOLSET_SLUG_LEN,
};

#[test]
fn test_validate_toolset_slug_accepts_valid_slugs() {
  assert!(validate_toolset_slug("my-toolset").is_ok());
  assert!(validate_toolset_slug("MyToolset123").is_ok());
  assert!(validate_toolset_slug("a").is_ok());
  assert!(validate_toolset_slug("toolset-1").is_ok());
}

#[test]
fn test_validate_toolset_slug_rejects_empty() {
  let result = validate_toolset_slug("");
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_validate_toolset_slug_rejects_too_long() {
  let long_slug = "a".repeat(MAX_TOOLSET_SLUG_LEN + 1);
  let result = validate_toolset_slug(&long_slug);
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("cannot exceed"));
}

#[test]
fn test_validate_toolset_slug_rejects_invalid_characters() {
  assert!(validate_toolset_slug("my_toolset").is_err());
  assert!(validate_toolset_slug("my toolset").is_err());
  assert!(validate_toolset_slug("my.toolset").is_err());
  assert!(validate_toolset_slug("my@toolset").is_err());
}

#[test]
fn test_validate_toolset_slug_accepts_max_length() {
  let max_slug = "a".repeat(MAX_TOOLSET_SLUG_LEN);
  assert!(validate_toolset_slug(&max_slug).is_ok());
}

#[test]
fn test_validate_toolset_description_accepts_valid_descriptions() {
  assert!(validate_toolset_description("").is_ok());
  assert!(validate_toolset_description("A short description").is_ok());
  assert!(validate_toolset_description("A description with special chars: @#$%").is_ok());
}

#[test]
fn test_validate_toolset_description_rejects_too_long() {
  let long_desc = "a".repeat(MAX_TOOLSET_DESCRIPTION_LEN + 1);
  let result = validate_toolset_description(&long_desc);
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("cannot exceed"));
}

#[test]
fn test_validate_toolset_description_accepts_max_length() {
  let max_desc = "a".repeat(MAX_TOOLSET_DESCRIPTION_LEN);
  assert!(validate_toolset_description(&max_desc).is_ok());
}
