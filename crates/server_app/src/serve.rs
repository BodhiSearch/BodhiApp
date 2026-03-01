use crate::{
  build_server_handle, shutdown_signal, ServerError, ServerHandle, ServerKeepAlive,
  ShutdownCallback, TaskJoinError, VariantChangeListener,
};
use axum::Router;
use include_dir::Dir;
use routes_app::build_routes;
use server_core::{ContextError, DefaultSharedContext, SharedContext};
use services::{impl_error_from, AppError, ErrorType};
use services::{AppService, SettingServiceError};
use std::sync::Arc;
use tokio::{sync::oneshot::Sender, task::JoinHandle};
use tower_serve_static::ServeDir;
use tracing::error;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ServeError {
  #[error(transparent)]
  Setting(#[from] SettingServiceError),
  #[error(transparent)]
  Join(#[from] TaskJoinError),
  #[error(transparent)]
  Context(#[from] ContextError),
  #[error(transparent)]
  Server(#[from] ServerError),
  #[error("Server started but readiness signal not received.")]
  #[error_meta(error_type = ErrorType::Unknown)]
  Unknown,
}

impl_error_from!(tokio::task::JoinError, ServeError::Join, TaskJoinError);

type Result<T> = std::result::Result<T, ServeError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ServeCommand {
  ByParams { host: String, port: u16 },
}

pub struct ShutdownContextCallback {
  ctx: Arc<dyn SharedContext>,
}

#[async_trait::async_trait]
impl ShutdownCallback for ShutdownContextCallback {
  async fn shutdown(&self) {
    if let Err(err) = self.ctx.stop().await {
      tracing::warn!(err = ?err, "error stopping llama context");
    }
  }
}

#[derive(Debug)]
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
    static_dir: Option<&'static Dir<'static>>,
  ) -> Result<()> {
    let handle = self.get_server_handle(service, static_dir).await?;
    handle.shutdown_on_ctrlc().await?;
    Ok::<(), ServeError>(())
  }

  // TODO: move this to another module that returns a handle when passed server components
  pub async fn get_server_handle(
    &self,
    service: Arc<dyn AppService>,
    static_dir: Option<&'static Dir<'static>>,
  ) -> Result<ServerShutdownHandle> {
    let ServeCommand::ByParams { host, port } = self;
    let setting_service = service.setting_service();
    let ServerHandle {
      server,
      shutdown,
      ready_rx,
    } = build_server_handle(host, *port);

    let exec_path = service.setting_service().exec_path_from().await;
    if !exec_path.exists() {
      error!("exec not found at {}", exec_path.to_string_lossy());
      return Err(ContextError::ExecNotExists(
        exec_path.to_string_lossy().to_string(),
      ))?;
    }
    let ctx = DefaultSharedContext::new(service.hub_service(), service.setting_service()).await;
    let ctx: Arc<dyn SharedContext> = Arc::new(ctx);
    setting_service
      .add_listener(Arc::new(VariantChangeListener::new(ctx.clone())))
      .await;

    let keep_alive = Arc::new(ServerKeepAlive::new(
      ctx.clone(),
      setting_service.keep_alive().await,
    ));
    setting_service.add_listener(keep_alive.clone()).await;
    ctx.add_state_listener(keep_alive).await;

    // Create static router from directory if provided
    let static_router = static_dir.map(|dir| {
      let static_service = ServeDir::new(dir).append_index_html_on_directories(true);
      Router::new().fallback_service(static_service)
    });

    let app = build_routes(ctx.clone(), service, static_router).await;
    let scheme = setting_service.scheme().await;
    let server_url = format!("{scheme}://{host}:{port}");
    let public_url = setting_service.public_server_url().await;

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
        println!("server started on server_url={server_url}, public_url={public_url}");
        tracing::info!(server_url, public_url, "server started");
      }
      Err(err) => {
        tracing::warn!(?err, "ready channel closed before could receive signal");
        let server_result = join_handle.await?;
        match server_result {
          Ok(_) => {
            tracing::warn!("server completed successfully but ready signal was not sent - this should not happen");
            return Err(ServeError::Unknown);
          }
          Err(err) => return Err(err),
        }
      }
    };
    Ok(ServerShutdownHandle {
      join_handle,
      shutdown,
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::{ServeCommand, ServeError, ServerError};
  use rstest::rstest;
  use services::test_utils::temp_dir;
  use services::test_utils::AppServiceStubBuilder;
  use std::sync::Arc;
  use tempfile::TempDir;
  use tokio::net::TcpListener;

  #[rstest]
  #[tokio::test]
  async fn test_server_fails_when_port_already_in_use(temp_dir: TempDir) -> anyhow::Result<()> {
    // Bind to a random port first to ensure it's in use (use high ports to avoid permission issues)
    let port = 8000 + (rand::random::<u16>() % 1000); // Use ports 8000-8999
    let host = "127.0.0.1";
    let addr = format!("{}:{}", host, port);
    let _listener = TcpListener::bind(&addr).await?;
    let app_service = Arc::new(
      AppServiceStubBuilder::default()
        .with_temp_home_as(temp_dir)
        .build()
        .await
        .unwrap(),
    );
    let serve_command = ServeCommand::ByParams {
      host: host.to_string(),
      port,
    };
    let result = serve_command.get_server_handle(app_service, None).await;
    assert!(result.is_err());
    match result.unwrap_err() {
      ServeError::Server(ServerError::Io(_)) => {
        // This is expected - binding to port should fail
      }
      other => panic!("Expected IO error for port conflict, got: {:?}", other),
    }
    Ok(())
  }
}
