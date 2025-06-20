use crate::app::AppCommand;
use lib_bodhiserver::{
  build_app_service, setup_app_dirs, AppType, ErrorMessage, ErrorType, ServeCommand,
  SettingService, BODHI_LOG_STDOUT,
};
use std::sync::Arc;
use tokio::runtime::Builder;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Server-specific constants
#[cfg(feature = "production")]
mod env_config {
  use lib_bodhiserver::{DefaultSettingService, ErrorMessage};

  pub fn set_feature_settings(
    _setting_service: &DefaultSettingService,
  ) -> Result<(), ErrorMessage> {
    Ok(())
  }
}

#[cfg(not(feature = "production"))]
mod env_config {
  use lib_bodhiserver::{
    DefaultSettingService, ErrorMessage, SettingService, BODHI_EXEC_LOOKUP_PATH,
  };

  #[allow(clippy::result_large_err)]
  pub fn set_feature_settings(setting_service: &DefaultSettingService) -> Result<(), ErrorMessage> {
    setting_service.set_default(
      BODHI_EXEC_LOOKUP_PATH,
      &serde_yaml::Value::String(concat!(env!("CARGO_MANIFEST_DIR"), "/bin").to_string()),
    );
    Ok(())
  }
}

use crate::common::build_app_options;
use env_config::*;

const APP_TYPE: AppType = AppType::Container;

pub fn initialize_and_execute(command: AppCommand) -> Result<(), ErrorMessage> {
  let app_options = build_app_options(APP_TYPE)?;
  let setting_service = setup_app_dirs(&app_options)?;
  set_feature_settings(&setting_service)?;

  // Server mode uses file-based logging
  let _guard = setup_logs(&setting_service);

  let result = aexecute(Arc::new(setting_service), command);

  drop(_guard);
  result
}

fn aexecute(
  setting_service: Arc<dyn SettingService>,
  command: AppCommand,
) -> Result<(), ErrorMessage> {
  let runtime = Builder::new_multi_thread()
    .enable_all()
    .build()
    .map_err(crate::error::AppSetupError::from)?;
  let result: Result<(), ErrorMessage> = runtime.block_on(async move {
    // Build the complete app service using the lib_bodhiserver function
    let app_service = Arc::new(build_app_service(setting_service.clone()).await?);

    // Handle server commands based on the parsed command
    let command = match command {
      AppCommand::Server(host, port) => {
        // Server deployment mode with specific host/port
        ServeCommand::ByParams { host, port }
      }
      AppCommand::Default => {
        // Default server mode - use settings from configuration
        let host = setting_service.host();
        let port = setting_service.port();
        ServeCommand::ByParams { host, port }
      }
    };
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

fn setup_logs(setting_service: &lib_bodhiserver::DefaultSettingService) -> WorkerGuard {
  let logs_dir = setting_service.logs_dir();
  let file_appender = tracing_appender::rolling::daily(logs_dir, "bodhi.log");
  let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
  let log_level: LevelFilter = setting_service.log_level().into();
  let log_level = log_level.to_string();
  let filter = EnvFilter::new(&log_level);
  let filter = filter.add_directive("hf_hub=error".parse().expect("is a valid directive"));

  // Check if we should output to stdout
  let enable_stdout = cfg!(debug_assertions)
    || setting_service
      .get_setting(BODHI_LOG_STDOUT)
      .map(|v| v == "1" || v.to_lowercase() == "true")
      .unwrap_or(false);

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
  guard
}
