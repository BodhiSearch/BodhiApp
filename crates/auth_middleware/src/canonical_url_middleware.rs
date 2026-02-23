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
mod tests {
  use super::*;
  use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
  };
  use rstest::rstest;
  use services::test_utils::SettingServiceStub;
  use std::collections::HashMap;
  use tower::ServiceExt;

  // Test scenarios for canonical URL middleware
  #[derive(Debug)]
  struct TestScenario {
    name: &'static str,
    method: Method,
    path: &'static str,
    host_header: &'static str,
    forwarded_proto: Option<&'static str>,
    public_scheme: &'static str,
    public_host: &'static str,
    public_port: u16,
    expected_status: StatusCode,
    expected_location: Option<&'static str>,
    skip_public_host: bool,   // When true, don't set BODHI_PUBLIC_HOST
    canonical_disabled: bool, // When true, disable canonical redirect setting
  }

  async fn create_setting_service(scheme: &str, host: &str, port: u16) -> Arc<dyn SettingService> {
    let mut settings = HashMap::new();
    settings.insert("BODHI_PUBLIC_SCHEME".to_string(), scheme.to_string());
    settings.insert("BODHI_PUBLIC_HOST".to_string(), host.to_string());
    settings.insert("BODHI_PUBLIC_PORT".to_string(), port.to_string());
    settings.insert("BODHI_CANONICAL_REDIRECT".to_string(), "true".to_string());
    let stub = SettingServiceStub::default()
      .append_settings(settings)
      .await;
    Arc::new(stub)
  }

  async fn create_setting_service_no_public_host() -> Arc<dyn SettingService> {
    let mut settings = HashMap::new();
    settings.insert("BODHI_CANONICAL_REDIRECT".to_string(), "true".to_string());
    let stub = SettingServiceStub::default()
      .append_settings(settings)
      .await;
    Arc::new(stub)
  }

  async fn create_setting_service_canonical_disabled() -> Arc<dyn SettingService> {
    let mut settings = HashMap::new();
    settings.insert("BODHI_CANONICAL_REDIRECT".to_string(), "false".to_string());
    settings.insert("BODHI_PUBLIC_SCHEME".to_string(), "https".to_string());
    settings.insert(
      "BODHI_PUBLIC_HOST".to_string(),
      "bodhi.example.com".to_string(),
    );
    settings.insert("BODHI_PUBLIC_PORT".to_string(), "443".to_string());
    let stub = SettingServiceStub::default()
      .append_settings(settings)
      .await;
    Arc::new(stub)
  }

  async fn test_handler() -> &'static str {
    "OK"
  }

  #[rstest]
  #[case::redirect_wrong_scheme(TestScenario {
    name: "redirect when scheme doesn't match",
    method: Method::GET,
    path: "/test",
    host_header: "bodhi.example.com",
    forwarded_proto: None, // Will default to http
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::MOVED_PERMANENTLY,
    expected_location: Some("https://bodhi.example.com/test"),
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::redirect_wrong_host(TestScenario {
    name: "redirect when host doesn't match",
    method: Method::GET,
    path: "/test",
    host_header: "other.example.com",
    forwarded_proto: Some("https"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::MOVED_PERMANENTLY,
    expected_location: Some("https://bodhi.example.com/test"),
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::redirect_wrong_port(TestScenario {
    name: "redirect when port doesn't match",
    method: Method::GET,
    path: "/test",
    host_header: "bodhi.example.com:8080",
    forwarded_proto: Some("https"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::MOVED_PERMANENTLY,
    expected_location: Some("https://bodhi.example.com/test"),
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::redirect_wrong_port_non_standard(TestScenario {
    name: "redirect when canonical uses non-standard port",
    method: Method::GET,
    path: "/test",
    host_header: "bodhi.example.com",
    forwarded_proto: Some("https"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 8080,
    expected_status: StatusCode::MOVED_PERMANENTLY,
    expected_location: Some("https://bodhi.example.com:8080/test"),
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::no_redirect_canonical_match(TestScenario {
    name: "no redirect when everything matches",
    method: Method::GET,
    path: "/test",
    host_header: "bodhi.example.com",
    forwarded_proto: Some("https"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::no_redirect_canonical_match_explicit_port(TestScenario {
    name: "no redirect when everything matches with explicit port",
    method: Method::GET,
    path: "/test",
    host_header: "bodhi.example.com:443",
    forwarded_proto: Some("https"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::no_redirect_canonical_match_non_standard_port(TestScenario {
    name: "no redirect when canonical non-standard port matches",
    method: Method::GET,
    path: "/test",
    host_header: "bodhi.example.com:8080",
    forwarded_proto: Some("https"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 8080,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::no_redirect_post_method(TestScenario {
    name: "no redirect for POST requests",
    method: Method::POST,
    path: "/api/test",
    host_header: "wrong-host.example.com",
    forwarded_proto: Some("http"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::no_redirect_health_path(TestScenario {
    name: "no redirect for exempt health path",
    method: Method::GET,
    path: "/health",
    host_header: "wrong-host.example.com",
    forwarded_proto: Some("http"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::no_redirect_ping_path(TestScenario {
    name: "no redirect for exempt ping path",
    method: Method::GET,
    path: "/ping",
    host_header: "wrong-host.example.com",
    forwarded_proto: Some("http"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::redirect_with_query_params(TestScenario {
    name: "redirect preserves query parameters",
    method: Method::GET,
    path: "/ui/chat?model=gpt-3.5&temp=0.7",
    host_header: "bodhi.example.com",
    forwarded_proto: None, // Will default to http
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::MOVED_PERMANENTLY,
    expected_location: Some("https://bodhi.example.com/ui/chat?model=gpt-3.5&temp=0.7"),
    skip_public_host: false,
    canonical_disabled: false,
  })]
  #[case::no_redirect_no_public_host_wrong_host(TestScenario {
    name: "no redirect when public_host not set even with wrong host",
    method: Method::GET,
    path: "/test",
    host_header: "wrong-host.example.com",
    forwarded_proto: Some("https"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: true,
    canonical_disabled: false,
  })]
  #[case::no_redirect_no_public_host_wrong_scheme(TestScenario {
    name: "no redirect when public_host not set even with wrong scheme",
    method: Method::GET,
    path: "/test",
    host_header: "bodhi.example.com",
    forwarded_proto: Some("http"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: true,
    canonical_disabled: false,
  })]
  #[case::no_redirect_no_public_host_wrong_port(TestScenario {
    name: "no redirect when public_host not set even with wrong port",
    method: Method::GET,
    path: "/test",
    host_header: "bodhi.example.com:8080",
    forwarded_proto: Some("https"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: true,
    canonical_disabled: false,
  })]
  #[case::no_redirect_canonical_disabled_wrong_host(TestScenario {
    name: "no redirect when canonical redirect is disabled even with wrong host",
    method: Method::GET,
    path: "/test",
    host_header: "wrong-host.example.com",
    forwarded_proto: Some("http"),
    public_scheme: "https",
    public_host: "bodhi.example.com",
    public_port: 443,
    expected_status: StatusCode::OK,
    expected_location: None,
    skip_public_host: false,
    canonical_disabled: true,
  })]
  #[tokio::test]
  async fn test_canonical_url_middleware_scenarios(#[case] scenario: TestScenario) {
    let setting_service = if scenario.canonical_disabled {
      create_setting_service_canonical_disabled().await
    } else if scenario.skip_public_host {
      create_setting_service_no_public_host().await
    } else {
      create_setting_service(
        scenario.public_scheme,
        scenario.public_host,
        scenario.public_port,
      )
      .await
    };

    // Create router with middleware and appropriate route
    let router = if scenario.method == Method::POST {
      Router::new().route(scenario.path, post(test_handler))
    } else {
      Router::new().route(scenario.path, get(test_handler))
    };

    let app = router.layer(from_fn_with_state(
      setting_service,
      canonical_url_middleware,
    ));

    // Build request
    let mut request_builder = Request::builder()
      .method(scenario.method)
      .uri(scenario.path)
      .header("host", scenario.host_header);

    if let Some(proto) = scenario.forwarded_proto {
      request_builder = request_builder.header("x-forwarded-proto", proto);
    }

    let request = request_builder.body(Body::empty()).unwrap();

    // Execute request
    let response = app.oneshot(request).await.unwrap();

    // Verify status
    assert_eq!(
      response.status(),
      scenario.expected_status,
      "Failed for scenario: {}",
      scenario.name
    );

    // Verify location header if redirect expected
    if let Some(expected_location) = scenario.expected_location {
      let location = response
        .headers()
        .get("location")
        .expect("Location header should be present")
        .to_str()
        .unwrap();
      assert_eq!(
        location, expected_location,
        "Failed for scenario: {}",
        scenario.name
      );
    } else {
      assert!(
        response.headers().get("location").is_none(),
        "Location header should not be present for scenario: {}",
        scenario.name
      );
    }
  }
}
