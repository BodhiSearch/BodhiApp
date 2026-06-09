use axum::http::{header::HOST, HeaderMap};

pub fn extract_request_host(headers: &HeaderMap) -> Option<String> {
  headers
    .get(HOST)
    .and_then(|host_header| host_header.to_str().ok())
    .map(|host_str| {
      let host = host_str.split(':').next().unwrap_or(host_str);
      host.to_string()
    })
    .and_then(|host| {
      if is_valid_hostname(&host) {
        Some(host)
      } else {
        None
      }
    })
}

pub fn is_valid_hostname(hostname: &str) -> bool {
  if hostname.is_empty() || hostname.len() > 253 {
    return false;
  }

  hostname
    .chars()
    .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
}
