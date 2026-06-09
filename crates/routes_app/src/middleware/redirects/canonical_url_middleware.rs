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
/// Redirects to the canonical public URL (from BODHI_PUBLIC_* env vars) when the
/// request host/scheme/port differ. Only GET/HEAD (to avoid breaking form POSTs),
/// 301 permanent for SEO/caching, skips exempt paths.
pub async fn canonical_url_middleware(
  headers: HeaderMap,
  State(setting_service): State<Arc<dyn SettingService>>,
  request: Request,
  next: Next,
) -> Response {
  if !setting_service.canonical_redirect_enabled().await {
    debug!("Canonical redirect is disabled, skipping redirect");
    return next.run(request).await;
  }

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

  if is_exempt_path(path) {
    debug!("Exempt path, skipping redirect");
    return next.run(request).await;
  }

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

/// Resolves the request scheme, honoring `x-forwarded-proto`/`-scheme` from proxies.
fn extract_scheme(headers: &HeaderMap, uri: &Uri) -> String {
  if let Some(forwarded_proto) = headers.get("x-forwarded-proto") {
    if let Ok(proto) = forwarded_proto.to_str() {
      return proto.to_lowercase();
    }
  }

  if let Some(forwarded_scheme) = headers.get("x-forwarded-scheme") {
    if let Ok(scheme) = forwarded_scheme.to_str() {
      return scheme.to_lowercase();
    }
  }

  uri.scheme_str().unwrap_or("http").to_lowercase()
}

fn is_exempt_path(path: &str) -> bool {
  EXEMPT_PATTERNS.iter().any(|pattern| {
    if pattern.ends_with('/') {
      path.starts_with(pattern)
    } else {
      path == *pattern
    }
  })
}

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
