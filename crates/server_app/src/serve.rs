use crate::{
  build_server_handle, shutdown_signal, ServerError, ServerHandle, ShutdownCallback, TaskJoinError,
};
use axum::Router;
use objs::{impl_error_from, AppError};
use routes_all::build_routes;
use server_core::{ContextError, DefaultSharedContextRw, SharedContextRw};
use services::AppService;
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::oneshot::Sender, task::JoinHandle};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ServeError {
  #[error(transparent)]
  Join(#[from] TaskJoinError),
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  Server(#[from] ServerError),
}

impl_error_from!(tokio::task::JoinError, ServeError::Join, TaskJoinError);

type Result<T> = std::result::Result<T, ServeError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ServeCommand {
  ByParams { host: String, port: u16 },
}

pub struct ShutdownContextCallback {
  ctx: Arc<dyn SharedContextRw>,
}

#[async_trait::async_trait]
impl ShutdownCallback for ShutdownContextCallback {
  async fn shutdown(&self) {
    if let Err(err) = self.ctx.try_stop().await {
      tracing::warn!(err = ?err, "error stopping llama context");
    }
  }
}

pub struct ServerShutdownHandle {
  join_handle: JoinHandle<Result<()>>,
  shutdown: Sender<()>,
}

impl ServerShutdownHandle {
  pub async fn shutdown_on_ctrlc(self) -> Result<()> {
    shutdown_signal().await;
    self.shutdown().await?;
    Ok(())
  }

  pub async fn shutdown(self) -> Result<()> {
    match self.shutdown.send(()) {
      Ok(()) => {}
      Err(err) => tracing::warn!(?err, "error sending shutdown signal on shutdown channel"),
    };
    (self.join_handle.await?)?;
    Ok(())
  }
}

impl ServeCommand {
  pub async fn aexecute(
    &self,
    service: Arc<dyn AppService>,
    static_router: Option<Router>,
  ) -> Result<()> {
    let handle = self.get_server_handle(service, static_router).await?;
    handle.shutdown_on_ctrlc().await?;
    Ok::<(), ServeError>(())
  }

  // TODO: move this to another module that returns a handle when passed server components
  pub async fn get_server_handle(
    &self,
    service: Arc<dyn AppService>,
    static_router: Option<Router>,
  ) -> Result<ServerShutdownHandle> {
    let ServeCommand::ByParams { host, port } = self;
    let ServerHandle {
      server,
      shutdown,
      ready_rx,
    } = build_server_handle(host, *port);

    let Some(library_path) = service.env_service().library_path() else {
      return Err(ContextError::LibraryPathMissing)?;
    };
    let library_path = PathBuf::from(library_path);
    if !library_path.exists() {
      return Err(ContextError::LibraryPathMissing)?;
    }
    let mut ctx = DefaultSharedContextRw::default();
    // ctx.disable_logging();
    ctx.set_library_path(library_path).await?;
    let ctx: Arc<dyn SharedContextRw> = Arc::new(ctx);
    let app = build_routes(ctx.clone(), service, static_router);

    let join_handle: JoinHandle<std::result::Result<(), ServeError>> = tokio::spawn(async move {
      let callback = Box::new(ShutdownContextCallback { ctx });
      match server.start_new(app, Some(callback)).await {
        Ok(()) => Ok(()),
        Err(err) => {
          tracing::error!(err = ?err, "server encountered an error");
          Err(err)?
        }
      }
    });
    match ready_rx.await {
      Ok(()) => {
        println!("server started on http://{host}:{port}");
      }
      Err(err) => tracing::warn!(?err, "ready channel closed before could receive signal"),
    }
    Ok(ServerShutdownHandle {
      join_handle,
      shutdown,
    })
  }
}