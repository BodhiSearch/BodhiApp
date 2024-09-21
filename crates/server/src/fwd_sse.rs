use axum::{body::Body, response::Response};
use futures::{Stream, StreamExt};
use std::convert::Infallible;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;

pub struct RawSSE<S>(S);

impl<S> RawSSE<S>
where
  S: Stream<Item = String> + Send + 'static,
{
  pub fn new(stream: S) -> Self {
    RawSSE(stream)
  }

  pub fn into_response(self) -> Response {
    let body = Body::from_stream(self.0.map(Ok::<_, Infallible>));

    Response::builder()
      .header("Content-Type", "text/event-stream")
      .header("Cache-Control", "no-cache")
      .body(body)
      .unwrap()
  }
}

pub fn fwd_sse(rx: Receiver<String>) -> Response {
  let stream = ReceiverStream::new(rx);
  RawSSE::new(stream).into_response()
}

#[cfg(test)]
mod tests {
  use crate::{fwd_sse, test_utils::ResponseTestExt};
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
  };
  use std::time::Duration;
  use tokio::sync::mpsc;
  use tower::ServiceExt;

  #[tokio::test]
  async fn test_proxy_sse_handler() -> anyhow::Result<()> {
    let app = Router::new().route(
      "/sse",
      get(|| async {
        let (tx, rx) = mpsc::channel::<String>(100);
        tokio::spawn(async move {
          for i in 1..=3 {
            tx.send(format!("data: message {}\n\n", i)).await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
          }
        });
        fwd_sse(rx)
      }),
    );

    let request = Request::builder().uri("/sse").body(Body::empty())?;
    let response = app.oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let response = response.direct_sse().await?;
    assert_eq!(6, response.len());
    assert_eq!(
      vec![
        "data: message 1".to_string(),
        "".to_string(),
        "data: message 2".to_string(),
        "".to_string(),
        "data: message 3".to_string(),
        "".to_string(),
      ],
      response
    );
    Ok(())
  }
}
