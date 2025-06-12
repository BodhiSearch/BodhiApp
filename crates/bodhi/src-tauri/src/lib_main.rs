use crate::{app::start, error::AppSetupError};
use lib_bodhiserver::{build_app_service, setup_app_dirs, AppOptionsBuilder};
use objs::{AppType, ErrorMessage, ErrorType, OpenAIApiError};
use services::{DefaultEnvWrapper, SettingService};
use std::sync::Arc;
use tokio::runtime::Builder;

#[cfg(feature = "production")]
mod env_config {
  use objs::{EnvType, ErrorMessage};
  use services::DefaultSettingService;

  pub static ENV_TYPE: EnvType = EnvType::Production;
  pub static AUTH_URL: &str = "https://id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";

  pub fn set_feature_settings(
    _setting_service: &DefaultSettingService,
  ) -> Result<(), ErrorMessage> {
    Ok(())
  }
}

#[cfg(not(feature = "production"))]
mod env_config {
  use objs::{EnvType, ErrorMessage};
  use services::DefaultSettingService;

  pub static ENV_TYPE: EnvType = EnvType::Development;
  pub static AUTH_URL: &str = "https://dev-id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";

  #[cfg(not(feature = "native"))]
  #[allow(clippy::result_large_err)]
  pub fn set_feature_settings(setting_service: &DefaultSettingService) -> Result<(), ErrorMessage> {
    use services::{SettingService, BODHI_EXEC_LOOKUP_PATH};

    setting_service.set_default(
      BODHI_EXEC_LOOKUP_PATH,
      &serde_yaml::Value::String(concat!(env!("CARGO_MANIFEST_DIR"), "/bin").to_string()),
    );
    Ok(())
  }
  #[cfg(feature = "native")]
  #[allow(clippy::result_large_err)]
  pub fn set_feature_settings(
    _setting_service: &DefaultSettingService,
  ) -> Result<(), ErrorMessage> {
    Ok(())
  }
}

pub use env_config::*;

#[cfg(feature = "native")]
pub const APP_TYPE: AppType = AppType::Native;

#[cfg(not(feature = "native"))]
pub const APP_TYPE: AppType = AppType::Container;

pub fn _main() {
  if let Err(err) = execute() {
    tracing::error!("fatal error: {err}\nexiting application");
    std::process::exit(1);
  }
}

#[allow(clippy::result_large_err)]
fn execute() -> Result<(), ErrorMessage> {
  let env_wrapper: Arc<dyn services::EnvWrapper> = Arc::new(DefaultEnvWrapper::default());

  // Construct AppOptions explicitly for production code clarity
  let app_options = AppOptionsBuilder::default()
    .env_wrapper(env_wrapper.clone())
    .env_type(ENV_TYPE.clone())
    .app_type(APP_TYPE.clone())
    .app_version(env!("CARGO_PKG_VERSION"))
    .auth_url(AUTH_URL)
    .auth_realm(AUTH_REALM)
    .build()
    .map_err(|err| {
      ErrorMessage::new(
        "app_options_builder_error".to_string(),
        ErrorType::InternalServer.to_string(),
        err.to_string(),
      )
    })?;

  let setting_service = setup_app_dirs(app_options)?;
  set_feature_settings(&setting_service)?;

  #[cfg(not(feature = "native"))]
  let _guard = setup_logs(&setting_service);

  let result = aexecute(Arc::new(setting_service));

  #[cfg(not(feature = "native"))]
  drop(_guard);

  result
}

fn aexecute(setting_service: Arc<dyn SettingService>) -> Result<(), ErrorMessage> {
  let runtime = Builder::new_multi_thread()
    .enable_all()
    .build()
    .map_err(AppSetupError::from)?;
  let result: Result<(), ErrorMessage> = runtime.block_on(async move {
    // Build the complete app service using the lib_bodhiserver function
    let app_service = Arc::new(build_app_service(setting_service.clone()).await?);
    match start(app_service).await {
      Err(err) => {
        tracing::warn!(?err, "application exited with error");
        let err: OpenAIApiError = err.into();
        let err: ErrorMessage = err.into();
        Err(err)
      }
      Ok(_) => {
        tracing::info!("application exited with success");
        Ok(())
      }
    }
  });
  result
}

#[cfg(not(feature = "native"))]
fn setup_logs(
  setting_service: &services::DefaultSettingService,
) -> Result<tracing_appender::non_blocking::WorkerGuard, crate::error::AppSetupError> {
  use crate::error::AppSetupError;
  use services::{SettingService, BODHI_LOG_STDOUT};
  use std::path::Path;
  use tracing::level_filters::LevelFilter;
  use tracing_appender::non_blocking::WorkerGuard;
  use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

  fn setup_logs(
    setting_service: &services::DefaultSettingService,
    logs_dir: &Path,
  ) -> Result<WorkerGuard, AppSetupError> {
    let file_appender = tracing_appender::rolling::daily(logs_dir, "bodhi.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let log_level: LevelFilter = setting_service.log_level().into();
    let filter = EnvFilter::new(log_level.to_string());
    let filter = filter.add_directive("hf_hub=error".parse().unwrap());

    // Check if we should output to stdout
    let enable_stdout = cfg!(debug_assertions)
      || setting_service
        .get_setting(BODHI_LOG_STDOUT)
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    let subscriber = tracing_subscriber::registry().with(filter);

    if enable_stdout {
      subscriber
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking))
        .init();
    } else {
      subscriber
        .with(fmt::layer().with_writer(non_blocking))
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

  let logs_dir = setting_service.logs_dir();
  let result = setup_logs(setting_service, &logs_dir);
  if result.is_err() {
    eprintln!("failed to configure logging, will be skipped");
  };
  result
}
