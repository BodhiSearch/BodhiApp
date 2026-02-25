use async_trait::async_trait;
use sqlx::AnyPool;
use std::fmt;
use tower_sessions::{
  session::{Id, Record},
  SessionStore,
};
use tower_sessions_sqlx_store::{PostgresStore, SqliteStore};

/// Backend-agnostic session store that wraps typed tower-sessions stores
/// and uses AnyPool for custom user_id tracking queries.
#[derive(Clone)]
pub struct SessionStoreBackend {
  inner: StoreInner,
  any_pool: AnyPool,
}

#[derive(Clone)]
enum StoreInner {
  Sqlite(SqliteStore),
  Postgres(PostgresStore),
}

impl fmt::Debug for SessionStoreBackend {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let kind = match &self.inner {
      StoreInner::Sqlite(_) => "Sqlite",
      StoreInner::Postgres(_) => "Postgres",
    };
    f.debug_struct("SessionStoreBackend")
      .field("kind", &kind)
      .finish()
  }
}

#[async_trait]
impl SessionStore for SessionStoreBackend {
  async fn save(
    &self,
    record: &Record,
  ) -> std::result::Result<(), tower_sessions::session_store::Error> {
    let user_id = record.data.get("user_id").and_then(|v| v.as_str());

    match &self.inner {
      StoreInner::Sqlite(store) => store.save(record).await?,
      StoreInner::Postgres(store) => store.save(record).await?,
    }

    if let Some(user_id) = user_id {
      // Note: $1/$2 positional parameters work with SQLite via AnyPool because
      // sqlx's AnyPool translates PostgreSQL-style positional params to ?-style
      // for SQLite. Parameterized tests (test_session_service.rs) verify both backends.
      sqlx::query("UPDATE tower_sessions SET user_id = $1 WHERE id = $2")
        .bind(user_id)
        .bind(record.id.to_string())
        .execute(&self.any_pool)
        .await
        .map_err(|e| tower_sessions::session_store::Error::Backend(e.to_string()))?;
    }

    Ok(())
  }

  async fn load(
    &self,
    session_id: &Id,
  ) -> std::result::Result<Option<Record>, tower_sessions::session_store::Error> {
    match &self.inner {
      StoreInner::Sqlite(store) => store.load(session_id).await,
      StoreInner::Postgres(store) => store.load(session_id).await,
    }
  }

  async fn delete(
    &self,
    session_id: &Id,
  ) -> std::result::Result<(), tower_sessions::session_store::Error> {
    match &self.inner {
      StoreInner::Sqlite(store) => store.delete(session_id).await,
      StoreInner::Postgres(store) => store.delete(session_id).await,
    }
  }
}

pub(crate) fn is_postgres_url(url: &str) -> bool {
  url.starts_with("postgres://") || url.starts_with("postgresql://")
}

impl SessionStoreBackend {
  pub(crate) fn new_sqlite(store: SqliteStore, any_pool: AnyPool) -> Self {
    Self {
      inner: StoreInner::Sqlite(store),
      any_pool,
    }
  }

  pub(crate) fn new_postgres(store: PostgresStore, any_pool: AnyPool) -> Self {
    Self {
      inner: StoreInner::Postgres(store),
      any_pool,
    }
  }

  pub fn any_pool(&self) -> &AnyPool {
    &self.any_pool
  }
}
