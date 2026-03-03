use super::{ToolsetRequest, MAX_TOOLSET_DESCRIPTION_LEN, MAX_TOOLSET_SLUG_LEN};
use rstest::rstest;
use validator::Validate;

fn request_with_slug(slug: &str) -> ToolsetRequest {
  ToolsetRequest {
    slug: slug.to_string(),
    ..Default::default()
  }
}

fn request_with_description(desc: Option<&str>) -> ToolsetRequest {
  ToolsetRequest {
    slug: "valid-slug".to_string(),
    description: desc.map(|s| s.to_string()),
    ..Default::default()
  }
}

#[rstest]
#[case::simple("my-toolset")]
#[case::mixed_case("MyToolset123")]
#[case::single_char("a")]
#[case::with_digits("toolset-1")]
#[case::max_length(&"a".repeat(MAX_TOOLSET_SLUG_LEN))]
fn test_toolset_request_valid_slug(#[case] slug: &str) {
  assert!(request_with_slug(slug).validate().is_ok());
}

#[rstest]
#[case::empty("")]
#[case::too_long(&"a".repeat(MAX_TOOLSET_SLUG_LEN + 1))]
#[case::underscore("my_toolset")]
#[case::space("my toolset")]
#[case::dot("my.toolset")]
#[case::at("my@toolset")]
fn test_toolset_request_invalid_slug(#[case] slug: &str) {
  assert!(request_with_slug(slug).validate().is_err());
}

#[rstest]
#[case::none(None)]
#[case::short(Some("A short description"))]
#[case::special_chars(Some("A description with special chars: @#$%"))]
#[case::max_length(Some(&*"a".repeat(MAX_TOOLSET_DESCRIPTION_LEN)))]
fn test_toolset_request_valid_description(#[case] desc: Option<&str>) {
  assert!(request_with_description(desc).validate().is_ok());
}

#[test]
fn test_toolset_request_rejects_too_long_description() {
  let long_desc = "a".repeat(MAX_TOOLSET_DESCRIPTION_LEN + 1);
  assert!(request_with_description(Some(&long_desc))
    .validate()
    .is_err());
}
