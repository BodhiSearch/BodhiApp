use sea_orm_migration::prelude::*;

/// Adds a nullable `archived_at` timestamp to `download_requests`. Archiving a terminal
/// (completed/failed) or queued download sets this column; the list endpoint filters
/// `archived_at IS NULL` so archived rows are excluded from the API response.
///
/// `down()` requires SQLite ≥ 3.35 for `DROP COLUMN` (satisfied by bundled `libsqlite3-sys`).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum DownloadRequests {
  Table,
  ArchivedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(DownloadRequests::Table)
          .add_column(
            ColumnDef::new(DownloadRequests::ArchivedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(DownloadRequests::Table)
          .drop_column(DownloadRequests::ArchivedAt)
          .to_owned(),
      )
      .await
  }
}
