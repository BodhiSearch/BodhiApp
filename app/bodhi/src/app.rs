use crate::{native::NativeCommand, AppError};
use axum::Router;
use bodhicore::server::{run::RunCommand, serve::ServeCommand};
use clap::Parser;
use commands::{
  Cli, Command, CreateCommand, DefaultStdoutWriter, EnvCommand, ListCommand, ManageAliasCommand,
  PullCommand,
};
use include_dir::{include_dir, Dir};
use services::{
  db::{DbPool, DbService, SqliteDbService, TimeService},
  AppService, EnvService, EnvServiceFn, HfHubService, KeycloakAuthService, KeyringSecretService,
  LocalDataService, MokaCacheService, SqliteSessionService,
};
use std::{env, path::Path, sync::Arc};
use tokio::runtime::Builder;
use tower_serve_static::ServeDir;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../out");

pub fn main_internal(env_service: Arc<EnvService>) -> super::Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build()?;
  runtime.block_on(async move { aexecute(env_service).await })
}

async fn aexecute(env_service: Arc<EnvService>) -> super::Result<()> {
  let bodhi_home = env_service.bodhi_home();
  let hf_cache = env_service.hf_cache();
  let data_service = LocalDataService::new(bodhi_home.clone());
  let hub_service = HfHubService::new_from_hf_cache(hf_cache, true);
  let app_suffix = if env_service.is_production() {
    ""
  } else {
    " - Dev"
  };
  let app_name = format!("Bodhi App{app_suffix}");
  let secret_service = KeyringSecretService::with_service_name(app_name);

  let dbpath = env_service.db_path();
  let pool = DbPool::connect(&format!("sqlite:{}", dbpath.display())).await?;
  let db_service = SqliteDbService::new(pool.clone(), Arc::new(TimeService));
  db_service.migrate().await?;
  let session_service = SqliteSessionService::new(pool);
  session_service.migrate().await?;
  let cache_service = MokaCacheService::default();

  let auth_url = env_service.auth_url();
  let auth_realm = env_service.auth_realm();
  let auth_service = KeycloakAuthService::new(auth_url, auth_realm);

  let app_service = AppService::new(
    env_service.clone(),
    Arc::new(hub_service),
    Arc::new(data_service),
    Arc::new(auth_service),
    Arc::new(db_service),
    Arc::new(session_service),
    Arc::new(secret_service),
    Arc::new(cache_service),
  );
  let service = Arc::new(app_service);

  let args = env::args().collect::<Vec<_>>();
  if args.len() == 1
    && args
      .first()
      .ok_or_else(|| AppError::Unreachable("already checked the length is 1".to_string()))?
      .contains(".app/Contents/MacOS/")
  {
    // the app was launched using Bodhi.app, launch the native app with system tray
    NativeCommand::new(service, true)
      .aexecute(Some(static_router()))
      .await?;
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
      NativeCommand::new(service, ui)
        .aexecute(Some(static_router()))
        .await?;
    }
    list @ Command::List { .. } => {
      let list_command = ListCommand::try_from(list)?;
      list_command.execute(service)?;
    }
    serve @ Command::Serve { .. } => {
      let serve_command = ServeCommand::try_from(serve)?;
      match &serve_command {
        cmd @ ServeCommand::ByParams { host, port } => {
          env_service.set_host(host);
          env_service.set_port(*port);
          cmd.aexecute(service, None).await?;
        }
      }
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
      run_command.aexecute(service).await?;
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
