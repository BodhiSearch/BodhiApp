use crate::native::main_native;
use anyhow::{anyhow, Context};
use bodhicore::{
  cli::{Cli, Command},
  home::logs_dir,
  server::{
    build_routes, build_server_handle, shutdown_signal, ServerHandle, SharedContextRw,
    SharedContextRwExts,
  },
  AppService, CreateCommand, List, PullCommand, RunCommand, Serve,
};
use clap::Parser;
use futures_util::{future::BoxFuture, FutureExt};
use std::{env, sync::Arc};
use tokio::runtime::Builder;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn main_internal() -> anyhow::Result<()> {
  let args = env::args().collect::<Vec<_>>();
  if args.len() == 1
    && args
      .first()
      .ok_or_else(|| anyhow!("already checked the length is 1"))?
      .contains(".app/Contents/MacOS/")
  {
    // the app was launched using Bodhi.app, launch the native app with system tray
    return main_native();
  }
  // the app was called from wrapper
  // or the executable was called from outside the `Bodhi.app` bundle
  let cli = Cli::parse();
  let service = AppService::default();
  match cli.command {
    Command::App {} => {
      main_native()?;
    }
    Command::Init {} => {
      unimplemented!()
    }
    Command::List { remote, models } => {
      List::new(remote, models).execute(&service)?;
    }
    Command::Serve { host, port } => {
      main_async(Serve { host, port })?;
    }
    pull @ Command::Pull { .. } => {
      let pull_command: PullCommand = pull.try_into()?;
      pull_command.execute(&service)?;
    }
    create @ Command::Create { .. } => {
      let create_command: CreateCommand = create.try_into()?;
      create_command.execute(&service)?;
    }
    run @ Command::Run { .. } => {
      let run_command: RunCommand = run.try_into()?;
      run_command.execute(&service)?;
    }
  }
  Ok(())
}

pub fn setup_logs() -> anyhow::Result<WorkerGuard> {
  let file_appender = tracing_appender::rolling::daily(logs_dir()?, "bodhi.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let filter = filter.add_directive("hf_hub=error".parse().unwrap());
  tracing_subscriber::registry()
    .with(filter)
    .with(fmt::layer().with_writer(non_blocking))
    .init();
  Ok(guard)
}

fn main_async(serve: Serve) -> anyhow::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build();
  match runtime {
    Ok(runtime) => runtime.block_on(async move { main_server(serve).await }),
    Err(err) => Err(err.into()),
  }
}

async fn main_server(serve: Serve) -> anyhow::Result<()> {
  let ServerHandle {
    server,
    shutdown,
    ready_rx: _ready_rx,
  } = build_server_handle(serve.clone().into())?;
  let mut ctx = SharedContextRw::new_shared_rw(None).await?;
  let service = AppService::default();
  let app = build_routes(ctx.clone(), Arc::new(service));
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
      .map_err(|_| anyhow::anyhow!("error sending shutdown signal on channel"))
      .context("sending shutdown signal to server")
      .unwrap();
  });
  (server_async.await?)?;
  Ok(())
}
