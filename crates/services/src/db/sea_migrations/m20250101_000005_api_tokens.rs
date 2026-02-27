use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ApiTokens {
  Table,
  Id,
  UserId,
  Name,
  TokenPrefix,
  TokenHash,
  Scopes,
  Status,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ApiTokens::Table)
          .col(string(ApiTokens::Id).primary_key())
          .col(string(ApiTokens::UserId))
          .col(string(ApiTokens::Name).default(""))
          .col(string(ApiTokens::TokenPrefix).unique_key())
          .col(string(ApiTokens::TokenHash))
          .col(string(ApiTokens::Scopes))
          .col(string(ApiTokens::Status))
          .col(timestamp_with_time_zone(ApiTokens::CreatedAt))
          .col(timestamp_with_time_zone(ApiTokens::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_api_tokens_token_prefix")
          .table(ApiTokens::Table)
          .col(ApiTokens::TokenPrefix)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(ApiTokens::Table).to_owned())
      .await
  }
}
