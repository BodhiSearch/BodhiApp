use crate::{DefaultSessionService, SessionService, SessionStoreBackend};
use serde_json::Value;
use std::{fs::File, path::PathBuf, str::FromStr};
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};

impl DefaultSessionService {
  pub async fn build_session_service(dbfile: PathBuf) -> DefaultSessionService {
    if !dbfile.exists() {
      File::create(&dbfile).expect("Failed to create database file");
    }
    let url = format!("sqlite:{}", dbfile.display());
    DefaultSessionService::connect_sqlite(&url).await.unwrap()
  }

  pub async fn build_pg_session_service(url: &str) -> DefaultSessionService {
    DefaultSessionService::connect_postgres(url).await.unwrap()
  }
}

#[async_trait::async_trait]
pub trait SessionTestExt {
  async fn get_session_value(&self, session_id: &str, key: &str) -> Option<Value>;

  async fn get_session_record(&self, session_id: &str) -> Option<Record>;
}

#[async_trait::async_trait]
impl SessionTestExt for DefaultSessionService {
  async fn get_session_value(&self, session_id: &str, key: &str) -> Option<Value> {
    let record = self.get_session_record(session_id).await.unwrap();
    record.data.get(key).cloned()
  }

  async fn get_session_record(&self, session_id: &str) -> Option<Record> {
    self
      .get_session_store()
      .load(&Id::from_str(session_id).unwrap())
      .await
      .unwrap()
  }
}

#[async_trait::async_trait]
impl SessionTestExt for SessionStoreBackend {
  async fn get_session_value(&self, session_id: &str, key: &str) -> Option<Value> {
    let record = self.get_session_record(session_id).await.unwrap();
    record.data.get(key).cloned()
  }

  async fn get_session_record(&self, session_id: &str) -> Option<Record> {
    self.load(&Id::from_str(session_id).unwrap()).await.unwrap()
  }
}
