use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Settings {
  Table,
  Key,
  Value,
  ValueType,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Settings::Table)
          .col(string(Settings::Key).primary_key())
          .col(string(Settings::Value))
          .col(string(Settings::ValueType))
          .col(timestamp_with_time_zone(Settings::CreatedAt))
          .col(timestamp_with_time_zone(Settings::UpdatedAt))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Settings::Table).to_owned())
      .await
  }
}
