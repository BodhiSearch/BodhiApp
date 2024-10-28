#[cfg(feature = "native")]
use crate::native;

use crate::{
  convert::{
    build_create_command, build_list_command, build_manage_alias_command, build_pull_command,
    build_run_command, build_serve_command,
  },
  error::{BodhiError, Result},
};
use clap::Parser;
use commands::{Cli, Command, DefaultStdoutWriter, EnvCommand};
use objs::FluentLocalizationService;
use services::{
  db::{DbPool, DbService, DefaultTimeService, SqliteDbService},
  DefaultAppService, DefaultEnvService, EnvService, HfHubService, KeycloakAuthService,
  KeyringSecretService, LocalDataService, MokaCacheService, SqliteSessionService,
};
use std::{env, path::Path, sync::Arc};
use tokio::runtime::Builder;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn main_internal(env_service: Arc<DefaultEnvService>) -> Result<()> {
  let runtime = Builder::new_multi_thread().enable_all().build()?;
  runtime.block_on(async move { aexecute(env_service).await })
}

async fn aexecute(env_service: Arc<DefaultEnvService>) -> Result<()> {
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
  let time_service = Arc::new(DefaultTimeService);
  let db_service = SqliteDbService::new(pool.clone(), time_service.clone());
  db_service.migrate().await?;
  let session_service = SqliteSessionService::new(pool);
  session_service.migrate().await?;
  let cache_service = MokaCacheService::default();

  let auth_url = env_service.auth_url();
  let auth_realm = env_service.auth_realm();
  let auth_service = KeycloakAuthService::new(auth_url, auth_realm);
  let localization_service = FluentLocalizationService::get_instance();
  localization_service
    .load_resource(objs::l10n::L10N_RESOURCES)?
    .load_resource(services::l10n::L10N_RESOURCES)?
    .load_resource(commands::l10n::L10N_RESOURCES)?
    .load_resource(server_core::l10n::L10N_RESOURCES)?
    .load_resource(auth_middleware::l10n::L10N_RESOURCES)?
    .load_resource(routes_oai::l10n::L10N_RESOURCES)?
    .load_resource(routes_app::l10n::L10N_RESOURCES)?
    .load_resource(routes_all::l10n::L10N_RESOURCES)?
    .load_resource(server_app::l10n::L10N_RESOURCES)?
    .load_resource(crate::l10n::L10N_RESOURCES)?;

  let app_service = DefaultAppService::new(
    env_service.clone(),
    Arc::new(hub_service),
    Arc::new(data_service),
    Arc::new(auth_service),
    Arc::new(db_service),
    Arc::new(session_service),
    Arc::new(secret_service),
    Arc::new(cache_service),
    localization_service,
    time_service,
  );
  let service = Arc::new(app_service);

  let args = env::args().collect::<Vec<_>>();
  if args.len() == 1 {
    if env_service.is_native() {
      if cfg!(feature = "native") {
        // the app was launched executing the executable, launch the native app with system tray
        #[cfg(feature = "native")]
        native::NativeCommand::new(service.clone(), true)
          .aexecute(Some(native::static_router()))
          .await?;
      } else {
        Err(BodhiError::Unreachable(format!(
          r#"env_service.is_native() returned true, but cfg!(feature = "native") is false"#
        )))?;
      }
    }
  }

  // the app was called from wrapper
  // or the executable was called from outside the `Bodhi.app` bundle
  let cli = Cli::parse();
  match cli.command {
    Command::Envs {} => {
      EnvCommand::new(service).execute()?;
    }
    Command::App { ui: _ui } => {
      if env_service.is_native() {
        if cfg!(feature = "native") {
          #[cfg(feature = "native")]
          native::NativeCommand::new(service, _ui)
            .aexecute(Some(native::static_router()))
            .await?;
        } else {
          Err(BodhiError::Unreachable(format!(
            r#"env_service.is_native() returned true, but cfg!(feature = "native") is false"#
          )))?;
        }
      } else {
        Err(BodhiError::NativeNotSupported)?;
      }
    }
    Command::Serve { host, port } => {
      let serve_command = build_serve_command(host, port)?;
      serve_command.aexecute(service, None).await?;
    }
    Command::List { remote, models } => {
      let list_command = build_list_command(remote, models)?;
      list_command.execute(service)?;
    }
    Command::Pull {
      alias,
      repo,
      filename,
      snapshot,
    } => {
      let pull_command = build_pull_command(alias, repo, filename, snapshot)?;
      pull_command.execute(service)?;
    }
    cmd @ Command::Create { .. } => {
      let create_command = build_create_command(cmd)?;
      create_command.execute(service)?;
    }
    Command::Run { alias } => {
      let run_command = build_run_command(alias)?;
      run_command.aexecute(service).await?;
    }
    cmd @ (Command::Show { .. }
    | Command::Cp { .. }
    | Command::Edit { .. }
    | Command::Rm { .. }) => {
      let manage_alias_command = build_manage_alias_command(cmd)?;
      manage_alias_command.execute(service, &mut DefaultStdoutWriter::default())?;
    }
  }
  Ok(())
}

pub fn setup_logs(logs_dir: &Path) -> Result<WorkerGuard> {
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
