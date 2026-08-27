use sea_orm_migration::prelude::*;

/// Data-only migration for the OAuth consent-flow cutover. Draft/failed rows belong to the
/// removed pre-create flow and can never progress; approved rows whose scope predates the
/// dotted `scope_access_request:<resource-client-id>.<uuid>` format get no audience from the
/// stateless Keycloak mapper and can never validate again — marking them revoked stops the
/// Connected Apps screen advertising dead grants. No DDL.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    conn
      .execute_unprepared(
        "UPDATE app_access_requests SET status = 'expired' WHERE status IN ('draft', 'failed')",
      )
      .await?;
    conn
      .execute_unprepared(
        "UPDATE app_access_requests SET status = 'revoked' \
         WHERE status = 'approved' \
         AND (access_request_scope IS NULL OR access_request_scope NOT LIKE '%.%')",
      )
      .await?;
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    // Data-only forward step; the prior statuses are not recoverable.
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use anyhow_trace::anyhow_trace;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
  use sea_orm_migration::MigratorTrait;

  use crate::db::sea_migrations::Migrator;

  /// Migrates up to just before 000028, seeds legacy rows, then finishes the chain
  /// so the data-only UPDATEs actually run against pre-cutover data.
  #[rstest]
  #[tokio::test]
  #[anyhow_trace]
  async fn test_m20250101_000028_expires_stale_rows() -> anyhow::Result<()> {
    let db = Database::connect("sqlite::memory:").await?;
    // 28 = number of migrations preceding this one in `migrations()`.
    Migrator::up(&db, Some(28)).await?;

    for (id, status, scope) in [
      ("ar-draft", "draft", "NULL"),
      ("ar-failed", "failed", "NULL"),
      (
        "ar-approved-undotted",
        "approved",
        "'scope_access_request:legacy-id'",
      ),
      ("ar-approved-null-scope", "approved", "NULL"),
      (
        "ar-approved-dotted",
        "approved",
        "'scope_access_request:rc.01AR'",
      ),
      ("ar-denied", "denied", "NULL"),
    ] {
      db.execute_unprepared(&format!(
        "INSERT INTO app_access_requests \
         (id, app_client_id, status, requested, requested_role, access_request_scope, \
          expires_at, created_at, updated_at) \
         VALUES ('{id}', 'legacy-client', '{status}', '{{\"version\":\"1\"}}', 'scope_user_user', {scope}, \
          '2025-01-01 00:00:00 +00:00', '2025-01-01 00:00:00 +00:00', '2025-01-01 00:00:00 +00:00')"
      ))
      .await?;
    }

    Migrator::up(&db, None).await?;

    let rows = db
      .query_all(Statement::from_string(
        DbBackend::Sqlite,
        "SELECT id, status FROM app_access_requests ORDER BY id",
      ))
      .await?;
    let mut actual = Vec::new();
    for row in rows {
      actual.push((
        row.try_get::<String>("", "id")?,
        row.try_get::<String>("", "status")?,
      ));
    }
    assert_eq!(
      vec![
        ("ar-approved-dotted".to_string(), "approved".to_string()),
        ("ar-approved-null-scope".to_string(), "revoked".to_string()),
        ("ar-approved-undotted".to_string(), "revoked".to_string()),
        ("ar-denied".to_string(), "denied".to_string()),
        ("ar-draft".to_string(), "expired".to_string()),
        ("ar-failed".to_string(), "expired".to_string()),
      ],
      actual
    );

    Ok(())
  }
}
