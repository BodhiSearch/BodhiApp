use std::time::Duration;

use reqwest::header::HeaderMap;

use super::error_wrappers::ReqwestError;
use super::url_validator::{validate_outbound_url, UrlValidationError};

/// A wrapper around `reqwest::Client` that validates outbound URLs against
/// security rules before making requests.
///
/// Always enforces scheme validation (http/https only, blocks javascript:/data:/file:).
/// Private IP/hostname blocklist is configurable via `allow_private_ips`.
#[derive(Debug, Clone)]
pub struct SafeReqwest {
  inner: reqwest::Client,
  allow_private_ips: bool,
}

impl SafeReqwest {
  pub fn builder() -> SafeReqwestBuilder {
    SafeReqwestBuilder::default()
  }

  /// Validate a URL and return a `reqwest::RequestBuilder` for a GET request.
  pub fn get(&self, url: &str) -> Result<reqwest::RequestBuilder, UrlValidationError> {
    validate_outbound_url(url, self.allow_private_ips)?;
    Ok(self.inner.get(url))
  }

  /// Validate a URL and return a `reqwest::RequestBuilder` for a POST request.
  pub fn post(&self, url: &str) -> Result<reqwest::RequestBuilder, UrlValidationError> {
    validate_outbound_url(url, self.allow_private_ips)?;
    Ok(self.inner.post(url))
  }

  /// Validate a URL and return a `reqwest::RequestBuilder` for a DELETE request.
  pub fn delete(&self, url: &str) -> Result<reqwest::RequestBuilder, UrlValidationError> {
    validate_outbound_url(url, self.allow_private_ips)?;
    Ok(self.inner.delete(url))
  }
}

#[derive(Debug, Default)]
pub struct SafeReqwestBuilder {
  timeout: Option<Duration>,
  default_headers: Option<HeaderMap>,
  allow_private_ips: bool,
}

impl SafeReqwestBuilder {
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = Some(timeout);
    self
  }

  pub fn default_headers(mut self, headers: HeaderMap) -> Self {
    self.default_headers = Some(headers);
    self
  }

  /// Allow connections to private/loopback IP addresses and localhost.
  /// Use for services that legitimately connect to local servers
  /// (e.g., Ollama for AI inference, local MCP servers).
  /// Scheme validation (http/https only) is still enforced.
  pub fn allow_private_ips(mut self) -> Self {
    self.allow_private_ips = true;
    self
  }

  pub fn build(self) -> Result<SafeReqwest, ReqwestError> {
    let mut builder = reqwest::Client::builder();
    if let Some(timeout) = self.timeout {
      builder = builder.timeout(timeout);
    }
    if let Some(headers) = self.default_headers {
      builder = builder.default_headers(headers);
    }
    Ok(SafeReqwest {
      inner: builder.build()?,
      allow_private_ips: self.allow_private_ips,
    })
  }
}
