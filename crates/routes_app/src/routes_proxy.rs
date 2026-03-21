use axum::{
  body::Body,
  extract::Request,
  http::{header, uri::Uri, Response, StatusCode},
  routing::any,
  Router,
};
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use tracing::{debug, error};

type HttpClient = Client<(), ()>;

/// Build a proxy router that forwards `/ui`, `/ui/`, and `/ui/{*path}` to a backend.
///
/// Uses explicit routes (not `nest`) to avoid axum's trailing-slash issues.
/// The request path is forwarded as-is since it already contains the `/ui` prefix
/// that the Next.js dev server expects (basePath is active in dev mode).
pub fn build_ui_proxy_router(backend_url: String) -> Router {
  let url1 = backend_url.clone();
  let url2 = backend_url.clone();
  let url3 = backend_url;
  Router::new()
    .route(
      "/ui",
      any(move |req: Request| proxy_handler(req, url1.clone())),
    )
    .route(
      "/ui/",
      any(move |req: Request| proxy_handler(req, url2.clone())),
    )
    .route(
      "/ui/{*path}",
      any(move |req: Request| proxy_handler(req, url3.clone())),
    )
}

/// Proxy handler that forwards the request path as-is to the backend.
///
/// The path already contains `/ui/...` since we use explicit routes, not `nest`.
async fn proxy_handler(req: Request, backend_url: String) -> Response<Body> {
  let path = req
    .uri()
    .path_and_query()
    .map(|x| x.as_str())
    .unwrap_or("/")
    .to_string();

  if is_websocket_upgrade(&req) {
    debug!(%path, version = ?req.version(), "proxying websocket upgrade");
    ws_proxy(req, &backend_url, &path).await
  } else {
    debug!(%path, "proxying http request");
    http_proxy(req, &backend_url, &path).await
  }
}

fn is_websocket_upgrade(req: &Request) -> bool {
  req
    .headers()
    .get(header::UPGRADE)
    .and_then(|v| v.to_str().ok())
    .map(|v| v.eq_ignore_ascii_case("websocket"))
    .unwrap_or(false)
}

async fn http_proxy(mut req: Request, backend_url: &str, path: &str) -> Response<Body> {
  let client = HttpClient::builder(TokioExecutor::new()).build_http();
  let backend_uri: Uri = backend_url.parse().unwrap();
  let backend_authority = backend_uri
    .authority()
    .map(|a| a.as_str())
    .unwrap_or("localhost:3000");
  let uri = format!("{backend_url}{path}").parse::<Uri>().unwrap();
  *req.uri_mut() = uri;
  // Replace Host header so Next.js doesn't treat this as a cross-origin request
  req
    .headers_mut()
    .insert(header::HOST, backend_authority.parse().unwrap());

  match client.request(req).await {
    Ok(res) => res.map(Body::new),
    Err(e) => {
      error!(?e, "error proxying http request");
      error_response()
    }
  }
}

async fn ws_proxy(req: Request, backend_url: &str, path: &str) -> Response<Body> {
  use hyper_util::rt::TokioIo;
  use tokio::io::{AsyncReadExt, AsyncWriteExt};
  use tokio::net::TcpStream;

  let backend_uri: Uri = backend_url.parse().unwrap();
  let host = backend_uri.host().unwrap_or("127.0.0.1");
  let port = backend_uri.port_u16().unwrap_or(3000);
  let addr = format!("{host}:{port}");

  // Extract the client upgrade handle + headers from the incoming request
  let (parts, _body) = req.into_parts();
  let req_headers = parts.headers.clone();
  let mut upgrade_req = hyper::Request::from_parts(parts, Body::empty());
  let client_upgrade = hyper::upgrade::on(&mut upgrade_req);

  // Connect raw TCP to backend
  let mut backend_stream = match TcpStream::connect(&addr).await {
    Ok(s) => s,
    Err(e) => {
      error!(?e, %addr, "failed to connect to backend for websocket");
      return error_response();
    }
  };

  // Build raw HTTP/1.1 upgrade request
  let mut raw_request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n");
  for (name, value) in req_headers.iter() {
    if name.as_str().eq_ignore_ascii_case("host") {
      continue;
    }
    if let Ok(v) = value.to_str() {
      raw_request.push_str(&format!("{}: {v}\r\n", name));
    }
  }
  raw_request.push_str("\r\n");

  debug!(raw_request_len = raw_request.len(), "sending ws upgrade to backend");

  // Write upgrade request
  if let Err(e) = backend_stream.write_all(raw_request.as_bytes()).await {
    error!(?e, "failed to write ws upgrade request to backend");
    return error_response();
  }

  // Read response headers (byte-by-byte until \r\n\r\n)
  let mut resp_buf = Vec::with_capacity(4096);
  let mut byte = [0u8; 1];
  loop {
    match backend_stream.read_exact(&mut byte).await {
      Ok(_) => {
        resp_buf.push(byte[0]);
        if resp_buf.len() >= 4 && resp_buf.ends_with(b"\r\n\r\n") {
          break;
        }
        if resp_buf.len() > 8192 {
          error!("ws upgrade response headers too large");
          return error_response();
        }
      }
      Err(e) => {
        error!(?e, "failed to read ws upgrade response from backend");
        return error_response();
      }
    }
  }

  // Parse response
  let resp_str = String::from_utf8_lossy(&resp_buf);
  let mut lines = resp_str.lines();
  let status_line = lines.next().unwrap_or("");

  if !status_line.contains(" 101 ") {
    error!(%status_line, "backend did not return 101 for ws upgrade");
    return error_response();
  }

  // Build response headers to forward to client
  let mut response_builder = Response::builder().status(StatusCode::SWITCHING_PROTOCOLS);
  for line in lines {
    if line.is_empty() {
      break;
    }
    if let Some((key, value)) = line.split_once(':') {
      response_builder = response_builder.header(key.trim(), value.trim());
    }
  }

  // Spawn bidirectional relay between upgraded client and backend
  tokio::spawn(async move {
    let upgraded_client = match client_upgrade.await {
      Ok(u) => u,
      Err(e) => {
        error!(?e, "failed to upgrade client connection");
        return;
      }
    };
    let mut client_io = TokioIo::new(upgraded_client);
    if let Err(e) = tokio::io::copy_bidirectional(&mut client_io, &mut backend_stream).await {
      debug!(?e, "websocket proxy relay ended");
    }
  });

  response_builder.body(Body::empty()).unwrap()
}

fn error_response() -> Response<Body> {
  Response::builder()
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .body(Body::from("Internal Server Error"))
    .unwrap()
}

#[cfg(test)]
mod tests {
  use crate::build_ui_proxy_router;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
  };
  use rstest::{fixture, rstest};
  use server_core::test_utils::ResponseTestExt;
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
    // Backend expects /ui-prefixed paths (simulates Next.js dev server with basePath)
    let backend_app = Router::new()
      .route("/ui", get(|| async { "Root no slash" }))
      .route("/ui/", get(|| async { "Root with slash" }))
      .route("/ui/page", get(|| async { "Proxied page" }))
      .route("/ui/api/data", get(|| async { "Proxied data" }));
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
  async fn test_proxy_forwards_ui_paths(
    #[future] backend_server: (SocketAddr, Sender<()>),
  ) -> anyhow::Result<()> {
    let (socket_addr, shutdown_tx) = backend_server;
    let app = build_ui_proxy_router(format!("http://{socket_addr}"));

    // /ui → backend /ui
    let res = app
      .clone()
      .oneshot(
        Request::builder()
          .uri("http://example.com/ui")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, res.status());
    assert_eq!("Root no slash", res.text().await.unwrap());

    // /ui/ → backend /ui/
    let res = app
      .clone()
      .oneshot(
        Request::builder()
          .uri("http://example.com/ui/")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, res.status());
    assert_eq!("Root with slash", res.text().await.unwrap());

    // /ui/page → backend /ui/page
    let res = app
      .clone()
      .oneshot(
        Request::builder()
          .uri("http://example.com/ui/page")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, res.status());
    assert_eq!("Proxied page", res.text().await.unwrap());

    // /ui/api/data → backend /ui/api/data
    let res = app
      .clone()
      .oneshot(
        Request::builder()
          .uri("http://example.com/ui/api/data")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, res.status());
    assert_eq!("Proxied data", res.text().await.unwrap());

    // /ui/missing → 404 from backend
    let res = app
      .oneshot(
        Request::builder()
          .uri("http://example.com/ui/missing")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(StatusCode::NOT_FOUND, res.status());

    shutdown_tx.send(()).unwrap();
    Ok(())
  }
}
