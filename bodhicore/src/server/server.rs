use crate::error::Common;
use crate::server::utils::{DEFAULT_HOST, DEFAULT_PORT};
use axum::Router;
use derive_new::new;
use std::future::Future;
use tokio::net::TcpListener;
use tokio::sync::oneshot::{self, Receiver, Sender};

/// ServerParams encapsulates the parameters required to start the server
#[derive(Debug, Clone, PartialEq, new)]
pub struct ServerParams {
  pub host: String,
  pub port: u16,
}

impl Default for ServerParams {
  fn default() -> Self {
    Self {
      host: String::from(DEFAULT_HOST),
      port: DEFAULT_PORT,
    }
  }
}

/// Server encapsulates the parameters to start, broadcast ready lifecycle, and receive shutdown request for a server
/// It contains the parameters to start the server on given host, port etc. and
/// contains a ready sender channel to notify the requester when the server is ready to receive connection and
/// contains the shutdown receiver channel to listen to shutdown request from requester
pub struct Server {
  server_params: ServerParams,
  ready: Sender<()>,
  shutdown_rx: Receiver<()>,
}

/// ServerHandle encapuslates the handles to start, listen to when server is ready, and request shutdown for a running server
pub struct ServerHandle {
  pub server: Server,
  pub shutdown: oneshot::Sender<()>,
  pub ready_rx: oneshot::Receiver<()>,
}

pub fn build_server_handle(server_params: ServerParams) -> ServerHandle {
  let (shutdown, shutdown_rx) = oneshot::channel::<()>();
  let (ready, ready_rx) = oneshot::channel::<()>();
  let server = Server::new(server_params, ready, shutdown_rx);
  ServerHandle {
    server,
    shutdown,
    ready_rx,
  }
}

impl Server {
  fn new(server_params: ServerParams, ready: Sender<()>, shutdown_rx: Receiver<()>) -> Self {
    Self {
      server_params,
      ready,
      shutdown_rx,
    }
  }

  pub async fn start_new<F, Fut>(self, app: Router, callback: Option<F>) -> crate::error::Result<()>
  where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    let Server {
      server_params: ServerParams { host, port },
      ready,
      shutdown_rx,
    } = self;
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await.map_err(Common::Io)?;
    tracing::info!(addr = addr, "server started");
    let axum_server = axum::serve(listener, app)
      .with_graceful_shutdown(Server::shutdown_handler(shutdown_rx, callback));
    if ready.send(()).is_err() {
      tracing::warn!("ready receiver dropped before start start notified")
    };
    axum_server.await.map_err(Common::Io)?;
    Ok(())
  }

  async fn shutdown_handler<F, Fut>(shutdown_rx: Receiver<()>, callback: Option<F>)
  where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
  {
    match shutdown_rx.await.is_err() {
      true => {
        tracing::warn!(
          "shutdown sender dropped without sending a stop signal, will stop the server"
        );
      }
      false => {
        tracing::warn!("shutdown request received, starting server shutdown");
      }
    };
    if let Some(callback) = callback {
      callback().await;
    }
  }
}

#[cfg(test)]
mod test {
  use crate::server::{build_server_handle, ServerHandle, ServerParams};
  use anyhow::anyhow;
  use axum::{routing::get, Router};
  use futures_util::{future::BoxFuture, FutureExt};
  use reqwest::StatusCode;
  use std::sync::{Arc, Mutex};

  // TODO: unstable test, use ctrlc crate
  #[tokio::test]
  pub async fn test_server_start_stop_with_callback() -> anyhow::Result<()> {
    let host = "localhost".to_string();
    let port = rand::random::<u16>() % 65535;
    let ServerHandle {
      server,
      shutdown,
      ready_rx,
    } = build_server_handle(ServerParams {
      host: host.clone(),
      port,
    });
    let app = Router::new().route("/ping", get(|| async { (StatusCode::OK, "pong") }));
    let callback_received = Arc::new(Mutex::new(false));
    let callback_clone = callback_received.clone();
    let callback: Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + 'static> = Box::new(|| {
      async move {
        let mut c = callback_clone.lock().unwrap();
        *c = true;
      }
      .boxed()
    });
    let join_handle = tokio::spawn(server.start_new(app, Some(callback)));
    ready_rx.await?;
    let response = reqwest::Client::new()
      .get(format!("http://{host}:{port}/ping"))
      .send()
      .await?
      .text()
      .await?;
    assert_eq!("pong", response);
    shutdown
      .send(())
      .map_err(|_| anyhow!("shutdown send failed"))?;
    (join_handle.await?)?;
    assert!(*callback_received.lock().unwrap());
    let response = reqwest::Client::new()
      .get(format!("http://{host}:{port}/ping"))
      .send()
      .await;
    assert!(response.is_err());
    Ok(())
  }
}
