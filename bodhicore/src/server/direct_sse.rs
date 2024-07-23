use axum::{
  body::Body,
  http::header,
  response::{IntoResponse, Response},
  BoxError,
};
use bytes::{BufMut, Bytes, BytesMut};
use futures_core::{ready, Future, Stream, TryStream};
use http_body::Frame;
use pin_project_lite::pin_project;
use std::{
  fmt,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use sync_wrapper::SyncWrapper;
use tokio::time::Sleep;

#[derive(Debug, Clone, Default)]
pub struct DirectEvent {
  buffer: BytesMut,
}

impl DirectEvent {
  pub fn new() -> Self {
    DirectEvent::default()
  }

  pub fn data<T>(mut self, data: T) -> Self
  where
    T: AsRef<str>,
  {
    for line in data.as_ref().split('\n') {
      self.buffer.extend_from_slice(line.as_bytes());
    }
    self
  }

  pub fn finalize(mut self) -> Bytes {
    self.buffer.put_u8(b'\n');
    self.buffer.freeze()
  }
}

#[derive(Clone)]
#[must_use]
pub struct DirectSse<S> {
  stream: S,
  keep_alive: Option<KeepAlive>,
}

impl<S> DirectSse<S> {
  pub fn new(stream: S) -> Self
  where
    S: TryStream<Ok = DirectEvent> + Send + 'static,
    S::Error: Into<BoxError>,
  {
    DirectSse {
      stream,
      keep_alive: None,
    }
  }

  pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
    self.keep_alive = Some(keep_alive);
    self
  }
}

impl<S> fmt::Debug for DirectSse<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DirectSse")
      .field("stream", &format_args!("{}", std::any::type_name::<S>()))
      .field("keep_alive", &self.keep_alive)
      .finish()
  }
}

impl<S, E> IntoResponse for DirectSse<S>
where
  S: Stream<Item = Result<DirectEvent, E>> + Send + 'static,
  E: Into<BoxError>,
{
  fn into_response(self) -> Response {
    let sse_body = DirectSseBody {
      event_stream: SyncWrapper::new(self.stream),
      keep_alive: self.keep_alive.map(KeepAliveStream::new),
    };
    let body = Body::new(sse_body);

    Response::builder()
      .header(header::CONTENT_TYPE, mime::TEXT_EVENT_STREAM.as_ref())
      .header(header::CACHE_CONTROL, "no-cache")
      .body(body)
      .unwrap()
  }
}

pin_project! {
  pub struct DirectSseBody<S> {
      #[pin]
      event_stream: SyncWrapper<S>,
      #[pin]
      keep_alive: Option<KeepAliveStream>,
  }
}

impl<S, E> http_body::Body for DirectSseBody<S>
where
  S: Stream<Item = Result<DirectEvent, E>>,
{
  type Data = Bytes;
  type Error = E;

  fn poll_frame(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
    let this = self.project();

    match this.event_stream.get_pin_mut().poll_next(cx) {
      Poll::Pending => {
        if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
          keep_alive.poll_event(cx).map(|e| Some(Ok(Frame::data(e))))
        } else {
          Poll::Pending
        }
      }
      Poll::Ready(Some(Ok(event))) => {
        if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
          keep_alive.reset();
        }
        Poll::Ready(Some(Ok(Frame::data(event.finalize()))))
      }
      Poll::Ready(Some(Err(error))) => {
        if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
          keep_alive.reset();
        }
        Poll::Ready(Some(Err(error)))
      }
      Poll::Ready(None) => {
        if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
          keep_alive.reset();
        }
        Poll::Ready(None)
      }
    }
  }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct KeepAlive {
  event: Bytes,
  max_interval: Duration,
}

impl KeepAlive {
  pub fn new() -> Self {
    Self {
      event: Bytes::from_static(b":\n\n"),
      max_interval: Duration::from_secs(15),
    }
  }

  pub fn interval(mut self, time: Duration) -> Self {
    self.max_interval = time;
    self
  }

  pub fn event(mut self, event: DirectEvent) -> Self {
    self.event = event.finalize();
    self
  }
}

impl Default for KeepAlive {
  fn default() -> Self {
    Self::new()
  }
}

pin_project! {
    #[derive(Debug)]
    struct KeepAliveStream {
        keep_alive: KeepAlive,
        #[pin]
        alive_timer: Sleep,
    }
}

impl KeepAliveStream {
  fn new(keep_alive: KeepAlive) -> Self {
    Self {
      alive_timer: tokio::time::sleep(keep_alive.max_interval),
      keep_alive,
    }
  }

  fn reset(self: Pin<&mut Self>) {
    let this = self.project();
    this
      .alive_timer
      .reset(tokio::time::Instant::now() + this.keep_alive.max_interval);
  }

  fn poll_event(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Bytes> {
    let this = self.as_mut().project();
    ready!(this.alive_timer.poll(cx));
    let event = this.keep_alive.event.clone();
    self.reset();
    Poll::Ready(event)
  }
}

#[cfg(test)]
mod test {
  use std::time::Duration;

  use super::{DirectSse, KeepAlive};
  use crate::{server::direct_sse::DirectEvent, test_utils::ResponseTestExt};
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
  };
  use futures_util::StreamExt;
  use tokio_stream::wrappers::ReceiverStream;
  use tower::ServiceExt;

  async fn stream_handler() -> Result<Response, String> {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
    let stream = ReceiverStream::new(rx)
      .map::<Result<DirectEvent, String>, _>(move |msg| Ok(DirectEvent::new().data(msg)));
    tokio::spawn(async move {
      for i in 1..=3 {
        tx.send(format!("value {i}")).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
      }
    });
    Ok(DirectSse::new(stream).into_response())
  }

  #[tokio::test]
  pub async fn test_direct_sse_sends_sse_event() -> anyhow::Result<()> {
    let app = Router::new().route("/test/stream", get(stream_handler));
    let response = app
      .oneshot(Request::get("/test/stream").body(Body::empty()).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let response = response.direct_sse().await.unwrap();
    assert_eq!(3, response.len());
    assert_eq!(
      vec![
        "value 1".to_string(),
        "value 2".to_string(),
        "value 3".to_string()
      ],
      response
    );
    Ok(())
  }

  async fn stream_handler_keep_alive() -> Result<Response, String> {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
    let stream = ReceiverStream::new(rx)
      .map::<Result<DirectEvent, String>, _>(move |msg| Ok(DirectEvent::new().data(msg)));
    tokio::spawn(async move {
      for i in 1..=3 {
        tx.send(format!("value {i}")).await.unwrap();
        tokio::time::sleep(Duration::from_millis(11)).await;
      }
    });
    Ok(
      DirectSse::new(stream)
        .keep_alive(
          KeepAlive::new()
            .event(DirectEvent::new().data(":"))
            .interval(Duration::from_millis(10)),
        )
        .into_response(),
    )
  }

  #[tokio::test]
  pub async fn test_direct_sse_sends_sse_event_with_keep_alive() -> anyhow::Result<()> {
    let app = Router::new().route("/test/stream", get(stream_handler_keep_alive));
    let response = app
      .oneshot(Request::get("/test/stream").body(Body::empty()).unwrap())
      .await
      .unwrap();
    assert_eq!(StatusCode::OK, response.status());
    let response = response.direct_sse().await.unwrap();
    assert_eq!(6, response.len());
    assert_eq!(
      vec![
        "value 1".to_string(),
        ":".to_string(),
        "value 2".to_string(),
        ":".to_string(),
        "value 3".to_string(),
        ":".to_string(),
      ],
      response
    );
    Ok(())
  }
}
