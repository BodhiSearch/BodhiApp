use axum::{
  extract::{Request, State},
  http::{header, HeaderMap, StatusCode, Uri},
  middleware::Next,
  response::{IntoResponse, Response},
};
use services::SettingService;
use std::sync::Arc;
use tracing::debug;

const EXEMPT_PATTERNS: &[&str] = &["/health", "/ping"];
/// Middleware that redirects requests to the canonical public URL if needed.
///
/// This middleware checks if the incoming request matches the canonical public URL
/// configured via BODHI_PUBLIC_* environment variables. If not, it redirects the
/// request to the canonical URL while preserving the path and query parameters.
///
/// Features:
/// - Only redirects GET/HEAD requests to avoid affecting form submissions
/// - Preserves full request path, query parameters, and fragments
/// - Uses 301 (permanent) redirects for SEO and caching benefits
/// - Skips health check endpoints and other exempt paths
pub async fn canonical_url_middleware(
  headers: HeaderMap,
  State(setting_service): State<Arc<dyn SettingService>>,
  request: Request,
  next: Next,
) -> Response {
  // Skip redirect if canonical redirect is disabled
  if !setting_service.canonical_redirect_enabled().await {
    debug!("Canonical redirect is disabled, skipping redirect");
    return next.run(request).await;
  }

  // Skip redirect if public_host is not explicitly set
  if setting_service.get_public_host_explicit().await.is_none() {
    debug!("Public host is not explicitly set, skipping redirect");
    return next.run(request).await;
  }
  let method = request.method();
  let uri = request.uri();
  let path = uri.path();
  // Only redirect GET and HEAD requests to avoid breaking forms and APIs
  if !matches!(method.as_str(), "GET" | "HEAD") {
    debug!("Not a GET or HEAD request, skipping redirect");
    return next.run(request).await;
  }

  // Skip redirects for health check and special endpoints
  if is_exempt_path(path) {
    debug!("Exempt path, skipping redirect");
    return next.run(request).await;
  }

  // Extract request URL components
  let request_scheme = extract_scheme(&headers, uri);
  let request_host = match headers.get(header::HOST) {
    Some(host_header) => match host_header.to_str() {
      Ok(host) => host,
      Err(_) => {
        debug!("Invalid host header, skipping redirect");
        return next.run(request).await;
      }
    },
    None => {
      debug!("No host header found, skipping redirect");
      return next.run(request).await;
    }
  };

  // Check if redirect is needed by comparing with canonical URL
  if should_redirect_to_canonical(setting_service.as_ref(), &request_scheme, request_host).await {
    let canonical_url = setting_service.public_server_url().await;
    let full_canonical_url = build_canonical_url(&canonical_url, uri);
    debug!(
      "Redirecting {}://{}{} to {}",
      request_scheme, request_host, path, full_canonical_url
    );

    return (
      StatusCode::MOVED_PERMANENTLY,
      [("location", &full_canonical_url)],
    )
      .into_response();
  }

  debug!(
    "No redirect needed for {}://{}{}",
    request_scheme, request_host, path
  );
  next.run(request).await
}

/// Extract the scheme from the request, considering proxy headers
fn extract_scheme(headers: &HeaderMap, uri: &Uri) -> String {
  // Check for forwarded protocol headers (common in load balancers/proxies)
  if let Some(forwarded_proto) = headers.get("x-forwarded-proto") {
    if let Ok(proto) = forwarded_proto.to_str() {
      return proto.to_lowercase();
    }
  }

  // Check for other common forwarded headers
  if let Some(forwarded_scheme) = headers.get("x-forwarded-scheme") {
    if let Ok(scheme) = forwarded_scheme.to_str() {
      return scheme.to_lowercase();
    }
  }

  // Fall back to URI scheme or default to http
  uri.scheme_str().unwrap_or("http").to_lowercase()
}

/// Check if a path should be exempt from canonical URL redirects
fn is_exempt_path(path: &str) -> bool {
  EXEMPT_PATTERNS.iter().any(|pattern| {
    if pattern.ends_with('/') {
      path.starts_with(pattern)
    } else {
      path == *pattern
    }
  })
}

/// Check if a request should be redirected to the canonical URL
async fn should_redirect_to_canonical(
  setting_service: &dyn SettingService,
  request_scheme: &str,
  request_host: &str,
) -> bool {
  let canonical_scheme = setting_service.public_scheme().await;
  let canonical_host = setting_service.public_host().await;
  let canonical_port = setting_service.public_port().await;

  // Parse request host to handle host:port format
  let (req_host, req_port) = if let Some((host, port_str)) = request_host.rsplit_once(':') {
    if let Ok(port) = port_str.parse::<u16>() {
      (host, port)
    } else {
      // Invalid port format, should redirect
      return true;
    }
  } else {
    // No port specified, use default based on scheme
    let default_port = match request_scheme {
      "https" => 443,
      "http" => 80,
      _ => return true, // Unknown scheme, should redirect
    };
    (request_host, default_port)
  };

  let scheme_matches = request_scheme == canonical_scheme;
  let host_matches = req_host == canonical_host;
  let port_matches = req_port == canonical_port;

  !(scheme_matches && host_matches && port_matches)
}

/// Build the full canonical URL including path and query parameters
fn build_canonical_url(canonical_base: &str, original_uri: &Uri) -> String {
  let mut canonical_url = canonical_base.trim_end_matches('/').to_string();

  if let Some(path_and_query) = original_uri.path_and_query() {
    canonical_url.push_str(path_and_query.as_str());
  } else {
    canonical_url.push_str(original_uri.path());
  }

  canonical_url
}

#[cfg(test)]
#[path = "test_canonical_url_middleware.rs"]
mod test_canonical_url_middleware;
