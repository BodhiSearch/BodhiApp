use anyhow::Context;
use bodhiserver::cli::{Cli, Command};
use bodhiserver::{
  build_server_handle, port_from_env_vars, server::ServerHandle, shutdown_signal, ServerArgs,
  DEFAULT_HOST,
};
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static ENV_BODHISERVER_PORT: &str = "BODHISERVER_PORT";

pub fn main() {
  dotenv::dotenv().ok();
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .init();
  let result = start();
  if let Err(err) = result {
    tracing::warn!(err = ?err, "application exited with error");
    std::process::exit(1);
  }
}

fn start() -> anyhow::Result<()> {
  let cli = Cli::parse();
  match cli.command {
    Command::Serve { host, port, model } => serve(host, port, model),
  }
}

fn serve(host: Option<String>, port: Option<u16>, model: PathBuf) -> anyhow::Result<()> {
  let host = host.unwrap_or_else(|| String::from(DEFAULT_HOST));
  let port = port.unwrap_or_else(|| port_from_env_vars(std::env::var(ENV_BODHISERVER_PORT)));
  if !model.exists() {
    anyhow::bail!(format!(
      "model file does not exist at location: '{}'",
      model.display()
    ));
  }
  let server_args = ServerArgs {
    host,
    port,
    model,
    lazy_load_model: false,
  };
  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build();

  match runtime {
    Ok(runtime) => runtime.block_on(async move { start_server(server_args).await }),
    Err(err) => Err(err.into()),
  }
}

async fn start_server(server_args: ServerArgs) -> anyhow::Result<()> {
  let ServerHandle {
    server,
    shutdown,
    ready_rx: _ready_rx,
  } = build_server_handle(server_args)?;
  let server_join = tokio::spawn(async move {
    match server.start().await {
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
      .map_err(|_| anyhow::anyhow!("error sending shutdown signal on channel"))
      .context("sending shutdown signal to server")
      .unwrap();
  });
  (server_join.await?)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_serve_fails_if_model_does_not_exist() {
    let result = serve(None, None, PathBuf::from("non-existent-model"));
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().to_string(),
      "model file does not exist at location: 'non-existent-model'"
    );
  }
}
