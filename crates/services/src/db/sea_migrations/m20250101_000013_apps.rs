use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Apps {
  Table,
  ClientId,
  EncryptedClientSecret,
  SaltClientSecret,
  NonceClientSecret,
  AppStatus,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Apps::Table)
          .col(string(Apps::ClientId).primary_key())
          .col(string(Apps::EncryptedClientSecret))
          .col(string(Apps::SaltClientSecret))
          .col(string(Apps::NonceClientSecret))
          .col(string(Apps::AppStatus).default("setup"))
          .col(timestamp_with_time_zone(Apps::CreatedAt))
          .col(timestamp_with_time_zone(Apps::UpdatedAt))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Apps::Table).to_owned())
      .await
  }
}
