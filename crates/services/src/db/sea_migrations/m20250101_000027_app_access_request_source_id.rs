use sea_orm_migration::prelude::*;

/// Adds a nullable `source_access_request_id` to `app_access_requests`. Set on an
/// upgrade/exchange draft to the id of the prior approved access request being
/// elevated; the review page loads that request's approved grant to pre-select the
/// consent form. `NULL` for fresh (non-exchange) requests.
///
/// `down()` requires SQLite ≥ 3.35 for `DROP COLUMN` (satisfied by bundled `libsqlite3-sys`).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AppAccessRequests {
  Table,
  SourceAccessRequestId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(AppAccessRequests::Table)
          .add_column(
            ColumnDef::new(AppAccessRequests::SourceAccessRequestId)
              .text()
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
          .table(AppAccessRequests::Table)
          .drop_column(AppAccessRequests::SourceAccessRequestId)
          .to_owned(),
      )
      .await
  }
}
