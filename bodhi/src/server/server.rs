use crate::server::bodhi_ctx::BodhiContextWrapper;
use crate::server::routes::build_routes;
use crate::{DEFAULT_HOST, DEFAULT_PORT};
use llama_server_bindings::GptParams;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot::{self, Receiver, Sender};

#[derive(Debug, Clone, PartialEq)]
pub struct ServerParams {
  pub host: String,
  pub port: u16,
  pub lazy_load_model: bool,
}

impl Default for ServerParams {
  fn default() -> Self {
    Self {
      host: String::from(DEFAULT_HOST),
      port: DEFAULT_PORT,
      lazy_load_model: true,
    }
  }
}

pub struct ServerHandle {
  pub server: Server,
  pub shutdown: oneshot::Sender<()>,
  pub ready_rx: oneshot::Receiver<()>,
}

pub fn build_server_handle(server_args: ServerParams) -> anyhow::Result<ServerHandle> {
  let (shutdown, shutdown_rx) = oneshot::channel::<()>();
  let (ready, ready_rx) = oneshot::channel::<()>();
  let server = Server::new(server_args, ready, shutdown_rx);
  let result = ServerHandle {
    server,
    shutdown,
    ready_rx,
  };
  Ok(result)
}

pub struct Server {
  server_args: ServerParams,
  ready: Sender<()>,
  shutdown_rx: Receiver<()>,
}

impl Server {
  fn new(server_args: ServerParams, ready: Sender<()>, shutdown_rx: Receiver<()>) -> Self {
    Self {
      server_args,
      ready,
      shutdown_rx,
    }
  }

  pub async fn start(self, gpt_params: GptParams) -> anyhow::Result<()> {
    let wrapper = BodhiContextWrapper::new(&gpt_params)?;
    let wrapper = Arc::new(Mutex::new(wrapper));
    let app = build_routes(wrapper.clone());
    let addr = format!("{}:{}", &self.server_args.host, &self.server_args.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(addr = addr, "Server started");
    let axum_server = axum::serve(listener, app).with_graceful_shutdown(async move {
      self.shutdown_rx.await.unwrap();
      if let Ok(mut wrapper) = wrapper.lock() {
        let result = wrapper.stop();
        if result.is_err() {
          tracing::warn!(err = format!("{result:?}"), "err stopping llama.cpp server");
        }
      }
    });
    self.ready.send(()).unwrap();
    axum_server.await?;
    Ok(())
  }
}
