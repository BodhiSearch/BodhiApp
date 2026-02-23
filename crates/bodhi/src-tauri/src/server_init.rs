use crate::app::AppCommand;
use lib_bodhiserver::{
  build_app_service, setup_app_dirs, setup_bootstrap_service, AppService, AppType,
  BootstrapService, ErrorMessage, ErrorType, ServeCommand,
};
use std::fs;
use std::sync::Arc;
use tokio::runtime::Builder;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(feature = "production")]
mod env_config {
  use lib_bodhiserver::{ErrorMessage, SettingService};

  pub async fn set_feature_settings(
    _setting_service: &dyn SettingService,
  ) -> Result<(), ErrorMessage> {
    Ok(())
  }
}

#[cfg(not(feature = "production"))]
mod env_config {
  use lib_bodhiserver::{ErrorMessage, ErrorType, SettingService, BODHI_EXEC_LOOKUP_PATH};

  #[allow(clippy::result_large_err)]
  pub async fn set_feature_settings(
    setting_service: &dyn SettingService,
  ) -> Result<(), ErrorMessage> {
    if setting_service
      .get_setting(BODHI_EXEC_LOOKUP_PATH)
      .await
      .is_none()
    {
      setting_service
        .set_default(
          BODHI_EXEC_LOOKUP_PATH,
          &serde_yaml::Value::String(concat!(env!("CARGO_MANIFEST_DIR"), "/bin").to_string()),
        )
        .await
        .map_err(|e| {
          ErrorMessage::new(
            "setting_service_error".to_string(),
            ErrorType::InternalServer.to_string(),
            e.to_string(),
          )
        })?;
    }
    Ok(())
  }
}

use crate::common::build_app_options;
use env_config::*;

const APP_TYPE: AppType = AppType::Container;

pub fn initialize_and_execute(command: AppCommand) -> Result<(), ErrorMessage> {
  let app_options = build_app_options(APP_TYPE)?;
  let (bodhi_home, source, file_defaults) = setup_app_dirs(&app_options)?;
  let bootstrap =
    setup_bootstrap_service(&app_options, bodhi_home, source, file_defaults, command)?;

  let _guard = setup_logs(&bootstrap).map_err(crate::error::AppSetupError::from)?;
  let parts = bootstrap.into_parts();

  let runtime = Builder::new_multi_thread()
    .enable_all()
    .build()
    .map_err(crate::error::AppSetupError::from)?;
  let result: Result<(), ErrorMessage> = runtime.block_on(async move {
    let app_service = Arc::new(build_app_service(parts).await?);
    set_feature_settings(app_service.setting_service().as_ref()).await?;
    let host = app_service.setting_service().host().await;
    let port = app_service.setting_service().port().await;
    let command = ServeCommand::ByParams { host, port };
    command
      .aexecute(app_service, Some(&crate::ui::ASSETS))
      .await
      .map_err(|err| {
        ErrorMessage::new(
          "serve_error".to_string(),
          ErrorType::InternalServer.to_string(),
          err.to_string(),
        )
      })?;
    tracing::info!("application exited with success");
    Ok(())
  });
  result
}

fn setup_logs(bootstrap_service: &BootstrapService) -> Result<WorkerGuard, std::io::Error> {
  let logs_dir = bootstrap_service.logs_dir();
  fs::create_dir_all(&logs_dir)?;
  let file_appender = tracing_appender::rolling::daily(logs_dir, "bodhi.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  let log_level: LevelFilter = bootstrap_service.log_level().into();
  let log_level = log_level.to_string();
  let filter = EnvFilter::new(&log_level);
  let filter = filter.add_directive("hf_hub=error".parse().expect("is a valid directive"));
  let filter = filter.add_directive("tower_sessions=warn".parse().expect("is a valid directive"));
  let filter = filter.add_directive("tower_http=warn".parse().expect("is a valid directive"));
  let filter = filter.add_directive(
    "tower_sessions_core=warn"
      .parse()
      .expect("is a valid directive"),
  );

  let enable_stdout = cfg!(debug_assertions) || bootstrap_service.log_stdout();

  let subscriber = tracing_subscriber::registry().with(filter);

  if enable_stdout {
    subscriber
      .with(
        fmt::layer()
          .with_writer(std::io::stdout)
          .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
          .with_target(true),
      )
      .with(
        fmt::layer()
          .with_writer(non_blocking)
          .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
          .with_target(true),
      )
      .init();
  } else {
    subscriber
      .with(
        fmt::layer()
          .with_writer(non_blocking)
          .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::CLOSE)
          .with_target(true),
      )
      .init();
  }
  #[cfg(debug_assertions)]
  {
    println!(
      "logging to stdout: {}, log_level: {}",
      enable_stdout, log_level
    );
  }
  Ok(guard)
}
