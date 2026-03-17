use crate::mcps::{
  McpAuthType, McpRequest, McpServerRequest, MAX_MCP_DESCRIPTION_LEN, MAX_MCP_INSTANCE_NAME_LEN,
  MAX_MCP_SERVER_NAME_LEN, MAX_MCP_SERVER_URL_LEN, MAX_MCP_SLUG_LEN,
};
use rstest::rstest;
use validator::Validate;

// ============================================================================
// Helper constructors
// ============================================================================

fn mcp_request_with_name(name: &str) -> McpRequest {
  McpRequest {
    name: name.to_string(),
    slug: "valid-slug".to_string(),
    mcp_server_id: Some("server-1".to_string()),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: McpAuthType::default(),
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  }
}

fn mcp_request_with_slug(slug: &str) -> McpRequest {
  McpRequest {
    name: "Valid Name".to_string(),
    slug: slug.to_string(),
    mcp_server_id: Some("server-1".to_string()),
    description: None,
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: McpAuthType::default(),
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  }
}

fn mcp_request_with_description(desc: Option<&str>) -> McpRequest {
  McpRequest {
    name: "Valid Name".to_string(),
    slug: "valid-slug".to_string(),
    mcp_server_id: Some("server-1".to_string()),
    description: desc.map(|s| s.to_string()),
    enabled: true,
    tools_cache: None,
    tools_filter: None,
    auth_type: McpAuthType::default(),
    auth_config_id: None,
    credentials: None,
    oauth_token_id: None,
  }
}

fn server_request_with_url(url: &str) -> McpServerRequest {
  McpServerRequest {
    url: url.to_string(),
    name: "Valid Name".to_string(),
    description: None,
    enabled: true,
    auth_config: None,
  }
}

fn server_request_with_name(name: &str) -> McpServerRequest {
  McpServerRequest {
    url: "https://mcp.example.com/mcp".to_string(),
    name: name.to_string(),
    description: None,
    enabled: true,
    auth_config: None,
  }
}

fn server_request_with_description(desc: Option<&str>) -> McpServerRequest {
  McpServerRequest {
    url: "https://mcp.example.com/mcp".to_string(),
    name: "Valid Name".to_string(),
    description: desc.map(|s| s.to_string()),
    enabled: true,
    auth_config: None,
  }
}

// ============================================================================
// McpRequest slug validation
// ============================================================================

#[rstest]
#[case::simple("my-mcp")]
#[case::mixed_case("MyMcp123")]
#[case::single_char("a")]
#[case::with_digits("deepwiki-1")]
#[case::max_length(&"a".repeat(MAX_MCP_SLUG_LEN))]
fn test_mcp_request_valid_slug(#[case] slug: &str) {
  assert!(mcp_request_with_slug(slug).validate().is_ok());
}

#[rstest]
#[case::empty("")]
#[case::too_long(&"a".repeat(MAX_MCP_SLUG_LEN + 1))]
#[case::underscore("my_mcp")]
#[case::space("my mcp")]
#[case::dot("my.mcp")]
#[case::at("my@mcp")]
fn test_mcp_request_invalid_slug(#[case] slug: &str) {
  assert!(mcp_request_with_slug(slug).validate().is_err());
}

// ============================================================================
// McpRequest name validation
// ============================================================================

#[rstest]
#[case::simple("My MCP")]
#[case::single_char("a")]
#[case::max_length(&"a".repeat(MAX_MCP_INSTANCE_NAME_LEN))]
fn test_mcp_request_valid_name(#[case] name: &str) {
  assert!(mcp_request_with_name(name).validate().is_ok());
}

#[rstest]
#[case::empty("")]
#[case::too_long(&"a".repeat(MAX_MCP_INSTANCE_NAME_LEN + 1))]
fn test_mcp_request_invalid_name(#[case] name: &str) {
  assert!(mcp_request_with_name(name).validate().is_err());
}

// ============================================================================
// McpRequest description validation
// ============================================================================

#[rstest]
#[case::none(None)]
#[case::short(Some("A short description"))]
#[case::max_length(Some(&*"a".repeat(MAX_MCP_DESCRIPTION_LEN)))]
fn test_mcp_request_valid_description(#[case] desc: Option<&str>) {
  assert!(mcp_request_with_description(desc).validate().is_ok());
}

#[test]
fn test_mcp_request_rejects_too_long_description() {
  let long_desc = "a".repeat(MAX_MCP_DESCRIPTION_LEN + 1);
  assert!(mcp_request_with_description(Some(&long_desc))
    .validate()
    .is_err());
}

// ============================================================================
// McpServerRequest URL validation
// ============================================================================

#[rstest]
#[case::https("https://mcp.deepwiki.com/mcp")]
#[case::localhost("http://localhost:8080/mcp")]
fn test_server_request_valid_url(#[case] url: &str) {
  assert!(server_request_with_url(url).validate().is_ok());
}

#[rstest]
#[case::empty("")]
#[case::not_a_url("not-a-url")]
#[case::missing_colon("ftp missing colon")]
fn test_server_request_invalid_url(#[case] url: &str) {
  assert!(server_request_with_url(url).validate().is_err());
}

#[test]
fn test_server_request_rejects_too_long_url() {
  let long_url = format!("https://example.com/{}", "a".repeat(MAX_MCP_SERVER_URL_LEN));
  assert!(server_request_with_url(&long_url).validate().is_err());
}

// ============================================================================
// McpServerRequest name validation
// ============================================================================

#[rstest]
#[case::display_name("DeepWiki MCP")]
#[case::single_char("a")]
#[case::max_length(&"a".repeat(MAX_MCP_SERVER_NAME_LEN))]
fn test_server_request_valid_name(#[case] name: &str) {
  assert!(server_request_with_name(name).validate().is_ok());
}

#[rstest]
#[case::empty("")]
#[case::too_long(&"a".repeat(MAX_MCP_SERVER_NAME_LEN + 1))]
fn test_server_request_invalid_name(#[case] name: &str) {
  assert!(server_request_with_name(name).validate().is_err());
}

// ============================================================================
// McpServerRequest description validation
// ============================================================================

#[rstest]
#[case::none(None)]
#[case::short(Some("A test server"))]
#[case::max_length(Some(&*"a".repeat(MAX_MCP_DESCRIPTION_LEN)))]
fn test_server_request_valid_description(#[case] desc: Option<&str>) {
  assert!(server_request_with_description(desc).validate().is_ok());
}

#[test]
fn test_server_request_rejects_too_long_description() {
  let long_desc = "a".repeat(MAX_MCP_DESCRIPTION_LEN + 1);
  assert!(server_request_with_description(Some(&long_desc))
    .validate()
    .is_err());
}
