use axum::http::{header::HOST, HeaderMap};

/// Extract the request host from the Host header, handling host:port format
pub fn extract_request_host(headers: &HeaderMap) -> Option<String> {
  headers
    .get(HOST)
    .and_then(|host_header| host_header.to_str().ok())
    .map(|host_str| {
      // Extract host part from host:port string
      let host = host_str.split(':').next().unwrap_or(host_str);
      host.to_string()
    })
    .and_then(|host| {
      // Basic validation - allow alphanumeric, dots, and hyphens
      if is_valid_hostname(&host) {
        Some(host)
      } else {
        None
      }
    })
}

/// Validate if a hostname is valid (basic validation)
pub fn is_valid_hostname(hostname: &str) -> bool {
  // Basic validation: alphanumeric, dots, hyphens, length check
  if hostname.is_empty() || hostname.len() > 253 {
    return false;
  }

  hostname
    .chars()
    .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
}