use crate::app::main_internal;
use objs::{ApiError, AppType, OpenAIApiError};
use services::{
  DefaultEnvService, DefaultEnvWrapper, DefaultSettingService, InitService, SettingService,
  SETTINGS_YAML,
};
use std::sync::Arc;

#[cfg(feature = "production")]
mod env_config {
  use objs::EnvType;
  use services::DefaultEnvWrapper;

  pub static ENV_TYPE: EnvType = EnvType::Production;
  pub static AUTH_URL: &str = "https://id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";

  pub fn set_feature_settings(setting_service: &mut DefaultSettingService) -> Result<(), ApiError> {
    Ok(())
  }
}

#[cfg(not(feature = "production"))]
mod env_config {
  use objs::{ApiError, EnvType};
  use services::DefaultSettingService;

  pub static ENV_TYPE: EnvType = EnvType::Development;
  pub static AUTH_URL: &str = "https://dev-id.getbodhi.app";
  pub static AUTH_REALM: &str = "bodhi";

  #[cfg(not(feature = "native"))]
  #[allow(clippy::result_large_err)]
  pub fn set_feature_settings(setting_service: &DefaultSettingService) -> Result<(), ApiError> {
    use services::{SettingService, BODHI_EXEC_LOOKUP_PATH};

    setting_service.set_default(
      BODHI_EXEC_LOOKUP_PATH,
      &serde_yaml::Value::String(concat!(env!("CARGO_MANIFEST_DIR"), "/bin").to_string()),
    )?;
    Ok(())
  }
  #[cfg(feature = "native")]
  #[allow(clippy::result_large_err)]
  pub fn set_feature_settings(_setting_service: &DefaultSettingService) -> Result<(), ApiError> {
    Ok(())
  }
}

pub use env_config::*;

#[cfg(feature = "native")]
pub const APP_TYPE: AppType = AppType::Native;

#[cfg(not(feature = "native"))]
pub const APP_TYPE: AppType = AppType::Container;

pub fn _main() {
  let env_wrapper = DefaultEnvWrapper::default();
  let init_service = InitService::new(&env_wrapper, &ENV_TYPE);
  let (bodhi_home, source) = match init_service.setup_bodhi_home() {
    Ok(bodhi_home) => bodhi_home,
    Err(err) => {
      eprintln!(
        "fatal error, setting up app dirs, error: {}\nexiting...",
        err
      );
      std::process::exit(1);
    }
  };
  let settings_file = bodhi_home.join(SETTINGS_YAML);
  let setting_service = DefaultSettingService::new_with_defaults(
    Arc::new(env_wrapper),
    &bodhi_home,
    source,
    settings_file,
  )
  .unwrap_or_else(|err| {
    let err: ApiError = err.into();
    eprintln!(
      "fatal error, setting up setting service, error: {}\nexiting...",
      err
    );
    std::process::exit(1);
  });
  setting_service.load_default_env();
  set_feature_settings(&setting_service).unwrap_or_else(|err| {
    eprintln!(
      "fatal error, setting up feature settings, error: {}\nexiting...",
      err
    );
    std::process::exit(1);
  });

  if let Err(err) = InitService::setup_hf_home(&setting_service) {
    eprintln!(
      "fatal error, setting up huggingface home, error: {}\nexiting...",
      err
    );
    std::process::exit(1);
  }
  if let Err(err) = InitService::setup_logs_dir(&setting_service, &bodhi_home) {
    eprintln!(
      "fatal error, setting up logs dir, error: {}\nexiting...",
      err
    );
    std::process::exit(1);
  }

  let env_service = match DefaultEnvService::new(
    ENV_TYPE.clone(),
    APP_TYPE.clone(),
    AUTH_URL.to_string(),
    AUTH_REALM.to_string(),
    Arc::new(setting_service),
  ) {
    Ok(env_service) => env_service,
    Err(err) => {
      let api_error: ApiError = err.into();
      eprintln!(
        "fatal error, setting up environment service, error: {}\nexiting...",
        api_error
      );
      std::process::exit(1);
    }
  };
  #[cfg(not(feature = "native"))]
  let _guard = setup_logs(&env_service);
  let result = main_internal(Arc::new(env_service));
  if let Err(err) = result {
    tracing::warn!(?err, "application exited with error");
    let err: ApiError = err.into();
    let err: OpenAIApiError = err.into();
    eprintln!("fatal error: {}\nexiting...", err);
    std::process::exit(1);
  } else {
    tracing::info!("application exited with success");
  }
  #[cfg(not(feature = "native"))]
  drop(_guard);
}

#[cfg(not(feature = "native"))]
fn setup_logs(
  env_service: &DefaultEnvService,
) -> Result<tracing_appender::non_blocking::WorkerGuard, crate::error::BodhiError> {
  use crate::error::Result;
  use services::{EnvService, BODHI_LOG_STDOUT};
  use std::path::Path;
  use tracing::level_filters::LevelFilter;
  use tracing_appender::non_blocking::WorkerGuard;
  use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

  fn setup_logs(env_service: &dyn EnvService, logs_dir: &Path) -> Result<WorkerGuard> {
    let file_appender = tracing_appender::rolling::daily(logs_dir, "bodhi.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let log_level: LevelFilter = env_service.log_level().into();
    let filter = EnvFilter::new(log_level.to_string());
    let filter = filter.add_directive("hf_hub=error".parse().unwrap());

    // Check if we should output to stdout
    let enable_stdout = cfg!(debug_assertions)
      || env_service
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

  let logs_dir = env_service.logs_dir();
  let result = setup_logs(env_service, &logs_dir);
  if result.is_err() {
    eprintln!("failed to configure logging, will be skipped");
  };
  result
}
