use tracing::{error, info, warn};

/// Mask sensitive parameters for logging with improved logic
/// - For params < 8 chars: show first 2 chars + *
/// - For params >= 8 chars: show first 2 and last 2 chars + *** in between
pub fn mask_sensitive_value(value: &str) -> String {
  if value.len() < 8 {
    if value.len() >= 2 {
      format!("{}*", &value[..2])
    } else {
      "*".to_string()
    }
  } else {
    format!("{}***{}", &value[..2], &value[value.len() - 2..])
  }
}

/// Mask all form parameters for logging (assuming all are potentially sensitive)
pub fn mask_form_params(params: &[(&str, &str)]) -> Vec<(String, String)> {
  params
    .iter()
    .map(|(key, value)| (key.to_string(), mask_sensitive_value(value)))
    .collect()
}

/// Log HTTP request details with masked parameters
pub fn log_http_request(method: &str, url: &str, service: &str, params: Option<&[(&str, &str)]>) {
  if let Some(params) = params {
    let masked_params = mask_form_params(params);
    info!(
      method = method,
      url = url,
      params = ?masked_params,
      service = service,
      "HTTP request started"
    );
  } else {
    info!(
      method = method,
      url = url,
      service = service,
      "HTTP request started"
    );
  }
}

/// Log HTTP response details
pub fn log_http_response(method: &str, url: &str, service: &str, status: u16, success: bool) {
  if success {
    info!(
      method = method,
      url = url,
      status = status,
      service = service,
      "HTTP request completed successfully"
    );
  } else {
    warn!(
      method = method,
      url = url,
      status = status,
      service = service,
      "HTTP request failed"
    );
  }
}

/// Log HTTP error details
pub fn log_http_error(method: &str, url: &str, service: &str, error: &str) {
  error!(
    method = method,
    url = url,
    error = error,
    service = service,
    "HTTP request error"
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mask_sensitive_value_short() {
    assert_eq!(mask_sensitive_value("ab"), "ab*");
    assert_eq!(mask_sensitive_value("a"), "*");
    assert_eq!(mask_sensitive_value(""), "*");
    assert_eq!(mask_sensitive_value("abc"), "ab*");
    assert_eq!(mask_sensitive_value("abcdefg"), "ab*");
  }

  #[test]
  fn test_mask_sensitive_value_long() {
    assert_eq!(mask_sensitive_value("abcdefgh"), "ab***gh");
    assert_eq!(mask_sensitive_value("abcdefghijk"), "ab***jk");
    assert_eq!(
      mask_sensitive_value("very_long_secret_token_12345"),
      "ve***45"
    );
  }

  #[test]
  fn test_mask_form_params() {
    let params = [
      ("grant_type", "client_credentials"),
      ("client_id", "short"),
      ("client_secret", "very_long_secret_token"),
      ("refresh_token", "another_very_long_token_value"),
    ];

    let masked = mask_form_params(&params);

    assert_eq!(masked[0], ("grant_type".to_string(), "cl***ls".to_string()));
    assert_eq!(masked[1], ("client_id".to_string(), "sh*".to_string()));
    assert_eq!(
      masked[2],
      ("client_secret".to_string(), "ve***en".to_string())
    );
    assert_eq!(
      masked[3],
      ("refresh_token".to_string(), "an***ue".to_string())
    );
  }
}
