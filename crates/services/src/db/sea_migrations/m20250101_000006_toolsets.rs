use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Toolsets {
  Table,
  Id,
  UserId,
  ToolsetType,
  Slug,
  Description,
  Enabled,
  EncryptedApiKey,
  Salt,
  Nonce,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum AppToolsetConfigs {
  Table,
  Id,
  ToolsetType,
  Enabled,
  UpdatedBy,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Toolsets::Table)
          .col(string(Toolsets::Id).primary_key())
          .col(string(Toolsets::UserId))
          .col(string(Toolsets::ToolsetType))
          .col(string(Toolsets::Slug))
          .col(string_null(Toolsets::Description))
          .col(boolean(Toolsets::Enabled).default(false))
          .col(string_null(Toolsets::EncryptedApiKey))
          .col(string_null(Toolsets::Salt))
          .col(string_null(Toolsets::Nonce))
          .col(timestamp_with_time_zone(Toolsets::CreatedAt))
          .col(timestamp_with_time_zone(Toolsets::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_toolsets_user_id")
          .table(Toolsets::Table)
          .col(Toolsets::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_toolsets_toolset_type")
          .table(Toolsets::Table)
          .col(Toolsets::ToolsetType)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_toolsets_user_toolset_type")
          .table(Toolsets::Table)
          .col(Toolsets::UserId)
          .col(Toolsets::ToolsetType)
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(AppToolsetConfigs::Table)
          .col(string(AppToolsetConfigs::Id).primary_key())
          .col(string(AppToolsetConfigs::ToolsetType).unique_key())
          .col(boolean(AppToolsetConfigs::Enabled).default(false))
          .col(string(AppToolsetConfigs::UpdatedBy))
          .col(timestamp_with_time_zone(AppToolsetConfigs::CreatedAt))
          .col(timestamp_with_time_zone(AppToolsetConfigs::UpdatedAt))
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(AppToolsetConfigs::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(Toolsets::Table).to_owned())
      .await
  }
}
