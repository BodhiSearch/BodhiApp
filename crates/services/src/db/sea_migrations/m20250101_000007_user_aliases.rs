use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum UserAliases {
  Table,
  Id,
  TenantId,
  UserId,
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
          .col(string(UserAliases::TenantId))
          .col(string(UserAliases::UserId))
          .col(string(UserAliases::Alias))
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

    manager
      .create_index(
        Index::create()
          .name("idx_user_aliases_tenant_id")
          .table(UserAliases::Table)
          .col(UserAliases::TenantId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_user_aliases_user_id")
          .table(UserAliases::Table)
          .col(UserAliases::UserId)
          .to_owned(),
      )
      .await?;

    // Composite unique index: (tenant_id, user_id, alias)
    manager
      .create_index(
        Index::create()
          .name("idx_user_aliases_tenant_alias")
          .table(UserAliases::Table)
          .col(UserAliases::TenantId)
          .col(UserAliases::UserId)
          .col(UserAliases::Alias)
          .unique()
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
