use crate::{
  build_server_handle, shutdown_signal, ServerError, ServerHandle, ShutdownCallback, TaskJoinError,
  VariantChangeListener,
};
use include_dir::Dir;
use routes_app::build_routes;
use services::{impl_error_from, AppError, ErrorType};
use services::{AppService, SettingServiceError, BODHI_KEEP_ALIVE_SECS, DEFAULT_KEEP_ALIVE_SECS};
use services::{SettingSource, SettingsChangeListener};
use std::sync::Arc;
use tokio::{sync::oneshot::Sender, task::JoinHandle};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ServeError {
  #[error(transparent)]
  Setting(#[from] SettingServiceError),
  #[error(transparent)]
  Join(#[from] TaskJoinError),
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

pub struct ShutdownInferenceCallback {
  app_service: Arc<dyn AppService>,
}

#[async_trait::async_trait]
impl ShutdownCallback for ShutdownInferenceCallback {
  async fn shutdown(&self) {
    if let Err(err) = self.app_service.inference_service().stop().await {
      tracing::warn!(err = ?err, "error stopping inference service");
    }
  }
}

/// Listener that forwards keep-alive setting changes to InferenceService.
#[derive(Debug)]
struct KeepAliveSettingListener {
  app_service: Arc<dyn AppService>,
}

impl SettingsChangeListener for KeepAliveSettingListener {
  fn on_change(
    &self,
    key: &str,
    _prev_value: &Option<serde_yaml::Value>,
    _prev_source: &SettingSource,
    new_value: &Option<serde_yaml::Value>,
    _new_source: &SettingSource,
  ) {
    if key != BODHI_KEEP_ALIVE_SECS {
      return;
    }
    let new_keep_alive = new_value
      .as_ref()
      .and_then(|v| v.as_i64())
      .unwrap_or(DEFAULT_KEEP_ALIVE_SECS);
    let inference = self.app_service.inference_service();
    tokio::task::spawn(async move {
      inference.set_keep_alive(new_keep_alive).await;
    });
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

    // Register variant change and keep-alive listeners via InferenceService
    setting_service
      .add_listener(Arc::new(VariantChangeListener::new(
        service.inference_service(),
      )))
      .await;
    setting_service
      .add_listener(Arc::new(KeepAliveSettingListener {
        app_service: service.clone(),
      }))
      .await;

    let app = build_routes(service.clone(), static_dir).await;
    let scheme = setting_service.scheme().await;
    let server_url = format!("{scheme}://{host}:{port}");
    let public_url = setting_service.public_server_url().await;

    let join_handle: JoinHandle<std::result::Result<(), ServeError>> = tokio::spawn(async move {
      let callback = Box::new(ShutdownInferenceCallback {
        app_service: service,
      });
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
