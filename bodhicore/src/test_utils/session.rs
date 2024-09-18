use crate::service::SqliteSessionService;
use serde_json::Value;
use sqlx::SqlitePool;
use std::{fs::File, path::PathBuf, str::FromStr};
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};

impl SqliteSessionService {
  pub async fn build_session_service(dbfile: PathBuf) -> SqliteSessionService {
    if !dbfile.exists() {
      File::create(&dbfile).expect("Failed to create database file");
    }
    let pool = SqlitePool::connect(&format!("sqlite:{}", dbfile.display()))
      .await
      .unwrap();
    let session_service = SqliteSessionService::new(pool);
    session_service.migrate().await.unwrap();
    session_service
  }
}

pub trait SessionTestExt {
  async fn get_session_value(&self, session_id: &str, key: &str) -> Option<Value>;

  async fn get_session_record(&self, session_id: &str) -> Option<Record>;
}

impl SessionTestExt for SqliteSessionService {
  async fn get_session_value(&self, session_id: &str, key: &str) -> Option<Value> {
    let record = self.get_session_record(session_id).await.unwrap();
    record.data.get(key).cloned()
  }

  async fn get_session_record(&self, session_id: &str) -> Option<Record> {
    self
      .session_store
      .load(&Id::from_str(session_id).unwrap())
      .await
      .unwrap()
  }
}
