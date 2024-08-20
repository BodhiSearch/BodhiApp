use crate::{native::NativeCommand, AppError};
use axum::Router;
use bodhicore::{
  cli::{Cli, Command, ServeCommand},
  service::{
    AppService, AppServiceBuilder, EnvService, EnvServiceFn, HfHubService, LocalDataService,
  },
  CreateCommand, DefaultStdoutWriter, EnvCommand, ListCommand, ManageAliasCommand, PullCommand,
  RunCommand,
};
use clap::Parser;
use include_dir::{include_dir, Dir};
use std::{env, path::Path, sync::Arc};
use tower_serve_static::ServeDir;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../out");

pub fn main_internal(env_service: Arc<EnvService>) -> super::Result<()> {
  let bodhi_home = env_service.bodhi_home();
  let hf_cache = env_service.hf_cache();
  let data_service = LocalDataService::new(bodhi_home);
  let hub_service = HfHubService::new_from_hf_cache(hf_cache, true);
  // new(env_service, hub_service, data_service, auth_service);
  let app_service = AppServiceBuilder::default()
    .env_service(env_service)
    .hub_service(Arc::new(hub_service))
    .data_service(Arc::new(data_service))
    .build()?;
  let service = Arc::new(app_service);

  let args = env::args().collect::<Vec<_>>();
  if args.len() == 1
    && args
      .first()
      .ok_or_else(|| AppError::Unreachable("already checked the length is 1".to_string()))?
      .contains(".app/Contents/MacOS/")
  {
    // the app was launched using Bodhi.app, launch the native app with system tray
    NativeCommand::new(service, true).execute(Some(static_router()))?;
    return Ok(());
  }

  // the app was called from wrapper
  // or the executable was called from outside the `Bodhi.app` bundle
  let cli = Cli::parse();
  match cli.command {
    Command::Envs {} => {
      EnvCommand::new(service).execute()?;
    }
    Command::App { ui } => {
      NativeCommand::new(service, ui).execute(Some(static_router()))?;
    }
    list @ Command::List { .. } => {
      let list_command = ListCommand::try_from(list)?;
      list_command.execute(service)?;
    }
    serve @ Command::Serve { .. } => {
      let serve_command = ServeCommand::try_from(serve)?;
      serve_command.execute(service)?;
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
    show @ Command::Show { .. } => {
      let show = ManageAliasCommand::try_from(show)?;
      show.execute(service, &mut DefaultStdoutWriter::default())?;
    }
    cp @ Command::Cp { .. } => {
      let cp = ManageAliasCommand::try_from(cp)?;
      cp.execute(service, &mut DefaultStdoutWriter::default())?;
    }
    edit @ Command::Edit { .. } => {
      let edit = ManageAliasCommand::try_from(edit)?;
      edit.execute(service, &mut DefaultStdoutWriter::default())?;
    }
    rm @ Command::Rm { .. } => {
      let rm = ManageAliasCommand::try_from(rm)?;
      rm.execute(service, &mut DefaultStdoutWriter::default())?;
    }
  }
  Ok(())
}

pub fn setup_logs(logs_dir: &Path) -> super::Result<WorkerGuard> {
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

fn static_router() -> Router {
  let static_service = ServeDir::new(&ASSETS).append_index_html_on_directories(true);
  Router::new().fallback_service(static_service)
}
