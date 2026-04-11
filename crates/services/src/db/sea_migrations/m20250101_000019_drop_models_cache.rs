use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ApiModelAliases {
  Table,
  ModelsCache,
  CacheFetchedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .drop_column(ApiModelAliases::ModelsCache)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .drop_column(ApiModelAliases::CacheFetchedAt)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .add_column(ColumnDef::new(ApiModelAliases::ModelsCache).text().null())
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .add_column(
            ColumnDef::new(ApiModelAliases::CacheFetchedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .to_owned(),
      )
      .await?;

    Ok(())
  }
}
