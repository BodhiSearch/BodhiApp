use anyhow::Context;
use bodhi::cli::{Cli, Command};
use bodhi::{build_server_handle, server::ServerHandle, shutdown_signal, ServerParams};
use clap::Parser;
use llama_server_bindings::GptParams;
use tokio::runtime::Builder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn main() {
  dotenv::dotenv().ok();
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .init();
  let result = main_internal();
  if let Err(err) = result {
    tracing::warn!(err = ?err, "application exited with error");
    std::process::exit(1);
  }
}

fn main_internal() -> anyhow::Result<()> {
  let cli = Cli::parse();
  match cli.command {
    cli_args @ Command::Serve { .. } => {
      let (server_params, gpt_params) = cli_args.to_params()?;
      main_async(server_params, gpt_params)?;
    }
  }
  Ok(())
}

fn main_async(server_params: ServerParams, gpt_params: GptParams) -> anyhow::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build();
  match runtime {
    Ok(runtime) => runtime.block_on(async move { main_server(server_params, gpt_params).await }),
    Err(err) => Err(err.into()),
  }
}

async fn main_server(server_params: ServerParams, gpt_params: GptParams) -> anyhow::Result<()> {
  let ServerHandle {
    server,
    shutdown,
    ready_rx: _ready_rx,
  } = build_server_handle(server_params)?;
  let server_async = tokio::spawn(async move {
    match server.start(gpt_params).await {
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
  (server_async.await?)?;
  Ok(())
}
