use crate::{native::main_native, AppError, PROD_DB};
use bodhicore::{
  cli::{Cli, Command},
  db::{DbPool, DbService, TimeService},
  home::{bodhi_home, logs_dir},
  server::{build_routes, build_server_handle, shutdown_signal, ServerHandle},
  AppService, BodhiError, CreateCommand, ListCommand, PullCommand, RunCommand, Serve,
  SharedContextRw, SharedContextRwFn,
};
use clap::Parser;
use futures_util::{future::BoxFuture, FutureExt};
use std::{env, fs::File, sync::Arc};
use tokio::runtime::Builder;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn main_internal() -> super::Result<()> {
  let args = env::args().collect::<Vec<_>>();
  if args.len() == 1
    && args
      .first()
      .ok_or_else(|| AppError::Unreachable("already checked the length is 1".to_string()))?
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
    list @ Command::List { .. } => {
      let list_command = ListCommand::try_from(list)?;
      list_command.execute(&service)?;
    }
    Command::Serve { host, port } => {
      main_async(Serve { host, port })?;
    }
    pull @ Command::Pull { .. } => {
      let pull_command = PullCommand::try_from(pull)?;
      pull_command.execute(&service)?;
    }
    create @ Command::Create { .. } => {
      let create_command = CreateCommand::try_from(create)?;
      create_command.execute(&service)?;
    }
    run @ Command::Run { .. } => {
      let run_command = RunCommand::try_from(run)?;
      run_command.execute(&service)?;
    }
  }
  Ok(())
}

pub fn setup_logs() -> super::Result<WorkerGuard> {
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

fn main_async(serve: Serve) -> super::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build();
  match runtime {
    Ok(runtime) => runtime.block_on(async move { main_server(serve).await }),
    Err(err) => Err(err.into()),
  }
}

async fn main_server(serve: Serve) -> super::Result<()> {
  let ServerHandle {
    server,
    shutdown,
    ready_rx: _ready_rx,
  } = build_server_handle(serve.clone().into());
  let ctx = SharedContextRw::new_shared_rw(None)
    .await
    .map_err(BodhiError::from)?;
  let ctx = Arc::new(ctx);
  let service = AppService::default();

  let dbpath = bodhi_home()?.join(PROD_DB);
  _ = File::create_new(&dbpath);
  let pool = DbPool::connect(&format!("sqlite:{}", dbpath.display())).await?;
  let db_service = DbService::new(pool, Arc::new(TimeService));

  let app = build_routes(ctx.clone(), Arc::new(service), Arc::new(db_service));
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
      .map_err(|_| AppError::Any("error sending shutdown signal on channel".to_string()))
      .unwrap();
  });
  (server_async
    .await
    .map_err(|err| AppError::Any(err.to_string()))?)?;
  Ok(())
}
