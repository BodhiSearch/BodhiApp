use axum::{
  body::Body,
  extract::Request,
  http::{uri::Uri, Response},
  Router,
};
use hyper_util::{client::legacy::Client, rt::TokioExecutor};

type HttpClient = Client<(), ()>;

pub fn proxy_router(backend_url: String) -> Router {
  Router::new().fallback(move |req| proxy_handler(req, backend_url.clone()))
}

async fn proxy_handler(mut req: Request, backend_url: String) -> Response<Body> {
  let client = HttpClient::builder(TokioExecutor::new()).build_http();
  let uri = format!(
    "{backend_url}{}",
    req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("")
  )
  .parse::<Uri>()
  .unwrap();

  *req.uri_mut() = uri;

  match client.request(req).await {
    Ok(res) => res.map(Body::new),
    Err(e) => {
      eprintln!("Error: {}", e);
      Response::builder()
        .status(500)
        .body(Body::from("Internal Server Error"))
        .unwrap()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_utils::ResponseTestExt;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
  };
  use rstest::{fixture, rstest};
  use std::net::SocketAddr;
  use tokio::{
    net::TcpListener,
    sync::oneshot::{channel, Sender},
  };
  use tower::ServiceExt;

  #[fixture]
  async fn backend_server() -> (SocketAddr, Sender<()>) {
    let addr = "127.0.0.1:0";
    let listener = TcpListener::bind(&addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    let backend_app = Router::new().route("/proxy-handled", get(|| async { "Proxied response" }));
    let (shutdown_tx, shutdown_rx) = channel::<()>();
    tokio::spawn(async move {
      axum::serve(listener, backend_app)
        .with_graceful_shutdown(async move {
          shutdown_rx.await.unwrap();
        })
        .await
        .unwrap();
    });
    (local_addr, shutdown_tx)
  }

  #[rstest]
  #[awt]
  #[tokio::test]
  async fn test_proxy_handler(
    #[future] backend_server: (SocketAddr, Sender<()>),
  ) -> anyhow::Result<()> {
    let (socket_addr, shutdown_tx) = backend_server;
    let app = Router::new()
      .route("/test", get(|| async { "Test response" }))
      .merge(proxy_router(format!("http://{socket_addr}")));

    // Test attended request (handled by the router)
    let req = Request::builder()
      .uri("http://example.com/test")
      .body(Body::empty())
      .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.text().await.unwrap(), "Test response");

    // response handled by proxy backend
    let res = app
      .clone()
      .oneshot(
        Request::builder()
          .uri("http://example.com/proxy-handled")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.text().await.unwrap(), "Proxied response");

    // response not handled by proxy backend
    let req = Request::builder()
      .uri("http://example.com/unattended")
      .body(Body::empty())
      .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    shutdown_tx.send(()).unwrap();
    Ok(())
  }
}
