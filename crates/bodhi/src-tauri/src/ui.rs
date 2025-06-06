use axum::Router;
use include_dir::{include_dir, Dir};
use tower_serve_static::ServeDir;

static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../dist");

pub fn router() -> Router {
  let static_service = ServeDir::new(&ASSETS).append_index_html_on_directories(true);
  Router::new().fallback_service(static_service)
}
