use std::time::Duration;

use reqwest::header::HeaderMap;

use super::error_wrappers::ReqwestError;
use super::url_validator::{validate_outbound_url, UrlValidationError};

/// Validates outbound URLs against security rules before making requests.
/// Enforces scheme validation (http/https only). Private IP blocklist is configurable.
#[derive(Debug, Clone)]
pub struct SafeReqwest {
  inner: reqwest::Client,
  allow_private_ips: bool,
}

impl SafeReqwest {
  pub fn builder() -> SafeReqwestBuilder {
    SafeReqwestBuilder::default()
  }

  pub fn request(
    &self,
    method: reqwest::Method,
    url: &str,
  ) -> Result<reqwest::RequestBuilder, UrlValidationError> {
    validate_outbound_url(url, self.allow_private_ips)?;
    Ok(self.inner.request(method, url))
  }

  pub fn get(&self, url: &str) -> Result<reqwest::RequestBuilder, UrlValidationError> {
    validate_outbound_url(url, self.allow_private_ips)?;
    Ok(self.inner.get(url))
  }

  pub fn post(&self, url: &str) -> Result<reqwest::RequestBuilder, UrlValidationError> {
    validate_outbound_url(url, self.allow_private_ips)?;
    Ok(self.inner.post(url))
  }

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
