use super::*;
use rstest::rstest;

#[rstest]
#[case("http://example.com")]
#[case("https://example.com")]
#[case("https://mcp.asana.com/mcp")]
#[case("http://api.openai.com/v1")]
#[case("https://huggingface.co/models")]
#[case("http://203.0.113.50:8080")]
fn test_allows_valid_external_urls(#[case] url: &str) {
  let result = validate_outbound_url(url, false);
  assert!(result.is_ok(), "Expected OK for {url}, got: {result:?}");
}

#[rstest]
#[case("javascript:alert(1)", "javascript")]
#[case("file:///etc/passwd", "file")]
#[case("data:text/html,<script>alert(1)</script>", "data")]
#[case("gopher://evil.com", "gopher")]
#[case("ftp://evil.com/file", "ftp")]
#[case("vbscript:MsgBox(1)", "vbscript")]
fn test_rejects_disallowed_schemes(#[case] url: &str, #[case] expected_scheme: &str) {
  let result = validate_outbound_url(url, false);
  assert!(result.is_err(), "Expected error for {url}");
  match result.unwrap_err() {
    UrlValidationError::DisallowedScheme { scheme } => {
      assert_eq!(scheme, expected_scheme);
    }
    other => panic!("Expected DisallowedScheme, got: {other:?}"),
  }
}

#[rstest]
#[case("http://127.0.0.1")]
#[case("http://127.0.0.1:8080")]
#[case("http://127.0.0.255")]
#[case("http://10.0.0.1")]
#[case("http://10.255.255.255")]
#[case("http://172.16.0.1")]
#[case("http://172.31.255.255")]
#[case("http://192.168.0.1")]
#[case("http://192.168.1.100")]
#[case("http://169.254.169.254")]
#[case("http://169.254.0.1")]
#[case("http://0.0.0.0")]
fn test_rejects_private_ipv4_addresses(#[case] url: &str) {
  let result = validate_outbound_url(url, false);
  assert!(result.is_err(), "Expected error for {url}");
  assert!(
    matches!(
      result.unwrap_err(),
      UrlValidationError::PrivateAddress { .. }
    ),
    "Expected PrivateAddress for {url}"
  );
}

#[rstest]
#[case("http://[::1]")]
#[case("http://[::1]:8080")]
#[case("http://[fc00::1]")]
#[case("http://[fd00::1]")]
#[case("http://[fe80::1]")]
fn test_rejects_private_ipv6_addresses(#[case] url: &str) {
  let result = validate_outbound_url(url, false);
  assert!(result.is_err(), "Expected error for {url}");
  assert!(
    matches!(
      result.unwrap_err(),
      UrlValidationError::PrivateAddress { .. }
    ),
    "Expected PrivateAddress for {url}"
  );
}

#[rstest]
#[case("http://localhost")]
#[case("http://localhost:1135")]
#[case("http://LOCALHOST")]
#[case("http://host.docker.internal")]
#[case("http://host.docker.internal:1135")]
#[case("http://HOST.DOCKER.INTERNAL")]
fn test_rejects_blocked_hostnames(#[case] url: &str) {
  let result = validate_outbound_url(url, false);
  assert!(result.is_err(), "Expected error for {url}");
  assert!(
    matches!(
      result.unwrap_err(),
      UrlValidationError::PrivateAddress { .. }
    ),
    "Expected PrivateAddress for {url}"
  );
}

#[rstest]
#[case("not-a-url")]
#[case("")]
#[case("://missing-scheme")]
fn test_rejects_invalid_urls(#[case] url: &str) {
  let result = validate_outbound_url(url, false);
  assert!(result.is_err(), "Expected error for {url}");
  assert!(
    matches!(result.unwrap_err(), UrlValidationError::InvalidUrl { .. }),
    "Expected InvalidUrl for {url}"
  );
}

#[rstest]
fn test_allows_public_ipv4() {
  assert!(validate_outbound_url("http://8.8.8.8", false).is_ok());
  assert!(validate_outbound_url("http://1.1.1.1", false).is_ok());
  assert!(validate_outbound_url("http://203.0.113.50", false).is_ok());
}

#[rstest]
fn test_rejects_private_with_path() {
  assert!(validate_outbound_url("http://127.0.0.1/dev/db-reset", false).is_err());
  assert!(validate_outbound_url("http://169.254.169.254/latest/meta-data/", false).is_err());
}

#[rstest]
fn test_allows_external_with_port_and_path() {
  assert!(validate_outbound_url("http://example.com:8080/api/v1", false).is_ok());
  assert!(validate_outbound_url("https://mcp.example.com:443/oauth/token", false).is_ok());
}

#[rstest]
fn test_172_non_private_range_allowed() {
  // 172.15.x.x is NOT private (only 172.16-31.x.x is private)
  assert!(validate_outbound_url("http://172.15.0.1", false).is_ok());
  assert!(validate_outbound_url("http://172.32.0.1", false).is_ok());
}

// ============================================================================
// allow_private_ips=true tests (for AI API / MCP local connections)
// ============================================================================

#[rstest]
#[case("http://127.0.0.1:11434")]
#[case("http://localhost:11434")]
#[case("http://localhost:8080")]
#[case("http://10.0.0.1:8080")]
#[case("http://192.168.1.100:3000")]
#[case("http://host.docker.internal:1135")]
#[case("http://[::1]:8080")]
fn test_allow_private_ips_permits_local_urls(#[case] url: &str) {
  let result = validate_outbound_url(url, true);
  assert!(
    result.is_ok(),
    "Expected OK for {url} with allow_private_ips=true, got: {result:?}"
  );
}

#[rstest]
#[case("javascript:alert(1)", "javascript")]
#[case("file:///etc/passwd", "file")]
#[case("data:text/html,<script>alert(1)</script>", "data")]
fn test_allow_private_ips_still_rejects_bad_schemes(#[case] url: &str, #[case] _scheme: &str) {
  let result = validate_outbound_url(url, true);
  assert!(
    result.is_err(),
    "Expected error for {url} even with allow_private_ips=true"
  );
  assert!(
    matches!(
      result.unwrap_err(),
      UrlValidationError::DisallowedScheme { .. }
    ),
    "Expected DisallowedScheme for {url}"
  );
}

// ============================================================================
// validate_http_url() tests (validator-crate-compatible wrapper)
// ============================================================================

#[rstest]
#[case("http://example.com")]
#[case("https://example.com")]
#[case("http://localhost:11434")]
#[case("http://127.0.0.1:8080")]
#[case("https://api.openai.com/v1")]
fn test_validate_http_url_accepts_valid(#[case] url: &str) {
  assert!(validate_http_url(url).is_ok(), "Expected OK for {url}");
}

#[rstest]
#[case("javascript:alert(1)")]
#[case("file:///etc/passwd")]
#[case("data:text/html,<script>alert(1)</script>")]
#[case("ftp://evil.com/file")]
#[case("gopher://evil.com")]
fn test_validate_http_url_rejects_bad_schemes(#[case] url: &str) {
  assert!(validate_http_url(url).is_err(), "Expected error for {url}");
}

#[rstest]
#[case("not-a-url")]
#[case("")]
fn test_validate_http_url_rejects_invalid_urls(#[case] url: &str) {
  assert!(validate_http_url(url).is_err(), "Expected error for {url}");
}
