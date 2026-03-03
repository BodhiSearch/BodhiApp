use crate::middleware::redirects::canonical_url_middleware;
use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  middleware::from_fn_with_state,
  routing::{get, post},
  Router,
};
use rstest::rstest;
use services::test_utils::SettingServiceStub;
use services::SettingService;
use std::collections::HashMap;
use std::sync::Arc;
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
