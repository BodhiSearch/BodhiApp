use crate::{
  db::DbSetting,
  test_utils::{bodhi_home_setting, EnvWrapperStub},
  BootstrapParts, DefaultSettingService, SettingService, BODHI_KEEP_ALIVE_SECS,
};
use objs::{AppCommand, SettingSource};
use rstest::rstest;
use serde_yaml::Value;
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;

async fn make_service(
  temp_dir: &TempDir,
  envs: HashMap<String, String>,
  settings_yaml_content: Option<&str>,
  db_settings: Vec<(&str, &str)>,
) -> DefaultSettingService {
  let bodhi_home = temp_dir.path().to_path_buf();
  let settings_file = bodhi_home.join("settings.yaml");
  if let Some(content) = settings_yaml_content {
    std::fs::write(&settings_file, content).unwrap();
  }

  let mut mock_repo = crate::db::MockSettingsRepository::new();
  let store: Arc<std::sync::RwLock<HashMap<String, DbSetting>>> =
    Arc::new(std::sync::RwLock::new(HashMap::new()));
  for (key, value) in &db_settings {
    store.write().unwrap().insert(
      key.to_string(),
      DbSetting {
        key: key.to_string(),
        value: value.to_string(),
        value_type: "number".to_string(),
        created_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
        updated_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
      },
    );
  }
  let store_get = store.clone();
  let store_upsert = store.clone();
  let store_delete = store.clone();
  let store_list = store;
  mock_repo
    .expect_get_setting()
    .returning(move |key| Ok(store_get.read().unwrap().get(key).cloned()));
  mock_repo.expect_upsert_setting().returning(move |setting| {
    store_upsert
      .write()
      .unwrap()
      .insert(setting.key.clone(), setting.clone());
    Ok(setting.clone())
  });
  mock_repo.expect_delete_setting().returning(move |key| {
    store_delete.write().unwrap().remove(key);
    Ok(())
  });
  mock_repo
    .expect_list_settings()
    .returning(move || Ok(store_list.read().unwrap().values().cloned().collect()));

  DefaultSettingService::from_parts(
    BootstrapParts {
      env_wrapper: Arc::new(EnvWrapperStub::new(envs)),
      settings_file,
      system_settings: vec![bodhi_home_setting(
        temp_dir.path(),
        SettingSource::Environment,
      )],
      file_defaults: HashMap::new(),
      app_settings: HashMap::new(),
      app_command: AppCommand::Default,
      bodhi_home,
    },
    Arc::new(mock_repo),
  )
}

#[rstest]
#[tokio::test]
async fn test_database_over_default() {
  let temp_dir = TempDir::new().unwrap();
  let service = make_service(
    &temp_dir,
    HashMap::new(),
    None,
    vec![(BODHI_KEEP_ALIVE_SECS, "600")],
  )
  .await;

  let (value, source) = service
    .get_setting_value_with_source(BODHI_KEEP_ALIVE_SECS)
    .await;
  assert_eq!(SettingSource::Database, source);
  assert_eq!(Some(Value::Number(600.into())), value);
}

#[rstest]
#[tokio::test]
async fn test_env_over_database() {
  let temp_dir = TempDir::new().unwrap();
  let service = make_service(
    &temp_dir,
    HashMap::from([(BODHI_KEEP_ALIVE_SECS.to_string(), "900".to_string())]),
    None,
    vec![(BODHI_KEEP_ALIVE_SECS, "600")],
  )
  .await;

  let (value, source) = service
    .get_setting_value_with_source(BODHI_KEEP_ALIVE_SECS)
    .await;
  assert_eq!(SettingSource::Environment, source);
  assert_eq!(Some(Value::Number(900.into())), value);
}

#[rstest]
#[tokio::test]
async fn test_database_over_file() {
  let temp_dir = TempDir::new().unwrap();
  let yaml_content = format!("{}: 1200", BODHI_KEEP_ALIVE_SECS);
  let service = make_service(
    &temp_dir,
    HashMap::new(),
    Some(&yaml_content),
    vec![(BODHI_KEEP_ALIVE_SECS, "600")],
  )
  .await;

  let (value, source) = service
    .get_setting_value_with_source(BODHI_KEEP_ALIVE_SECS)
    .await;
  assert_eq!(SettingSource::Database, source);
  assert_eq!(Some(Value::Number(600.into())), value);
}

#[rstest]
#[tokio::test]
async fn test_file_over_default() {
  let temp_dir = TempDir::new().unwrap();
  let yaml_content = format!("{}: 1200", BODHI_KEEP_ALIVE_SECS);
  let service = make_service(&temp_dir, HashMap::new(), Some(&yaml_content), vec![]).await;

  let (value, source) = service
    .get_setting_value_with_source(BODHI_KEEP_ALIVE_SECS)
    .await;
  assert_eq!(SettingSource::SettingsFile, source);
  assert_eq!(Some(Value::Number(1200.into())), value);
}

#[rstest]
#[tokio::test]
async fn test_set_then_get_via_database() {
  let temp_dir = TempDir::new().unwrap();
  let service = make_service(&temp_dir, HashMap::new(), None, vec![]).await;

  service
    .set_setting_with_source(
      BODHI_KEEP_ALIVE_SECS,
      &Value::Number(900.into()),
      SettingSource::Database,
    )
    .await
    .unwrap();

  let (value, source) = service
    .get_setting_value_with_source(BODHI_KEEP_ALIVE_SECS)
    .await;
  assert_eq!(SettingSource::Database, source);
  assert_eq!(Some(Value::Number(900.into())), value);
}
