use tower_http::trace::TraceLayer;

pub(super) fn build_app() -> axum::Router {
  axum::Router::new()
    .route("/ping", axum::routing::get(|| async { "pong" }))
    .layer(TraceLayer::new_for_http())
}
