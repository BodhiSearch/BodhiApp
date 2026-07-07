use axum::{
  body::Body,
  extract::Request,
  http::{header, Response, StatusCode},
  routing::get,
  Router,
};
use include_dir::Dir;
use std::sync::Arc;

/// Build the Content-Security-Policy header value, allowing the reference-API
/// (model catalog) origin in `connect-src`. The origin is derived from the
/// configured `reference_api_url` so prod (`api.getbodhi.app`), dev
/// (`dev-api.getbodhi.app`), and custom overrides are all honored without a
/// separate allowlist.
pub fn build_csp(reference_api_url: &str) -> String {
  let reference_origin = url::Url::parse(reference_api_url)
    .ok()
    .and_then(|u| u.origin().ascii_serialization().into())
    .filter(|origin| origin != "null");
  let connect_src = match reference_origin {
    Some(origin) => format!("'self' {origin}"),
    None => "'self'".to_string(),
  };
  format!(
    "default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src {connect_src}; font-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"
  )
}

/// Build an SPA-aware router that serves static files under `/ui/*` from an embedded directory.
///
/// Routes are explicitly registered as `/ui`, `/ui/`, and `/ui/{*path}` to avoid
/// axum `nest()` trailing-slash issues.
///
/// Behavior:
/// 1. Try exact file lookup (e.g., `/style.css` → `style.css` in dir)
/// 2. For directory paths, try appending `index.html`
/// 3. **SPA fallback**: If not found AND path has no file extension → serve root `index.html`
/// 4. If not found AND path has a file extension → return 404
pub fn build_ui_spa_router(dir: &'static Dir<'static>, csp: Arc<str>) -> Router {
  let csp_root = csp.clone();
  let csp_slash = csp.clone();
  Router::new()
    .route(
      "/ui",
      get(move |req: Request| {
        let csp = csp_root.clone();
        async move { serve_spa(dir, "/ui", req, &csp) }
      }),
    )
    .route(
      "/ui/",
      get(move |req: Request| {
        let csp = csp_slash.clone();
        async move { serve_spa(dir, "/ui/", req, &csp) }
      }),
    )
    .route(
      "/ui/{*path}",
      get(move |req: Request| {
        let csp = csp.clone();
        async move { serve_spa(dir, "/ui/", req, &csp) }
      }),
    )
}

fn serve_spa(dir: &'static Dir<'static>, prefix: &str, req: Request, csp: &str) -> Response<Body> {
  let raw_path = req.uri().path();
  let path = raw_path
    .strip_prefix(prefix)
    .unwrap_or(raw_path)
    .trim_start_matches('/');

  if let Some(response) = try_serve_file(dir, path, csp) {
    return response;
  }

  let index_path = if path.is_empty() {
    "index.html".to_string()
  } else {
    format!("{}/index.html", path.trim_end_matches('/'))
  };
  if let Some(response) = try_serve_file(dir, &index_path, csp) {
    return response;
  }

  // SPA fallback: no extension → serve root index.html
  if !has_extension(path) {
    if let Some(file) = dir.get_file("index.html") {
      return build_response(file.contents(), "text/html", csp);
    }
  }

  // File with extension not found → 404
  Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body(Body::from("Not Found"))
    .unwrap()
}

fn try_serve_file(dir: &'static Dir<'static>, path: &str, csp: &str) -> Option<Response<Body>> {
  let file = dir.get_file(path)?;
  let mime = mime_guess::from_path(path)
    .first_raw()
    .unwrap_or("application/octet-stream");
  Some(build_response(file.contents(), mime, csp))
}

fn build_response(body: &[u8], content_type: &str, csp: &str) -> Response<Body> {
  let mut builder = Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, content_type);

  // Add Content-Security-Policy to HTML responses (XSS defense-in-depth)
  if content_type.starts_with("text/html") {
    builder = builder.header("Content-Security-Policy", csp);
  }

  builder.body(Body::from(body.to_vec())).unwrap()
}

fn has_extension(path: &str) -> bool {
  path
    .rsplit('/')
    .next()
    .map(|segment| segment.contains('.'))
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::{build_csp, build_ui_spa_router};
  use axum::{body::Body, http::header, http::Request as HttpRequest, http::StatusCode};
  use include_dir::{include_dir, Dir};
  use rstest::rstest;
  use server_core::test_utils::ResponseTestExt;
  use tower::ServiceExt;

  static TEST_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/test_spa_assets");

  #[rstest]
  #[case::prod(
    "https://api.getbodhi.app",
    "connect-src 'self' https://api.getbodhi.app;"
  )]
  #[case::dev(
    "https://dev-api.getbodhi.app",
    "connect-src 'self' https://dev-api.getbodhi.app;"
  )]
  #[case::with_path(
    "https://api.getbodhi.app/api/v1",
    "connect-src 'self' https://api.getbodhi.app;"
  )]
  #[case::invalid("not a url", "connect-src 'self';")]
  fn test_build_csp(#[case] reference_api_url: &str, #[case] expected_connect_src: &str) {
    let csp = build_csp(reference_api_url);
    assert!(
      csp.contains(expected_connect_src),
      "expected connect-src `{expected_connect_src}` in `{csp}`"
    );
  }

  #[rstest]
  #[case::root_no_slash("/ui", StatusCode::OK, "text/html", "<html>root</html>\n")]
  #[case::root_slash("/ui/", StatusCode::OK, "text/html", "<html>root</html>\n")]
  #[case::exact_file("/ui/style.css", StatusCode::OK, "text/css", "body { color: red; }\n")]
  #[case::nested_index("/ui/sub/", StatusCode::OK, "text/html", "<html>sub</html>\n")]
  #[case::nested_index_no_slash("/ui/sub", StatusCode::OK, "text/html", "<html>sub</html>\n")]
  #[case::spa_fallback_no_ext("/ui/chat", StatusCode::OK, "text/html", "<html>root</html>\n")]
  #[case::spa_fallback_deep(
    "/ui/chat/session/123",
    StatusCode::OK,
    "text/html",
    "<html>root</html>\n"
  )]
  #[case::missing_asset_404("/ui/missing.js", StatusCode::NOT_FOUND, "", "")]
  #[case::js_file(
    "/ui/_next/chunk.js",
    StatusCode::OK,
    "text/javascript",
    "console.log('chunk');\n"
  )]
  #[case::non_ui_path("/api/foo", StatusCode::NOT_FOUND, "", "")]
  #[tokio::test]
  async fn test_ui_spa_router(
    #[case] path: &str,
    #[case] expected_status: StatusCode,
    #[case] expected_content_type: &str,
    #[case] expected_body: &str,
  ) {
    let csp = build_csp("https://api.getbodhi.app");
    let router = build_ui_spa_router(&TEST_DIR, csp.into());
    let req = HttpRequest::builder()
      .uri(path)
      .body(Body::empty())
      .unwrap();
    let res = router.oneshot(req).await.unwrap();
    assert_eq!(expected_status, res.status(), "status for {path}");
    if !expected_content_type.is_empty() {
      let ct = res
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
      assert!(
        ct.starts_with(expected_content_type),
        "content-type for {path}: expected {expected_content_type}, got {ct}"
      );
      if expected_content_type.starts_with("text/html") {
        let csp = res
          .headers()
          .get("Content-Security-Policy")
          .expect("CSP header on HTML response")
          .to_str()
          .unwrap();
        assert!(
          csp.contains("connect-src 'self' https://api.getbodhi.app;"),
          "CSP for {path}: {csp}"
        );
      }
    }
    if !expected_body.is_empty() {
      let body = res.text().await.unwrap();
      assert_eq!(expected_body, body, "body for {path}");
    }
  }
}
