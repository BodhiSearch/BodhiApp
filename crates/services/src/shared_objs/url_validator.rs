use std::net::IpAddr;

use errmeta::{AppError, ErrorType};

/// Error returned when a URL fails outbound validation (SSRF protection).
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum UrlValidationError {
  #[error("URL scheme '{scheme}' is not allowed. Only 'http' and 'https' are permitted.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  DisallowedScheme { scheme: String },

  #[error("Requests to private/internal network address '{host}' are not allowed.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  PrivateAddress { host: String },

  #[error("Invalid URL: {reason}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidUrl { reason: String },
}

/// Validates that a URL is safe for outbound requests.
///
/// Always enforced:
/// 1. Scheme must be `http` or `https` (blocks `javascript:`, `file:`, `data:`, etc.)
///
/// When `allow_private_ips` is false (stricter SSRF protection):
/// 2. Host must not be a private/loopback/link-local IP address
/// 3. Hostname must not be `localhost` or `host.docker.internal`
///
/// Set `allow_private_ips` to true for services that legitimately connect to
/// local servers (e.g., Ollama for AI inference, local MCP servers).
pub fn validate_outbound_url(
  url_str: &str,
  allow_private_ips: bool,
) -> Result<url::Url, UrlValidationError> {
  let parsed = url::Url::parse(url_str).map_err(|e| UrlValidationError::InvalidUrl {
    reason: e.to_string(),
  })?;

  // 1. Scheme allowlist (always enforced)
  let scheme = parsed.scheme();
  if scheme != "http" && scheme != "https" {
    return Err(UrlValidationError::DisallowedScheme {
      scheme: scheme.to_string(),
    });
  }

  // 2. Host validation
  let host_str = parsed
    .host_str()
    .ok_or_else(|| UrlValidationError::InvalidUrl {
      reason: "URL has no host".to_string(),
    })?;

  // Private IP/hostname checks (skipped when allow_private_ips is true)
  if !allow_private_ips {
    // 3. Hostname blocklist
    let host_lower = host_str.to_lowercase();
    if host_lower == "localhost" || host_lower == "host.docker.internal" {
      return Err(UrlValidationError::PrivateAddress {
        host: host_str.to_string(),
      });
    }

    // 4. IP address blocklist
    if let Ok(ip) = host_str.parse::<IpAddr>() {
      if is_private_ip(&ip) {
        return Err(UrlValidationError::PrivateAddress {
          host: host_str.to_string(),
        });
      }
    }

    // Also check bracketed IPv6 (e.g. [::1])
    let trimmed = host_str.trim_start_matches('[').trim_end_matches(']');
    if trimmed != host_str {
      if let Ok(ip) = trimmed.parse::<IpAddr>() {
        if is_private_ip(&ip) {
          return Err(UrlValidationError::PrivateAddress {
            host: host_str.to_string(),
          });
        }
      }
    }
  }

  Ok(parsed)
}

fn is_private_ip(ip: &IpAddr) -> bool {
  match ip {
    IpAddr::V4(v4) => {
      v4.is_loopback()                         // 127.0.0.0/8
        || v4.is_private()                     // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
        || v4.is_link_local()                  // 169.254.0.0/16 (AWS IMDS)
        || v4.is_unspecified()                 // 0.0.0.0
        || v4.octets()[0] == 0 // 0.0.0.0/8
    }
    IpAddr::V6(v6) => {
      v6.is_loopback()                         // ::1
        || v6.is_unspecified()                 // ::
        || is_ipv6_unique_local(v6)            // fc00::/7
        || is_ipv6_link_local(v6) // fe80::/10
    }
  }
}

fn is_ipv6_unique_local(v6: &std::net::Ipv6Addr) -> bool {
  // fc00::/7 — first byte is 0xfc or 0xfd
  let first = v6.octets()[0];
  first == 0xfc || first == 0xfd
}

fn is_ipv6_link_local(v6: &std::net::Ipv6Addr) -> bool {
  // fe80::/10 — first 10 bits are 1111 1110 10
  let octets = v6.octets();
  octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80
}

/// Validator-crate-compatible function for URL fields that must use http/https scheme.
/// Allows private IPs (for local Ollama, MCP servers).
/// Use with `#[validate(custom(function = "crate::validate_http_url"))]`
pub fn validate_http_url(url: &str) -> Result<(), validator::ValidationError> {
  validate_outbound_url(url, true)
    .map(|_| ())
    .map_err(|_| validator::ValidationError::new("invalid_url_scheme"))
}

#[cfg(test)]
#[path = "test_url_validator.rs"]
mod test_url_validator;
