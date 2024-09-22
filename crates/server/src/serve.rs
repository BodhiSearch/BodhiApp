use crate::{
  build_routes, build_server_handle, shutdown_signal, BodhiError, DefaultSharedContextRw,
  ServerHandle, SharedContextRw, ShutdownCallback,
};
use axum::Router;
use commands::{CmdIntoError, Command};
use services::AppService;
use std::sync::Arc;
use tokio::{sync::oneshot::Sender, task::JoinHandle};

#[derive(Debug, Clone, PartialEq)]
pub enum ServeCommand {
  ByParams { host: String, port: u16 },
}

impl TryFrom<Command> for ServeCommand {
  type Error = CmdIntoError;

  fn try_from(value: Command) -> Result<Self, Self::Error> {
    match value {
      Command::Serve { host, port } => Ok(ServeCommand::ByParams { host, port }),
      cmd => Err(CmdIntoError::Convert {
        input: cmd.to_string(),
        output: "ServeCommand".to_string(),
      }),
    }
  }
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
  join_handle: JoinHandle<Result<(), BodhiError>>,
  shutdown: Sender<()>,
}

impl ServerShutdownHandle {
  pub async fn shutdown_on_ctrlc(self) -> crate::error::Result<()> {
    shutdown_signal().await;
    self.shutdown().await?;
    Ok(())
  }

  pub async fn shutdown(self) -> crate::error::Result<()> {
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
  ) -> crate::error::Result<()> {
    let handle = self.get_server_handle(service, static_router).await?;
    handle.shutdown_on_ctrlc().await?;
    Ok::<(), BodhiError>(())
  }

  // TODO: move this to another module that returns a handle when passed server components
  pub async fn get_server_handle(
    &self,
    service: Arc<dyn AppService>,
    static_router: Option<Router>,
  ) -> crate::error::Result<ServerShutdownHandle> {
    let ServeCommand::ByParams { host, port } = self;
    let ServerHandle {
      server,
      shutdown,
      ready_rx,
    } = build_server_handle(host, *port);

    let ctx = DefaultSharedContextRw::new_shared_rw(None).await?;
    let ctx: Arc<dyn SharedContextRw> = Arc::new(ctx);
    let app = build_routes(ctx.clone(), service, static_router);

    let join_handle = tokio::spawn(async move {
      let callback = Box::new(ShutdownContextCallback { ctx });
      match server.start_new(app, Some(callback)).await {
        Ok(()) => Ok(()),
        Err(err) => {
          tracing::error!(err = ?err, "server encountered an error");
          Err(err)
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

#[cfg(test)]
mod test {
  use crate::ServeCommand;
  use commands::Command;
  use rstest::rstest;

  #[rstest]
  fn test_serve_command_from_serve() -> anyhow::Result<()> {
    let cmd = Command::Serve {
      host: "localhost".to_string(),
      port: 1135,
    };
    let result = ServeCommand::try_from(cmd)?;
    let expected = ServeCommand::ByParams {
      host: "localhost".to_string(),
      port: 1135,
    };
    assert_eq!(expected, result);
    Ok(())
  }

  #[rstest]
  fn test_serve_command_convert_err() -> anyhow::Result<()> {
    let cmd = Command::List {
      remote: false,
      models: false,
    };
    let result = ServeCommand::try_from(cmd);
    assert!(result.is_err());
    assert_eq!(
      "Command 'list' cannot be converted into command 'ServeCommand'",
      result.unwrap_err().to_string()
    );
    Ok(())
  }
}
