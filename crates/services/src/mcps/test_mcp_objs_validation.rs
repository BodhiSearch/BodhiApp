use crate::mcps::{
  validate_mcp_auth_config_name, validate_mcp_description, validate_mcp_instance_name,
  validate_mcp_server_description, validate_mcp_server_name, validate_mcp_server_url,
  validate_mcp_slug, McpInstanceNameError, MAX_MCP_AUTH_CONFIG_NAME_LEN, MAX_MCP_DESCRIPTION_LEN,
  MAX_MCP_INSTANCE_NAME_LEN, MAX_MCP_SERVER_NAME_LEN, MAX_MCP_SERVER_URL_LEN, MAX_MCP_SLUG_LEN,
};
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[case::simple("my-mcp")]
#[case::mixed_case("MyMcp123")]
#[case::single_char("a")]
#[case::with_digits("deepwiki-1")]
fn test_validate_mcp_slug_accepts_valid(#[case] slug: &str) {
  assert!(validate_mcp_slug(slug).is_ok());
}

#[test]
fn test_validate_mcp_slug_accepts_max_length() {
  let max_slug = "a".repeat(MAX_MCP_SLUG_LEN);
  assert!(validate_mcp_slug(&max_slug).is_ok());
}

#[test]
fn test_validate_mcp_slug_rejects_empty() {
  let result = validate_mcp_slug("");
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_validate_mcp_slug_rejects_too_long() {
  let long_slug = "a".repeat(MAX_MCP_SLUG_LEN + 1);
  let result = validate_mcp_slug(&long_slug);
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("cannot exceed"));
}

#[rstest]
#[case::underscore("my_mcp")]
#[case::space("my mcp")]
#[case::dot("my.mcp")]
#[case::at("my@mcp")]
fn test_validate_mcp_slug_rejects_invalid_chars(#[case] slug: &str) {
  assert!(validate_mcp_slug(slug).is_err());
}

#[rstest]
#[case::empty("")]
#[case::short("A short description")]
#[case::special_chars("A description with special chars: @#$%")]
fn test_validate_mcp_description_accepts_valid(#[case] desc: &str) {
  assert!(validate_mcp_description(desc).is_ok());
}

#[test]
fn test_validate_mcp_description_accepts_max_length() {
  let max_desc = "a".repeat(MAX_MCP_DESCRIPTION_LEN);
  assert!(validate_mcp_description(&max_desc).is_ok());
}

#[test]
fn test_validate_mcp_description_rejects_too_long() {
  let long_desc = "a".repeat(MAX_MCP_DESCRIPTION_LEN + 1);
  let result = validate_mcp_description(&long_desc);
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("cannot exceed"));
}

#[rstest]
#[case::display_name("DeepWiki MCP")]
#[case::single_char("a")]
fn test_validate_mcp_server_name_accepts_valid(#[case] name: &str) {
  assert!(validate_mcp_server_name(name).is_ok());
}

#[test]
fn test_validate_mcp_server_name_accepts_max_length() {
  assert!(validate_mcp_server_name(&"a".repeat(MAX_MCP_SERVER_NAME_LEN)).is_ok());
}

#[test]
fn test_validate_mcp_server_name_rejects_empty() {
  assert!(validate_mcp_server_name("").is_err());
}

#[test]
fn test_validate_mcp_server_name_rejects_too_long() {
  assert!(validate_mcp_server_name(&"a".repeat(MAX_MCP_SERVER_NAME_LEN + 1)).is_err());
}

#[rstest]
#[case::https("https://mcp.deepwiki.com/mcp")]
#[case::localhost("http://localhost:8080/mcp")]
fn test_validate_mcp_server_url_accepts_valid(#[case] url: &str) {
  assert!(validate_mcp_server_url(url).is_ok());
}

#[test]
fn test_validate_mcp_server_url_rejects_empty() {
  assert!(validate_mcp_server_url("").is_err());
}

#[rstest]
#[case::not_a_url("not-a-url")]
#[case::missing_colon("ftp missing colon")]
fn test_validate_mcp_server_url_rejects_invalid(#[case] url: &str) {
  assert!(validate_mcp_server_url(url).is_err());
}

#[test]
fn test_validate_mcp_server_url_rejects_too_long() {
  let long_url = format!("https://example.com/{}", "a".repeat(MAX_MCP_SERVER_URL_LEN));
  assert!(validate_mcp_server_url(&long_url).is_err());
}

#[rstest]
#[case::empty("")]
#[case::short("A test server")]
fn test_validate_mcp_server_description_accepts_valid(#[case] desc: &str) {
  assert!(validate_mcp_server_description(desc).is_ok());
}

#[test]
fn test_validate_mcp_server_description_accepts_max_length() {
  assert!(validate_mcp_server_description(&"a".repeat(MAX_MCP_DESCRIPTION_LEN)).is_ok());
}

#[test]
fn test_validate_mcp_server_description_rejects_too_long() {
  assert!(validate_mcp_server_description(&"a".repeat(MAX_MCP_DESCRIPTION_LEN + 1)).is_err());
}

#[rstest]
#[case::simple("Header")]
#[case::with_spaces("My OAuth Config")]
fn test_validate_mcp_auth_config_name_accepts_valid(#[case] name: &str) {
  assert!(validate_mcp_auth_config_name(name).is_ok());
}

#[test]
fn test_validate_mcp_auth_config_name_accepts_max_length() {
  assert!(validate_mcp_auth_config_name(&"a".repeat(MAX_MCP_AUTH_CONFIG_NAME_LEN)).is_ok());
}

#[test]
fn test_validate_mcp_auth_config_name_rejects_empty() {
  assert!(validate_mcp_auth_config_name("").is_err());
}

#[test]
fn test_validate_mcp_auth_config_name_rejects_too_long() {
  assert!(validate_mcp_auth_config_name(&"a".repeat(MAX_MCP_AUTH_CONFIG_NAME_LEN + 1)).is_err());
}

// ============================================================================
// validate_mcp_instance_name tests
// ============================================================================

#[rstest]
#[case::simple("My MCP")]
#[case::single_char("a")]
#[case::with_spaces("My MCP Instance")]
#[case::special_chars("MCP #1")]
fn test_validate_mcp_instance_name_accepts_valid(#[case] name: &str) {
  assert!(validate_mcp_instance_name(name).is_ok());
}

#[test]
fn test_validate_mcp_instance_name_accepts_max_length() {
  assert!(validate_mcp_instance_name(&"a".repeat(MAX_MCP_INSTANCE_NAME_LEN)).is_ok());
}

#[test]
fn test_validate_mcp_instance_name_rejects_empty() {
  let result = validate_mcp_instance_name("");
  assert_eq!(McpInstanceNameError::Empty, result.unwrap_err());
}

#[test]
fn test_validate_mcp_instance_name_rejects_too_long() {
  let long_name = "a".repeat(MAX_MCP_INSTANCE_NAME_LEN + 1);
  let result = validate_mcp_instance_name(&long_name);
  assert_eq!(
    McpInstanceNameError::TooLong {
      name: long_name.clone(),
      max_len: MAX_MCP_INSTANCE_NAME_LEN,
    },
    result.unwrap_err()
  );
}
