use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AppAccessRequests {
  Table,
  FlowType,
  RedirectUri,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // SQLite requires a separate ALTER TABLE per column.
    manager
      .alter_table(
        Table::alter()
          .table(AppAccessRequests::Table)
          .drop_column(AppAccessRequests::FlowType)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(AppAccessRequests::Table)
          .drop_column(AppAccessRequests::RedirectUri)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    // No-op: columns are permanently removed by the single-step OAuth access-request flow.
    Ok(())
  }
}
