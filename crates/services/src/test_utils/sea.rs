use crate::{
  db::{sea_migrations::Migrator, DefaultDbService, TimeService},
  test_utils::FrozenTimeService,
};
use chrono::{DateTime, Utc};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use tempfile::TempDir;

pub struct SeaTestContext {
  pub _temp_dir: Option<TempDir>,
  pub service: DefaultDbService,
  pub now: DateTime<Utc>,
}

pub async fn sea_context(db_type: &str) -> SeaTestContext {
  match db_type {
    "sqlite" => {
      let temp_dir = TempDir::new().unwrap();
      let db_path = temp_dir.path().join("test.db");
      let url = format!("sqlite:{}?mode=rwc", db_path.display());
      let db = Database::connect(&url).await.unwrap();
      Migrator::fresh(&db).await.unwrap();

      let time_service = FrozenTimeService::default();
      let now = time_service.utc_now();
      let encryption_key = b"01234567890123456789012345678901".to_vec();

      let service = DefaultDbService::new(db, Arc::new(time_service), encryption_key);
      SeaTestContext {
        _temp_dir: Some(temp_dir),
        service,
        now,
      }
    }
    "postgres" => {
      let pg_url = std::env::var("INTEG_TEST_APP_DB_PG_URL")
        .expect("INTEG_TEST_APP_DB_PG_URL must be set for PostgreSQL tests");

      let db = Database::connect(&pg_url)
        .await
        .expect("Failed to connect to PostgreSQL");

      Migrator::fresh(&db).await.unwrap();

      let time_service = FrozenTimeService::default();
      let now = time_service.utc_now();
      let encryption_key = b"01234567890123456789012345678901".to_vec();

      let service = DefaultDbService::new(db, Arc::new(time_service), encryption_key);
      SeaTestContext {
        _temp_dir: None,
        service,
        now,
      }
    }
    _ => panic!("Unknown db_type: {}", db_type),
  }
}
