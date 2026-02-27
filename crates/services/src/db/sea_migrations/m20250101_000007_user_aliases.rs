use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum UserAliases {
  Table,
  Id,
  Alias,
  Repo,
  Filename,
  Snapshot,
  RequestParams,
  ContextParams,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(UserAliases::Table)
          .col(string(UserAliases::Id).primary_key())
          .col(string(UserAliases::Alias).unique_key())
          .col(string(UserAliases::Repo))
          .col(string(UserAliases::Filename))
          .col(string(UserAliases::Snapshot))
          .col(json_binary(UserAliases::RequestParams))
          .col(json_binary(UserAliases::ContextParams))
          .col(timestamp_with_time_zone(UserAliases::CreatedAt))
          .col(timestamp_with_time_zone(UserAliases::UpdatedAt))
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(UserAliases::Table).to_owned())
      .await
  }
}
