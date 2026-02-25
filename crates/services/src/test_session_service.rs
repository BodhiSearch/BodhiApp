use crate::session_service::{AppSessionStoreExt, DefaultSessionService, SessionService};
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serial_test::serial;
use std::{collections::HashMap, path::PathBuf};
use tempfile::TempDir;
use time::OffsetDateTime;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};

fn pg_url() -> String {
  let env_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/.env.test"));
  if env_path.exists() {
    let _ = dotenv::from_filename(env_path).ok();
  }
  std::env::var("INTEG_TEST_SESSION_PG_URL")
    .expect("INTEG_TEST_SESSION_PG_URL must be set for postgres integration tests")
}

async fn create_session_service(backend: &str) -> (DefaultSessionService, Option<TempDir>) {
  match backend {
    "sqlite" => {
      let temp_dir = tempfile::TempDir::new().unwrap();
      let db_path = temp_dir.path().join("test_sessions.sqlite");
      std::fs::File::create(&db_path).unwrap();
      let url = format!("sqlite:{}", db_path.display());
      let service = DefaultSessionService::connect_sqlite(&url).await.unwrap();
      (service, Some(temp_dir))
    }
    "postgres" => {
      let service = DefaultSessionService::connect_postgres(&pg_url())
        .await
        .unwrap();
      AppSessionStoreExt::clear_all_sessions(&service)
        .await
        .unwrap();
      (service, None)
    }
    other => panic!("unsupported backend: {}", other),
  }
}

fn make_record(user_id: Option<&str>) -> Record {
  let session_id = Id::default();
  let mut data = HashMap::new();
  if let Some(uid) = user_id {
    data.insert(
      "user_id".to_string(),
      serde_json::Value::String(uid.to_string()),
    );
  }
  data.insert(
    "test_key".to_string(),
    serde_json::Value::String("test_value".to_string()),
  );
  Record {
    id: session_id,
    data,
    expiry_date: OffsetDateTime::now_utc() + time::Duration::hours(1),
  }
}

#[rstest]
#[case::sqlite("sqlite")]
#[case::postgres("postgres")]
#[tokio::test]
#[serial(pg_session)]
#[anyhow_trace]
async fn test_session_service_save_with_user_id(#[case] backend: &str) -> anyhow::Result<()> {
  let (service, _temp_dir) = create_session_service(backend).await;
  let store = service.get_session_store();
  let record = make_record(Some("user123"));
  let session_id = record.id;

  store.save(&record).await?;

  let sessions = AppSessionStoreExt::dump_all_sessions(&service).await?;
  let found = sessions
    .iter()
    .find(|(id, _)| id == &session_id.to_string());
  assert!(found.is_some(), "session should exist in store");
  assert_eq!(
    Some("user123".to_string()),
    found.unwrap().1,
    "user_id should be stored"
  );
  Ok(())
}

#[rstest]
#[case::sqlite("sqlite")]
#[case::postgres("postgres")]
#[tokio::test]
#[serial(pg_session)]
#[anyhow_trace]
async fn test_session_service_save_without_user_id(#[case] backend: &str) -> anyhow::Result<()> {
  let (service, _temp_dir) = create_session_service(backend).await;
  let store = service.get_session_store();
  let record = make_record(None);
  let session_id = record.id;

  store.save(&record).await?;

  let sessions = AppSessionStoreExt::dump_all_sessions(&service).await?;
  let found = sessions
    .iter()
    .find(|(id, _)| id == &session_id.to_string());
  assert!(found.is_some(), "session should exist in store");
  assert_eq!(None, found.unwrap().1, "user_id should be null");
  Ok(())
}

#[rstest]
#[case::sqlite("sqlite")]
#[case::postgres("postgres")]
#[tokio::test]
#[serial(pg_session)]
#[anyhow_trace]
async fn test_session_service_load_and_delete(#[case] backend: &str) -> anyhow::Result<()> {
  let (service, _temp_dir) = create_session_service(backend).await;
  let store = service.get_session_store();
  let record = make_record(Some("user_load"));

  store.save(&record).await?;

  let loaded = store.load(&record.id).await?;
  assert!(loaded.is_some(), "session should be loadable");
  assert_eq!(record.id, loaded.unwrap().id);

  store.delete(&record.id).await?;
  let loaded_after = store.load(&record.id).await?;
  assert!(loaded_after.is_none(), "session should be deleted");
  Ok(())
}

#[rstest]
#[case::sqlite("sqlite")]
#[case::postgres("postgres")]
#[tokio::test]
#[serial(pg_session)]
#[anyhow_trace]
async fn test_session_service_clear_sessions_for_user(
  #[case] backend: &str,
) -> anyhow::Result<()> {
  let (service, _temp_dir) = create_session_service(backend).await;
  let store = service.get_session_store();

  for _ in 0..3 {
    let record = make_record(Some("user_clear"));
    store.save(&record).await?;
  }
  let other = make_record(Some("user_other"));
  store.save(&other).await?;

  let cleared = AppSessionStoreExt::clear_sessions_for_user(&service, "user_clear").await?;
  assert_eq!(3, cleared);

  let remaining =
    AppSessionStoreExt::count_sessions_for_user(&service, "user_clear").await?;
  assert_eq!(0, remaining);

  let other_remaining =
    AppSessionStoreExt::count_sessions_for_user(&service, "user_other").await?;
  assert_eq!(1, other_remaining);
  Ok(())
}

#[rstest]
#[case::sqlite("sqlite")]
#[case::postgres("postgres")]
#[tokio::test]
#[serial(pg_session)]
#[anyhow_trace]
async fn test_session_service_clear_all_sessions(#[case] backend: &str) -> anyhow::Result<()> {
  let (service, _temp_dir) = create_session_service(backend).await;
  let store = service.get_session_store();

  for uid in &["u1", "u2", "u3"] {
    let record = make_record(Some(uid));
    store.save(&record).await?;
  }

  let cleared = AppSessionStoreExt::clear_all_sessions(&service).await?;
  assert_eq!(3, cleared);

  let all = AppSessionStoreExt::dump_all_sessions(&service).await?;
  assert!(all.is_empty());
  Ok(())
}

#[rstest]
#[case::sqlite("sqlite")]
#[case::postgres("postgres")]
#[tokio::test]
#[serial(pg_session)]
#[anyhow_trace]
async fn test_session_service_count_and_get_ids(#[case] backend: &str) -> anyhow::Result<()> {
  let (service, _temp_dir) = create_session_service(backend).await;
  let store = service.get_session_store();

  let mut expected_ids = Vec::new();
  for _ in 0..2 {
    let record = make_record(Some("user_ids"));
    expected_ids.push(record.id.to_string());
    store.save(&record).await?;
  }

  let count = AppSessionStoreExt::count_sessions_for_user(&service, "user_ids").await?;
  assert_eq!(2, count);

  let mut actual_ids =
    AppSessionStoreExt::get_session_ids_for_user(&service, "user_ids").await?;
  actual_ids.sort();
  expected_ids.sort();
  assert_eq!(expected_ids, actual_ids);
  Ok(())
}

#[rstest]
#[case::sqlite("sqlite")]
#[case::postgres("postgres")]
#[tokio::test]
#[serial(pg_session)]
#[anyhow_trace]
async fn test_session_service_multi_user_isolation(#[case] backend: &str) -> anyhow::Result<()> {
  let (service, _temp_dir) = create_session_service(backend).await;
  let store = service.get_session_store();

  for uid in &["alice", "alice", "bob"] {
    let record = make_record(Some(uid));
    store.save(&record).await?;
  }

  let alice_count =
    AppSessionStoreExt::count_sessions_for_user(&service, "alice").await?;
  let bob_count = AppSessionStoreExt::count_sessions_for_user(&service, "bob").await?;
  assert_eq!(2, alice_count);
  assert_eq!(1, bob_count);

  AppSessionStoreExt::clear_sessions_for_user(&service, "alice").await?;
  let alice_after =
    AppSessionStoreExt::count_sessions_for_user(&service, "alice").await?;
  let bob_after = AppSessionStoreExt::count_sessions_for_user(&service, "bob").await?;
  assert_eq!(0, alice_after);
  assert_eq!(1, bob_after);
  Ok(())
}
