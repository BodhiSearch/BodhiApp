use crate::{native::NativeCommand, AppError};
use bodhicore::{
  cli::{Cli, Command, ServeCommand},
  service::{AppService, HfHubService, LocalDataService},
  CreateCommand, ListCommand, PullCommand, RunCommand,
};
use clap::Parser;
use std::{
  env,
  path::{Path, PathBuf},
  sync::Arc,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn main_internal(bodhi_home: PathBuf, hf_cache: PathBuf) -> super::Result<()> {
  let data_service = LocalDataService::new(bodhi_home.clone());
  let hub_service = HfHubService::new_from_hf_cache(hf_cache.clone(), true);
  let service = Arc::new(AppService::new(hub_service, data_service));

  let args = env::args().collect::<Vec<_>>();
  if args.len() == 1
    && args
      .first()
      .ok_or_else(|| AppError::Unreachable("already checked the length is 1".to_string()))?
      .contains(".app/Contents/MacOS/")
  {
    // the app was launched using Bodhi.app, launch the native app with system tray
    NativeCommand::new(service, bodhi_home).execute()?;
    return Ok(());
  }

  // the app was called from wrapper
  // or the executable was called from outside the `Bodhi.app` bundle
  let cli = Cli::parse();
  match cli.command {
    Command::App {} => {
      NativeCommand::new(service, bodhi_home).execute()?;
    }
    list @ Command::List { .. } => {
      let list_command = ListCommand::try_from(list)?;
      list_command.execute(service)?;
    }
    serve @ Command::Serve { .. } => {
      let serve_command = ServeCommand::try_from(serve)?;
      serve_command.execute(service, bodhi_home)?;
    }
    pull @ Command::Pull { .. } => {
      let pull_command = PullCommand::try_from(pull)?;
      pull_command.execute(service)?;
    }
    create @ Command::Create { .. } => {
      let create_command = CreateCommand::try_from(create)?;
      create_command.execute(service)?;
    }
    run @ Command::Run { .. } => {
      let run_command = RunCommand::try_from(run)?;
      run_command.execute(service)?;
    }
  }
  Ok(())
}

pub fn setup_logs(bodhi_home: &Path) -> super::Result<WorkerGuard> {
  let logs_dir = bodhi_home.join("logs");
  std::fs::create_dir_all(&logs_dir)?;
  let file_appender = tracing_appender::rolling::daily(logs_dir, "bodhi.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
  let filter = filter.add_directive("hf_hub=error".parse().unwrap());
  tracing_subscriber::registry()
    .with(filter)
    .with(fmt::layer().with_writer(non_blocking))
    .init();
  Ok(guard)
}
