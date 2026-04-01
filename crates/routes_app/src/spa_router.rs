use axum::{
  body::Body,
  extract::Request,
  http::{header, Response, StatusCode},
  routing::get,
  Router,
};
use include_dir::Dir;

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
pub fn build_ui_spa_router(dir: &'static Dir<'static>) -> Router {
  Router::new()
    .route(
      "/ui",
      get(move |req: Request| async move { serve_spa(dir, "/ui", req) }),
    )
    .route(
      "/ui/",
      get(move |req: Request| async move { serve_spa(dir, "/ui/", req) }),
    )
    .route(
      "/ui/{*path}",
      get(move |req: Request| async move { serve_spa(dir, "/ui/", req) }),
    )
}

fn serve_spa(dir: &'static Dir<'static>, prefix: &str, req: Request) -> Response<Body> {
  // Strip the /ui prefix to get the file path
  let raw_path = req.uri().path();
  let path = raw_path
    .strip_prefix(prefix)
    .unwrap_or(raw_path)
    .trim_start_matches('/');

  // Try exact file
  if let Some(response) = try_serve_file(dir, path) {
    return response;
  }

  // Try directory/index.html
  let index_path = if path.is_empty() {
    "index.html".to_string()
  } else {
    format!("{}/index.html", path.trim_end_matches('/'))
  };
  if let Some(response) = try_serve_file(dir, &index_path) {
    return response;
  }

  // SPA fallback: no extension → serve root index.html
  if !has_extension(path) {
    if let Some(file) = dir.get_file("index.html") {
      return build_response(file.contents(), "text/html");
    }
  }

  // File with extension not found → 404
  Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body(Body::from("Not Found"))
    .unwrap()
}

fn try_serve_file(dir: &'static Dir<'static>, path: &str) -> Option<Response<Body>> {
  let file = dir.get_file(path)?;
  let mime = mime_guess::from_path(path)
    .first_raw()
    .unwrap_or("application/octet-stream");
  Some(build_response(file.contents(), mime))
}

fn build_response(body: &[u8], content_type: &str) -> Response<Body> {
  let mut builder = Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, content_type);

  // Add Content-Security-Policy to HTML responses (XSS defense-in-depth)
  if content_type.starts_with("text/html") {
    builder = builder.header(
      "Content-Security-Policy",
      "default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
    );
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
  use super::build_ui_spa_router;
  use axum::{body::Body, http::header, http::Request as HttpRequest, http::StatusCode};
  use include_dir::{include_dir, Dir};
  use rstest::rstest;
  use server_core::test_utils::ResponseTestExt;
  use tower::ServiceExt;

  static TEST_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/test_spa_assets");

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
    let router = build_ui_spa_router(&TEST_DIR);
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
    }
    if !expected_body.is_empty() {
      let body = res.text().await.unwrap();
      assert_eq!(expected_body, body, "body for {path}");
    }
  }
}
