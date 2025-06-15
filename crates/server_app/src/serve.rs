use crate::{
  build_server_handle, shutdown_signal, ServerError, ServerHandle, ServerKeepAlive,
  ShutdownCallback, TaskJoinError, VariantChangeListener,
};
use axum::Router;
use include_dir::Dir;
use llama_server_proc::exec_path_from;
use objs::{impl_error_from, AppError, SettingSource};
use routes_all::build_routes;
use server_core::{ContextError, DefaultSharedContext, SharedContext};
use services::{AppService, SettingServiceError, BODHI_HOST, BODHI_PORT};
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::oneshot::Sender, task::JoinHandle};
use tower_serve_static::ServeDir;

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
    let host_source = if let Some(serde_yaml::Value::String(default_host)) =
      setting_service.get_default_value(BODHI_HOST)
    {
      if *host == default_host {
        SettingSource::Default
      } else {
        SettingSource::CommandLine
      }
    } else {
      SettingSource::CommandLine
    };
    setting_service.set_setting_with_source(
      BODHI_HOST,
      &serde_yaml::Value::String(host.to_string()),
      host_source,
    );
    let port_source = if let Some(serde_yaml::Value::Number(default_port)) =
      setting_service.get_default_value(BODHI_PORT)
    {
      if Some((*port) as u64) == default_port.as_u64() {
        SettingSource::Default
      } else {
        SettingSource::CommandLine
      }
    } else {
      SettingSource::CommandLine
    };
    setting_service.set_setting_with_source(
      BODHI_PORT,
      &serde_yaml::Value::Number((*port).into()),
      port_source,
    );
    let ServerHandle {
      server,
      shutdown,
      ready_rx,
    } = build_server_handle(host, *port);

    let exec_variant = service.setting_service().exec_variant();
    let exec_lookup_path = PathBuf::from(service.setting_service().exec_lookup_path());
    let exec_path = exec_path_from(&exec_lookup_path, &exec_variant);
    if !exec_path.exists() {
      println!("exec not found at {}", exec_path.to_string_lossy());
      return Err(ContextError::ExecNotExists(
        exec_path.to_string_lossy().to_string(),
      ))?;
    }
    let ctx = DefaultSharedContext::new(service.hub_service(), &exec_lookup_path, &exec_variant);
    let ctx: Arc<dyn SharedContext> = Arc::new(ctx);
    setting_service.add_listener(Arc::new(VariantChangeListener::new(ctx.clone())));

    let keep_alive = Arc::new(ServerKeepAlive::new(
      ctx.clone(),
      setting_service.keep_alive(),
    ));
    setting_service.add_listener(keep_alive.clone());
    ctx.add_state_listener(keep_alive).await;

    // Create static router from directory if provided
    let static_router = static_dir.map(|dir| {
      let static_service = ServeDir::new(dir).append_index_html_on_directories(true);
      Router::new().fallback_service(static_service)
    });

    let app = build_routes(ctx.clone(), service, static_router);

    let join_handle: JoinHandle<std::result::Result<(), ServeError>> = tokio::spawn(async move {
      let callback = Box::new(ShutdownContextCallback { ctx });
      match server.start_new(app, Some(callback)).await {
        Ok(()) => {
          tracing::info!("server started");
          Ok(())
        }
        Err(err) => {
          tracing::error!(err = ?err, "server encountered an error");
          Err(err)?
        }
      }
    });
    match ready_rx.await {
      Ok(()) => {
        println!("server started on http://{host}:{port}");
        tracing::info!(addr = format!("{host}:{port}"), "server started");
      }
      Err(err) => tracing::warn!(?err, "ready channel closed before could receive signal"),
    }
    Ok(ServerShutdownHandle {
      join_handle,
      shutdown,
    })
  }
}
