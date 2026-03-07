use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Tenants {
  Table,
  Id,
  ClientId,
  EncryptedClientSecret,
  SaltClientSecret,
  NonceClientSecret,
  AppStatus,
  CreatedBy,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Tenants::Table)
          .col(string(Tenants::Id).primary_key())
          .col(string(Tenants::ClientId).unique_key())
          .col(string_null(Tenants::EncryptedClientSecret))
          .col(string_null(Tenants::SaltClientSecret))
          .col(string_null(Tenants::NonceClientSecret))
          .col(string(Tenants::AppStatus).default("setup"))
          .col(string_null(Tenants::CreatedBy))
          .col(timestamp_with_time_zone(Tenants::CreatedAt))
          .col(timestamp_with_time_zone(Tenants::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_tenants_client_id")
          .table(Tenants::Table)
          .col(Tenants::ClientId)
          .unique()
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Tenants::Table).to_owned())
      .await
  }
}
