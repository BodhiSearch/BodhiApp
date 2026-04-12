use sea_orm_migration::prelude::*;

/// Nullable JSON columns on `api_model_aliases`. `down()` requires SQLite ≥ 3.35 for
/// `DROP COLUMN` (satisfied by bundled `libsqlite3-sys`).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ApiModelAliases {
  Table,
  ExtraHeaders,
  ExtraBody,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // SQLite requires separate ALTER TABLE statements per column
    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .add_column(
            ColumnDef::new(ApiModelAliases::ExtraHeaders)
              .json_binary()
              .null(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .add_column(
            ColumnDef::new(ApiModelAliases::ExtraBody)
              .json_binary()
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
          .table(ApiModelAliases::Table)
          .drop_column(ApiModelAliases::ExtraHeaders)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .drop_column(ApiModelAliases::ExtraBody)
          .to_owned(),
      )
      .await
  }
}
