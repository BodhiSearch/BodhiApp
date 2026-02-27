use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ApiModelAliases {
  Table,
  Id,
  ApiFormat,
  BaseUrl,
  Models,
  Prefix,
  ForwardAllWithPrefix,
  ModelsCache,
  CacheFetchedAt,
  EncryptedApiKey,
  Salt,
  Nonce,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ApiModelAliases::Table)
          .col(
            ColumnDef::new(ApiModelAliases::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(ApiModelAliases::ApiFormat)
              .string()
              .not_null(),
          )
          .col(ColumnDef::new(ApiModelAliases::BaseUrl).string().not_null())
          .col(
            ColumnDef::new(ApiModelAliases::Models)
              .json_binary()
              .not_null(),
          )
          .col(ColumnDef::new(ApiModelAliases::Prefix).string().null())
          .col(
            ColumnDef::new(ApiModelAliases::ForwardAllWithPrefix)
              .boolean()
              .not_null()
              .default(false),
          )
          .col(
            ColumnDef::new(ApiModelAliases::ModelsCache)
              .json_binary()
              .not_null()
              .default("[]"),
          )
          .col(
            ColumnDef::new(ApiModelAliases::CacheFetchedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(ApiModelAliases::EncryptedApiKey)
              .string()
              .null(),
          )
          .col(ColumnDef::new(ApiModelAliases::Salt).string().null())
          .col(ColumnDef::new(ApiModelAliases::Nonce).string().null())
          .col(
            ColumnDef::new(ApiModelAliases::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(ApiModelAliases::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_api_model_aliases_api_format")
          .table(ApiModelAliases::Table)
          .col(ApiModelAliases::ApiFormat)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_api_model_aliases_prefix")
          .table(ApiModelAliases::Table)
          .col(ApiModelAliases::Prefix)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_api_model_aliases_updated_at")
          .table(ApiModelAliases::Table)
          .col(ApiModelAliases::UpdatedAt)
          .to_owned(),
      )
      .await?;

    // Partial unique index on non-null prefix (raw SQL - identical for both backends)
    let db = manager.get_connection();
    db.execute_unprepared(
      "CREATE UNIQUE INDEX IF NOT EXISTS idx_api_model_aliases_prefix_unique ON api_model_aliases(prefix) WHERE prefix IS NOT NULL AND prefix != ''"
    ).await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(ApiModelAliases::Table).to_owned())
      .await
  }
}
