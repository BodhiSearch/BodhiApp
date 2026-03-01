use crate::test_utils::temp_dir;
use crate::{
  test_utils::{bodhi_home_setting, EnvWrapperStub},
  BootstrapParts, DefaultSettingService, MockSettingsChangeListener, SettingService,
  BODHI_EXEC_VARIANT, BODHI_HOME, BODHI_HOST, BODHI_LOGS, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT,
  BODHI_ON_RUNPOD, BODHI_PORT, BODHI_PUBLIC_HOST, BODHI_PUBLIC_PORT, BODHI_PUBLIC_SCHEME,
  BODHI_SCHEME, DEFAULT_HOST, DEFAULT_LOG_LEVEL, DEFAULT_LOG_STDOUT, DEFAULT_PORT, DEFAULT_SCHEME,
  HF_HOME, RUNPOD_POD_ID,
};
use crate::{AppCommand, Setting, SettingInfo, SettingMetadata, SettingSource};
use anyhow_trace::anyhow_trace;
use mockall::predicate::eq;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_yaml::Value;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use tempfile::TempDir;

fn noop_settings_repo() -> Arc<InMemorySettingsRepository> {
  Arc::new(InMemorySettingsRepository::new())
}

fn make_service_from_parts(
  env_wrapper: Arc<dyn crate::EnvWrapper>,
  bodhi_home_setting: Setting,
  system_settings: Vec<Setting>,
  file_defaults: HashMap<String, Value>,
  settings_file: PathBuf,
  db_service: Arc<dyn crate::SettingsRepository>,
) -> DefaultSettingService {
  let bodhi_home = PathBuf::from(bodhi_home_setting.value.as_str().unwrap());
  let mut all_system_settings = system_settings;
  all_system_settings.push(bodhi_home_setting);
  DefaultSettingService::from_parts(
    BootstrapParts {
      env_wrapper,
      settings_file,
      system_settings: all_system_settings,
      file_defaults,
      app_settings: HashMap::new(),
      app_command: AppCommand::Default,
      bodhi_home,
    },
    db_service,
  )
}

fn make_simple_service(
  env_wrapper: Arc<dyn crate::EnvWrapper>,
  settings_file: PathBuf,
  system_settings: Vec<Setting>,
  db_service: Arc<dyn crate::SettingsRepository>,
) -> DefaultSettingService {
  let bodhi_home = env_wrapper
    .home_dir()
    .unwrap_or_else(|| PathBuf::from("/tmp"));
  DefaultSettingService::from_parts(
    BootstrapParts {
      env_wrapper,
      settings_file,
      system_settings,
      file_defaults: HashMap::new(),
      app_settings: HashMap::new(),
      app_command: AppCommand::Default,
      bodhi_home,
    },
    db_service,
  )
}

use crate::test_utils::InMemorySettingsRepository;
#[rstest]
#[case::system_settings_cannot_be_overridden(
  "TEST_SYSTEM_KEY",
  Some("cmdline_value"),
  Some("env_value"),
  Some("file_value"),
  Some("default_value"),
  "system_value",
  SettingSource::System
)]
#[case::command_line_highest_priority(
  "TEST_KEY",
  Some("cmdline_value"),
  Some("env_value"),
  Some("file_value"),
  Some("default_value"),
  "cmdline_value",
  SettingSource::CommandLine
)]
#[case::environment_override(
  "TEST_KEY",
  None,
  Some("env_value"),
  Some("file_value"),
  Some("default_value"),
  "env_value",
  SettingSource::Environment
)]
#[case::file_when_no_env(
  "TEST_KEY",
  None,
  None,
  Some("file_value"),
  Some("default_value"),
  "file_value",
  SettingSource::SettingsFile
)]
#[case::default_when_no_others(
  "SOME_KEY",
  None,
  None,
  None,
  Some("default_value"),
  "default_value",
  SettingSource::Default
)]
#[tokio::test]
async fn test_settings_precedence(
  temp_dir: TempDir,
  #[case] key: &str,
  #[case] cmdline_value: Option<&str>,
  #[case] env_value: Option<&str>,
  #[case] file_value: Option<&str>,
  #[case] default_value: Option<&str>,
  #[case] expected_value: &str,
  #[case] expected_source: SettingSource,
) -> anyhow::Result<()> {
  let settings_file = temp_dir.path().join("settings.yaml");
  if let Some(file_val) = file_value {
    std::fs::write(&settings_file, format!("{}: {}", key, file_val))?;
  }
  let mut env_vars = maplit::hashmap! {
    BODHI_HOME.to_string() => temp_dir.path().display().to_string()
  };
  if let Some(env_val) = env_value {
    env_vars.insert(key.to_string(), env_val.to_string());
  }
  let env_stub = EnvWrapperStub::new(env_vars);
  let file_defaults = if let Some(default_val) = default_value {
    maplit::hashmap! {
      key.to_string() => Value::String(default_val.to_string())
    }
  } else {
    HashMap::new()
  };
  let service = make_service_from_parts(
    Arc::new(env_stub),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![Setting {
      key: "TEST_SYSTEM_KEY".to_string(),
      value: Value::String("system_value".to_string()),
      source: SettingSource::System,
      metadata: SettingMetadata::String,
    }],
    file_defaults,
    settings_file,
    noop_settings_repo(),
  );
  if let Some(cmdline_val) = cmdline_value {
    service
      .set_setting_with_source(
        key,
        &Value::String(cmdline_val.to_string()),
        SettingSource::CommandLine,
      )
      .await?;
  }
  helpers::assert_setting_value_with_source(&service, key, Some(expected_value), expected_source)
    .await;
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_setting_service_init_with_defaults(temp_dir: TempDir) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let home_dir = temp_dir.path().join("home");
  let env_wrapper =
    EnvWrapperStub::new(maplit::hashmap! {"HOME".to_string() => home_dir.display().to_string()});
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path.clone(),
    noop_settings_repo(),
  );
  for (key, expected) in [
    (
      BODHI_HOME,
      home_dir.join(".cache").join("bodhi").display().to_string(),
    ),
    (
      BODHI_LOGS,
      home_dir
        .join(".cache")
        .join("bodhi")
        .join("logs")
        .display()
        .to_string(),
    ),
    (
      HF_HOME,
      home_dir
        .join(".cache")
        .join("huggingface")
        .display()
        .to_string(),
    ),
    (BODHI_SCHEME, DEFAULT_SCHEME.to_string()),
    (BODHI_HOST, DEFAULT_HOST.to_string()),
    (BODHI_LOG_LEVEL, DEFAULT_LOG_LEVEL.to_string()),
    (
      BODHI_EXEC_VARIANT,
      llama_server_proc::DEFAULT_VARIANT.to_string(),
    ),
  ] {
    assert_eq!(
      expected,
      service
        .get_default_value(key)
        .await
        .unwrap()
        .as_str()
        .unwrap()
    );
  }
  assert_eq!(
    DEFAULT_PORT as i64,
    service
      .get_default_value(BODHI_PORT)
      .await
      .unwrap()
      .as_i64()
      .unwrap()
  );
  assert_eq!(
    DEFAULT_LOG_STDOUT,
    service
      .get_default_value(BODHI_LOG_STDOUT)
      .await
      .unwrap()
      .as_bool()
      .unwrap()
  );
  Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum NotificationOperation {
  OverrideSetting,
  DeleteSetting,
  SetWithEnvOverride,
  SetDefault,
}

#[rstest]
#[case::override_setting_db_write(
  NotificationOperation::OverrideSetting,
  None, // no env var
  None, // no initial file value
  Some("default.host"), // default value
  Some("new.host"), // new value to set via DB
  Some((
    Some("default.host"), SettingSource::Default,
    Some("new.host"), SettingSource::Database
  )) // DB write becomes new effective value
)]
#[case::delete_setting_from_db(
  NotificationOperation::DeleteSetting,
  None, // no env var
  None, // no initial file value
  Some("default.host"), // default value
  None, // delete operation
  Some((
    Some("default.host"), SettingSource::Default,
    Some("default.host"), SettingSource::Default
  )) // DB delete falls back to default
)]
#[case::set_with_env_override(
  NotificationOperation::SetWithEnvOverride,
  Some("env.host"), // env var set
  None, // no initial file value
  None, // no default needed
  Some("new.host"), // new value to set (env still wins)
  Some((
    Some("env.host"), SettingSource::Environment,
    Some("env.host"), SettingSource::Environment
  )) // no actual change in effective value
)]
#[case::set_default_no_notification(
  NotificationOperation::SetDefault,
  None, // no env var
  Some("test.host"), // initial file value
  None, // no existing default
  Some("default.host"), // default value to set
  None // no notification expected
)]
#[tokio::test]
async fn test_change_notifications(
  temp_dir: TempDir,
  #[case] operation: NotificationOperation,
  #[case] env_value: Option<&str>,
  #[case] initial_file_value: Option<&str>,
  #[case] default_value: Option<&str>,
  #[case] new_value: Option<&str>,
  #[case] expected_notification: Option<(Option<&str>, SettingSource, Option<&str>, SettingSource)>,
) -> anyhow::Result<()> {
  let key = BODHI_HOST;
  let path = temp_dir.path().join("settings.yaml");
  if let Some(file_val) = initial_file_value {
    std::fs::write(&path, format!("{}: {}", key, file_val))?;
  }
  let env_vars = if let Some(val) = env_value {
    maplit::hashmap! { key.to_string() => val.to_string() }
  } else {
    HashMap::new()
  };
  let env_stub = EnvWrapperStub::new(env_vars);
  let service = make_simple_service(Arc::new(env_stub), path, vec![], noop_settings_repo());
  if let Some(default_val) = default_value {
    service
      .set_default(key, &Value::String(default_val.to_string()))
      .await?;
  }
  let mut mock_listener = MockSettingsChangeListener::default();
  match expected_notification {
    Some((old_val, old_source, new_val, new_source)) => {
      mock_listener
        .expect_on_change()
        .with(
          eq(key),
          eq(old_val.map(|v| Value::String(v.to_string()))),
          eq(old_source),
          eq(new_val.map(|v| Value::String(v.to_string()))),
          eq(new_source),
        )
        .times(1)
        .return_once(|_, _, _, _, _| ());
    }
    None => {
      mock_listener.expect_on_change().never();
    }
  }

  service.add_listener(Arc::new(mock_listener)).await;

  match operation {
    NotificationOperation::OverrideSetting => {
      service.set_setting(key, new_value.unwrap()).await?;
    }
    NotificationOperation::DeleteSetting => {
      service.delete_setting(key).await?;
    }
    NotificationOperation::SetWithEnvOverride => {
      service.set_setting(key, new_value.unwrap()).await?;
    }
    NotificationOperation::SetDefault => {
      service
        .set_default(key, &Value::String(new_value.unwrap().to_string()))
        .await?;
    }
  }
  Ok(())
}

#[rstest]
#[case::essential_properties_from_file(
  maplit::hashmap! {
    BODHI_SCHEME => Value::String("https".to_string()),
    BODHI_HOST => Value::String("example.com".to_string()),
    BODHI_PORT => Value::Number(8443.into()),
    BODHI_LOG_LEVEL => Value::String("debug".to_string()),
    BODHI_LOG_STDOUT => Value::Bool(true)
  },
  vec![
    (BODHI_SCHEME, "https"),
    (BODHI_HOST, "example.com"),
    (BODHI_PORT, "8443"),
    (BODHI_LOG_LEVEL, "debug"),
    (BODHI_LOG_STDOUT, "true")
  ]
)]
#[case::mixed_file_and_hardcoded(
  maplit::hashmap! {
    BODHI_SCHEME => Value::String("https".to_string())
  },
  vec![
    (BODHI_SCHEME, "https"),
    (BODHI_HOST, DEFAULT_HOST)
  ]
)]
#[case::precedence_with_file_defaults(
  maplit::hashmap! {
    BODHI_SCHEME => Value::String("file_default_value".to_string())
  },
  vec![
    ("BODHI_SCHEME", "file_default_value"),
  ]
)]
#[case::custom_properties_support(
  maplit::hashmap! {
    BODHI_SCHEME => Value::String("https".to_string()),
    BODHI_HOST => Value::String("example.com".to_string()),
    "CUSTOM_TIMEOUT" => Value::Number(30.into()),
    "CUSTOM_STRING_SETTING" => Value::String("custom_value".to_string()),
    "CUSTOM_BOOL_SETTING" => Value::Bool(true),
    "BODHI_LLAMACPP_ARGS_METAL" => Value::String("--threads 8 --gpu-layers 32".to_string())
  },
  vec![
    (BODHI_SCHEME, "https"),
    (BODHI_HOST, "example.com"),
    ("CUSTOM_TIMEOUT", "30"),
    ("CUSTOM_STRING_SETTING", "custom_value"),
    ("CUSTOM_BOOL_SETTING", "true"),
    ("BODHI_LLAMACPP_ARGS_METAL", "--threads 8 --gpu-layers 32")
  ]
)]
#[case::essential_properties_fallbacks(
  maplit::hashmap! {
    "CUSTOM_PROPERTY" => Value::String("custom_value".to_string())
  },
  vec![
    (BODHI_SCHEME, DEFAULT_SCHEME),
    (BODHI_HOST, DEFAULT_HOST),
    (BODHI_PORT, "1135"),
    (BODHI_LOG_STDOUT, "false"),
    ("CUSTOM_PROPERTY", "custom_value")
  ]
)]
#[tokio::test]
async fn test_file_defaults_integration(
  temp_dir: TempDir,
  #[case] file_defaults: HashMap<&str, Value>,
  #[case] expected_values: Vec<(&str, &str)>,
) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let env_vars = maplit::hashmap! {
    "HOME".to_string() => temp_dir.path().display().to_string()
  };
  let env_stub = EnvWrapperStub::new(env_vars);
  let file_defaults: HashMap<String, Value> = file_defaults
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
  let service = make_service_from_parts(
    Arc::new(env_stub),
    bodhi_home_setting(temp_dir.path(), SettingSource::Default),
    vec![],
    file_defaults,
    path,
    noop_settings_repo(),
  );

  for (key, expected) in expected_values {
    match key {
      BODHI_PORT => {
        let expected_num = expected
          .parse::<i64>()
          .expect("Port should be parseable as number");
        helpers::assert_default_value_i64(&service, key, expected_num).await;
      }
      BODHI_LOG_STDOUT => {
        let expected_bool = expected
          .parse::<bool>()
          .expect("Log stdout should be parseable as bool");
        helpers::assert_default_value_bool(&service, key, expected_bool).await;
      }
      "CUSTOM_TIMEOUT" => {
        let expected_num = expected
          .parse::<i64>()
          .expect("Custom timeout should be parseable as number");
        helpers::assert_default_value_i64(&service, key, expected_num).await;
      }
      "CUSTOM_BOOL_SETTING" => {
        let expected_bool = expected
          .parse::<bool>()
          .expect("Custom bool setting should be parseable as bool");
        helpers::assert_default_value_bool(&service, key, expected_bool).await;
      }
      _ => helpers::assert_default_value_str(&service, key, expected).await,
    }
  }
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_setting_service_delete_non_existent(temp_dir: TempDir) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let env_stub = EnvWrapperStub::new(HashMap::new());
  let service = make_simple_service(
    Arc::new(env_stub),
    path.clone(),
    vec![],
    noop_settings_repo(),
  );

  service.delete_setting("NON_EXISTENT_KEY").await?;
  assert_eq!(None, service.get_setting("NON_EXISTENT_KEY").await);

  Ok(())
}

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_setting_service_list(
  temp_dir: TempDir,
  #[from(temp_dir)] bodhi_home: TempDir,
) -> anyhow::Result<()> {
  let env_wrapper = EnvWrapperStub::new(maplit::hashmap! {
    "HOME".to_owned() => "/test/home".to_string(),
    BODHI_LOGS.to_owned() => "/test/logs".to_string(),
    BODHI_LOG_LEVEL.to_owned() => "debug".to_string(),
    BODHI_LOG_STDOUT.to_owned() => "true".to_string(),
    HF_HOME.to_owned() => "/test/hf/home".to_string(),
  });

  let settings_file = temp_dir.path().join("settings.yaml");
  fs::write(
    &settings_file,
    r#"
BODHI_HOST: test.host
BODHI_PORT: 8080
BODHI_EXEC_VARIANT: metal
BODHI_EXEC_LOOKUP_PATH: /test/exec/lookup
"#,
  )?;

  let setting_service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(bodhi_home.path(), SettingSource::Default),
    vec![],
    HashMap::new(),
    settings_file.clone(),
    noop_settings_repo(),
  );
  let bodhi_home = bodhi_home.path().to_path_buf();
  // WHEN
  let settings = setting_service
    .list()
    .await
    .into_iter()
    .map(|setting| (setting.key.clone(), setting))
    .collect::<HashMap<String, SettingInfo>>();

  // THEN
  // System settings
  let expected_bodhi_home = SettingInfo {
    key: BODHI_HOME.to_string(),
    current_value: serde_yaml::Value::String(bodhi_home.display().to_string()),
    default_value: serde_yaml::Value::String(bodhi_home.display().to_string()),
    source: SettingSource::Default,
    metadata: SettingMetadata::String,
  };
  assert_eq!(
    expected_bodhi_home,
    settings.get(BODHI_HOME).unwrap().clone()
  );

  // Environment variable settings
  let expected_log_level = SettingInfo {
    key: BODHI_LOG_LEVEL.to_string(),
    current_value: serde_yaml::Value::String("debug".to_string()),
    default_value: serde_yaml::Value::String(DEFAULT_LOG_LEVEL.to_string()),
    source: SettingSource::Environment,
    metadata: SettingMetadata::option(
      ["error", "warn", "info", "debug", "trace"]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    ),
  };
  assert_eq!(
    expected_log_level,
    settings.get(BODHI_LOG_LEVEL).unwrap().clone()
  );

  // Settings file settings
  let expected_port = SettingInfo {
    key: BODHI_PORT.to_string(),
    current_value: serde_yaml::Value::Number(8080.into()),
    default_value: serde_yaml::Value::Number(DEFAULT_PORT.into()),
    source: SettingSource::SettingsFile,
    metadata: SettingMetadata::Number { min: 1, max: 65535 },
  };
  assert_eq!(expected_port, settings.get(BODHI_PORT).unwrap().clone());

  // Boolean setting
  let expected_stdout = SettingInfo {
    key: BODHI_LOG_STDOUT.to_string(),
    current_value: serde_yaml::Value::Bool(true),
    default_value: serde_yaml::Value::Bool(false),
    source: SettingSource::Environment,
    metadata: SettingMetadata::Boolean,
  };
  assert_eq!(
    expected_stdout,
    settings.get(BODHI_LOG_STDOUT).unwrap().clone()
  );

  // Default value setting
  let expected_scheme = SettingInfo {
    key: BODHI_SCHEME.to_string(),
    current_value: serde_yaml::Value::String(DEFAULT_SCHEME.to_string()),
    default_value: serde_yaml::Value::String(DEFAULT_SCHEME.to_string()),
    source: SettingSource::Default,
    metadata: SettingMetadata::String,
  };
  assert_eq!(expected_scheme, settings.get(BODHI_SCHEME).unwrap().clone());

  let expected_host = SettingInfo {
    key: BODHI_HOST.to_string(),
    current_value: serde_yaml::Value::String("test.host".to_string()),
    default_value: serde_yaml::Value::String("0.0.0.0".to_string()),
    source: SettingSource::SettingsFile,
    metadata: SettingMetadata::String,
  };
  assert_eq!(expected_host, settings.get(BODHI_HOST).unwrap().clone());
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_public_settings_fallback_behavior(temp_dir: TempDir) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let env_wrapper = EnvWrapperStub::new(HashMap::new());
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path,
    noop_settings_repo(),
  );

  assert_eq!(service.public_server_url().await, "http://0.0.0.0:1135");
  helpers::assert_default_value_str(&service, BODHI_PUBLIC_HOST, DEFAULT_HOST).await;
  assert_eq!(
    service
      .get_default_value(BODHI_PUBLIC_PORT)
      .await
      .unwrap()
      .as_u64()
      .unwrap(),
    DEFAULT_PORT as u64
  );
  helpers::assert_default_value_str(&service, BODHI_PUBLIC_SCHEME, DEFAULT_SCHEME).await;

  service.set_setting(BODHI_HOST, "example.com").await?;
  service.set_setting(BODHI_PORT, "8080").await?;
  service.set_setting(BODHI_SCHEME, "https").await?;

  assert_eq!(
    service.public_server_url().await,
    "https://example.com:8080"
  );
  helpers::assert_default_value_str(&service, BODHI_PUBLIC_HOST, "example.com").await;
  assert_eq!(
    service
      .get_default_value(BODHI_PUBLIC_PORT)
      .await
      .unwrap()
      .as_u64()
      .unwrap(),
    8080
  );
  helpers::assert_default_value_str(&service, BODHI_PUBLIC_SCHEME, "https").await;

  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_public_settings_explicit_override(temp_dir: TempDir) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let env_wrapper = EnvWrapperStub::new(HashMap::new());
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path,
    noop_settings_repo(),
  );

  service
    .set_setting(BODHI_HOST, "internal.example.com")
    .await?;
  service.set_setting(BODHI_PORT, "8080").await?;
  service.set_setting(BODHI_SCHEME, "http").await?;
  assert_eq!(
    service.public_server_url().await,
    "http://internal.example.com:8080"
  );

  service
    .set_setting(BODHI_PUBLIC_HOST, "public.example.com")
    .await?;
  service.set_setting(BODHI_PUBLIC_PORT, "443").await?;
  service.set_setting(BODHI_PUBLIC_SCHEME, "https").await?;

  assert_eq!(
    service.public_server_url().await,
    "https://public.example.com"
  );

  service.set_setting(BODHI_PUBLIC_PORT, "8443").await?;
  assert_eq!(
    service.public_server_url().await,
    "https://public.example.com:8443"
  );

  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_public_settings_metadata_validation(temp_dir: TempDir) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let env_wrapper = EnvWrapperStub::new(HashMap::new());
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path,
    noop_settings_repo(),
  );

  assert_eq!(
    service.get_setting_metadata(BODHI_PUBLIC_PORT).await,
    SettingMetadata::Number { min: 1, max: 65535 }
  );

  assert_eq!(
    service.get_setting_metadata(BODHI_PUBLIC_HOST).await,
    SettingMetadata::String
  );
  assert_eq!(
    service.get_setting_metadata(BODHI_PUBLIC_SCHEME).await,
    SettingMetadata::String
  );

  Ok(())
}

#[rstest]
#[case("http", "example.com", "80", "http://example.com")] // Standard port omitted for HTTP
#[case("https", "example.com", "443", "https://example.com")] // Standard port omitted for HTTPS
#[case("https", "example.com", "8080", "https://example.com:8080")] // Non-standard port included
#[case("http", "example.com", "8443", "http://example.com:8443")] // Non-standard port included
#[case("http", "localhost", "80", "http://localhost")] // Standard port omitted for localhost
#[tokio::test]
async fn test_public_settings_url_construction_edge_cases(
  temp_dir: TempDir,
  #[case] scheme: &str,
  #[case] host: &str,
  #[case] port: &str,
  #[case] expected_url: &str,
) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let env_wrapper = EnvWrapperStub::new(HashMap::new());
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path.clone(),
    noop_settings_repo(),
  );

  service.set_setting(BODHI_PUBLIC_SCHEME, scheme).await?;
  service.set_setting(BODHI_PUBLIC_HOST, host).await?;
  service.set_setting(BODHI_PUBLIC_PORT, port).await?;

  assert_eq!(service.public_server_url().await, expected_url);

  Ok(())
}

#[rstest]
#[case(
  // Default settings scenario
  None, None, None, // No public settings override
  None, None, None, // No regular settings override
  "http://0.0.0.0:1135",
  "http://0.0.0.0:1135/ui/chat",
  "http://0.0.0.0:1135/ui/auth/callback"
)]
#[case(
  // All public settings overridden
  Some("https"), Some("public.example.com"), Some("443"),
  None, None, None, // Regular settings not needed
  "https://public.example.com", // Port 443 omitted
  "https://public.example.com/ui/chat",
  "https://public.example.com/ui/auth/callback"
)]
#[case(
  // Public settings with non-standard port
  Some("https"), Some("public.example.com"), Some("8443"),
  None, None, None,
  "https://public.example.com:8443",
  "https://public.example.com:8443/ui/chat",
  "https://public.example.com:8443/ui/auth/callback"
)]
#[case(
  // Mixed scenario: only public host set, fallback to regular scheme/port
  None, Some("cdn.example.com"), None, // Only public host
  Some("http"), Some("internal.example.com"), Some("8080"), // Regular settings set
  "http://cdn.example.com:8080", // Uses public host, regular scheme/port
  "http://cdn.example.com:8080/ui/chat",
  "http://cdn.example.com:8080/ui/auth/callback"
)]
#[tokio::test]
async fn test_integration_method_behaviors(
  temp_dir: TempDir,
  #[case] public_scheme: Option<&str>,
  #[case] public_host: Option<&str>,
  #[case] public_port: Option<&str>,
  #[case] regular_scheme: Option<&str>,
  #[case] regular_host: Option<&str>,
  #[case] regular_port: Option<&str>,
  #[case] expected_public_url: &str,
  #[case] expected_frontend_url: &str,
  #[case] expected_callback_url: &str,
) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let env_wrapper = EnvWrapperStub::new(HashMap::new());
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path,
    noop_settings_repo(),
  );

  if let Some(scheme) = regular_scheme {
    service.set_setting(BODHI_SCHEME, scheme).await?;
  }
  if let Some(host) = regular_host {
    service.set_setting(BODHI_HOST, host).await?;
  }
  if let Some(port) = regular_port {
    service.set_setting(BODHI_PORT, port).await?;
  }

  if let Some(scheme) = public_scheme {
    service.set_setting(BODHI_PUBLIC_SCHEME, scheme).await?;
  }
  if let Some(host) = public_host {
    service.set_setting(BODHI_PUBLIC_HOST, host).await?;
  }
  if let Some(port) = public_port {
    service.set_setting(BODHI_PUBLIC_PORT, port).await?;
  }

  assert_eq!(service.public_server_url().await, expected_public_url);
  assert_eq!(service.frontend_default_url().await, expected_frontend_url);
  assert_eq!(service.login_callback_url().await, expected_callback_url);

  Ok(())
}

#[rstest]
#[case::runpod_disabled_no_env(Some("false"), None, "http://0.0.0.0:1135")]
#[case::runpod_disabled_unparseable(Some("invalid"), None, "http://0.0.0.0:1135")]
#[case::runpod_disabled_not_set(None, None, "http://0.0.0.0:1135")]
#[case::runpod_enabled_no_pod_id(Some("true"), None, "http://0.0.0.0:1135")]
#[case::runpod_enabled_with_pod_id(
  Some("true"),
  Some("abc123def456"),
  "https://abc123def456-1135.proxy.runpod.net"
)]
#[case::runpod_enabled_empty_pod_id(Some("true"), Some(""), "http://0.0.0.0:1135")]
#[tokio::test]
async fn test_runpod_feature_behavior(
  temp_dir: TempDir,
  #[case] runpod_flag: Option<&str>,
  #[case] runpod_pod_id: Option<&str>,
  #[case] expected_url: &str,
) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let mut env_vars = HashMap::new();

  env_vars.insert(
    BODHI_HOME.to_string(),
    temp_dir.path().display().to_string(),
  );

  runpod_flag.map(|flag| env_vars.insert(BODHI_ON_RUNPOD.to_string(), flag.to_string()));
  runpod_pod_id.map(|hostname| env_vars.insert(RUNPOD_POD_ID.to_string(), hostname.to_string()));

  let env_wrapper = EnvWrapperStub::new(env_vars);
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path,
    noop_settings_repo(),
  );

  assert_eq!(service.public_server_url().await, expected_url);
  Ok(())
}

#[rstest]
#[case::explicit_overrides_runpod(
  "true",
  Some("abc123def456"),
  Some("https"),
  Some("explicit.example.com"),
  Some("8443"),
  "https://explicit.example.com:8443"
)]
#[case::partial_override_scheme(
  "true",
  Some("abc123def456"),
  Some("http"),
  None,
  None,
  "http://abc123def456-1135.proxy.runpod.net:443"
)]
#[case::partial_override_port(
  "true",
  Some("abc123def456"),
  None,
  None,
  Some("8080"),
  "https://abc123def456-1135.proxy.runpod.net:8080"
)]
#[tokio::test]
async fn test_runpod_with_explicit_overrides(
  temp_dir: TempDir,
  #[case] runpod_flag: &str,
  #[case] runpod_pod_id: Option<&str>,
  #[case] public_scheme: Option<&str>,
  #[case] public_host: Option<&str>,
  #[case] public_port: Option<&str>,
  #[case] expected_url: &str,
) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let mut env_vars = HashMap::new();

  env_vars.insert(
    BODHI_HOME.to_string(),
    temp_dir.path().display().to_string(),
  );
  env_vars.insert(BODHI_ON_RUNPOD.to_string(), runpod_flag.to_string());

  runpod_pod_id.map(|hostname| env_vars.insert(RUNPOD_POD_ID.to_string(), hostname.to_string()));

  let env_wrapper = EnvWrapperStub::new(env_vars);
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path,
    noop_settings_repo(),
  );

  if let Some(scheme) = public_scheme {
    service.set_setting(BODHI_PUBLIC_SCHEME, scheme).await?;
  }
  if let Some(host) = public_host {
    service.set_setting(BODHI_PUBLIC_HOST, host).await?;
  }
  if let Some(port) = public_port {
    service.set_setting(BODHI_PUBLIC_PORT, port).await?;
  }

  assert_eq!(service.public_server_url().await, expected_url);
  Ok(())
}

#[rstest]
#[case::runpod_disabled_no_pod_id(Some("false"), None, false)]
#[case::runpod_enabled_no_pod_id(Some("true"), None, false)]
#[case::runpod_enabled_with_pod_id(Some("true"), Some("abc123def456"), true)]
#[case::runpod_unparseable_with_pod_id(Some("invalid"), Some("abc123def456"), false)]
#[case::runpod_empty_flag_with_pod_id(Some(""), Some("abc123def456"), false)]
#[case::runpod_not_set(None, Some("abc123def456"), false)]
#[case::runpod_enabled_empty_pod_id(Some("true"), Some(""), false)]
#[tokio::test]
async fn test_on_runpod_enabled_parsing(
  temp_dir: TempDir,
  #[case] runpod_flag: Option<&str>,
  #[case] runpod_pod_id: Option<&str>,
  #[case] expected: bool,
) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let mut env_vars = HashMap::new();

  env_vars.insert(
    BODHI_HOME.to_string(),
    temp_dir.path().display().to_string(),
  );

  runpod_flag.map(|flag| env_vars.insert(BODHI_ON_RUNPOD.to_string(), flag.to_string()));
  runpod_pod_id.map(|hostname| env_vars.insert(RUNPOD_POD_ID.to_string(), hostname.to_string()));

  let env_wrapper = EnvWrapperStub::new(env_vars);
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path,
    noop_settings_repo(),
  );

  assert_eq!(service.on_runpod_enabled().await, expected);
  Ok(())
}

#[rstest]
#[tokio::test]
async fn test_runpod_feature_individual_methods(temp_dir: TempDir) -> anyhow::Result<()> {
  let path = temp_dir.path().join("settings.yaml");
  let env_vars = maplit::hashmap! {
    BODHI_ON_RUNPOD.to_string() => "true".to_string(),
    RUNPOD_POD_ID.to_string() => "abc123def456".to_string(),
    BODHI_HOME.to_string() => temp_dir.path().display().to_string()
  };

  let env_wrapper = EnvWrapperStub::new(env_vars);
  let service = make_service_from_parts(
    Arc::new(env_wrapper),
    bodhi_home_setting(temp_dir.path(), SettingSource::Environment),
    vec![],
    HashMap::new(),
    path,
    noop_settings_repo(),
  );

  assert_eq!(service.on_runpod_enabled().await, true);
  assert_eq!(
    service.public_host().await,
    "abc123def456-1135.proxy.runpod.net"
  );
  assert_eq!(service.public_scheme().await, "https");
  assert_eq!(service.public_port().await, 443);

  assert_eq!(
    service.public_server_url().await,
    "https://abc123def456-1135.proxy.runpod.net"
  );

  Ok(())
}

mod helpers {
  use crate::{SettingService, SettingSource};
  use pretty_assertions::assert_eq;
  use serde_yaml::Value;

  pub async fn assert_default_value_str(service: &dyn SettingService, key: &str, expected: &str) {
    match service.get_default_value(key).await {
      Some(Value::String(actual)) => assert_eq!(expected, &actual),
      Some(other_value) => panic!(
        "Expected string value for key '{}' but got: {:?}",
        key, other_value
      ),
      None => panic!("Expected default value for key '{}' but got None", key),
    }
  }

  pub async fn assert_default_value_i64(service: &dyn SettingService, key: &str, expected: i64) {
    match service.get_default_value(key).await {
      Some(Value::Number(actual)) => assert_eq!(expected, actual.as_i64().unwrap()),
      Some(other_value) => panic!(
        "Expected number value for key '{}' but got: {:?}",
        key, other_value
      ),
      None => panic!("Expected default value for key '{}' but got None", key),
    }
  }

  pub async fn assert_default_value_bool(service: &dyn SettingService, key: &str, expected: bool) {
    match service.get_default_value(key).await {
      Some(Value::Bool(actual)) => assert_eq!(expected, actual),
      Some(other_value) => panic!(
        "Expected boolean value for key '{}' but got: {:?}",
        key, other_value
      ),
      None => panic!("Expected default value for key '{}' but got None", key),
    }
  }

  pub async fn assert_setting_value_with_source(
    service: &dyn SettingService,
    key: &str,
    expected_value: Option<&str>,
    expected_source: SettingSource,
  ) {
    let (value, source) = service.get_setting_value_with_source(key).await;
    let actual_value = value.map(|v| match v {
      Value::String(s) => s,
      Value::Number(n) => n.to_string(),
      Value::Bool(b) => b.to_string(),
      _ => "null".to_string(),
    });
    assert_eq!(expected_value.map(|s| s.to_string()), actual_value);
    assert_eq!(expected_source, source);
  }
}
