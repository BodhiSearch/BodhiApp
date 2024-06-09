use super::{CliError, Command};
use crate::{
  db::{DbPool, DbService, TimeService},
  error::Common,
  server::{build_routes, build_server_handle, shutdown_signal, ServerHandle},
  service::{AppServiceFn, PROD_DB},
  BodhiError, SharedContextRw, SharedContextRwFn,
};
use futures_util::{future::BoxFuture, FutureExt};
use std::{fs::File, path::PathBuf, sync::Arc};
use tokio::runtime::Builder;

#[derive(Debug, Clone, PartialEq)]
pub enum ServeCommand {
  ByParams { host: String, port: u16 },
}

impl TryFrom<Command> for ServeCommand {
  type Error = CliError;

  fn try_from(value: Command) -> Result<Self, Self::Error> {
    match value {
      Command::Serve { host, port } => Ok(ServeCommand::ByParams { host, port }),
      cmd => Err(CliError::ConvertCommand(
        cmd.to_string(),
        "serve".to_string(),
      )),
    }
  }
}

impl ServeCommand {
  pub fn execute(
    &self,
    service: Arc<dyn AppServiceFn>,
    bodhi_home: PathBuf,
  ) -> crate::error::Result<()> {
    match self {
      ServeCommand::ByParams { host, port } => {
        self.execute_by_params(host, *port, service, bodhi_home)?;
      }
    }
    Ok(())
  }

  fn execute_by_params(
    &self,
    host: &str,
    port: u16,
    service: Arc<dyn AppServiceFn>,
    bodhi_home: PathBuf,
  ) -> crate::error::Result<()> {
    let runtime = Builder::new_multi_thread()
      .enable_all()
      .build()
      .map_err(Common::from)?;
    runtime.block_on(async move {
      self
        .aexecute_by_params(host, port, service, bodhi_home)
        .await?;
      Ok::<(), BodhiError>(())
    })?;
    Ok(())
  }

  async fn aexecute_by_params(
    &self,
    host: &str,
    port: u16,
    service: Arc<dyn AppServiceFn>,
    bodhi_home: PathBuf,
  ) -> crate::error::Result<()> {
    let dbpath = bodhi_home.join(PROD_DB);
    _ = File::create_new(&dbpath);
    let pool = DbPool::connect(&format!("sqlite:{}", dbpath.display())).await?;
    let db_service = DbService::new(pool, Arc::new(TimeService));

    let ServerHandle {
      server,
      shutdown,
      ready_rx: _ready_rx,
    } = build_server_handle(host, port);

    let ctx = SharedContextRw::new_shared_rw(None).await?;
    let ctx: Arc<dyn SharedContextRwFn> = Arc::new(ctx);

    let app = build_routes(ctx.clone(), service, Arc::new(db_service));

    let server_async = tokio::spawn(async move {
      let callback: Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + 'static> = Box::new(|| {
        async move {
          if let Err(err) = ctx.try_stop().await {
            tracing::warn!(err = ?err, "error stopping llama context");
          }
        }
        .boxed()
      });
      match server.start_new(app, Some(callback)).await {
        Ok(()) => Ok(()),
        Err(err) => {
          tracing::error!(err = ?err, "server encountered an error");
          Err(err)
        }
      }
    });
    tokio::spawn(async move {
      shutdown_signal().await;
      shutdown
        .send(())
        .map_err(|_| Common::Sender("shutdown".to_string()))
        .unwrap();
    });
    (server_async.await.map_err(Common::Join)?)?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::{Command, ServeCommand};
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
      "Command 'list' cannot be converted into command 'serve'",
      result.unwrap_err().to_string()
    );
    Ok(())
  }
}
