use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Mcps {
  Table,
  ToolsCache,
  ToolsFilter,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Mcps::Table)
          .drop_column(Mcps::ToolsCache)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Mcps::Table)
          .drop_column(Mcps::ToolsFilter)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    // No-op: columns are permanently removed
    Ok(())
  }
}
