use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ModelMetadata {
  Table,
  Id,
  Source,
  Repo,
  Filename,
  Snapshot,
  ApiModelId,
  Capabilities,
  Context,
  Architecture,
  AdditionalMetadata,
  ChatTemplate,
  ExtractedAt,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ModelMetadata::Table)
          .col(string(ModelMetadata::Id).primary_key())
          .col(string(ModelMetadata::Source))
          .col(string_null(ModelMetadata::Repo))
          .col(string_null(ModelMetadata::Filename))
          .col(string_null(ModelMetadata::Snapshot))
          .col(string_null(ModelMetadata::ApiModelId))
          .col(
            ColumnDef::new(ModelMetadata::Capabilities)
              .json_binary()
              .null(),
          )
          .col(ColumnDef::new(ModelMetadata::Context).json_binary().null())
          .col(
            ColumnDef::new(ModelMetadata::Architecture)
              .json_binary()
              .null(),
          )
          .col(string_null(ModelMetadata::AdditionalMetadata))
          .col(string_null(ModelMetadata::ChatTemplate))
          .col(timestamp_with_time_zone(ModelMetadata::ExtractedAt))
          .col(timestamp_with_time_zone(ModelMetadata::CreatedAt))
          .col(timestamp_with_time_zone(ModelMetadata::UpdatedAt))
          .to_owned(),
      )
      .await?;

    // Unique constraint on (source, repo, filename, snapshot, api_model_id)
    manager
      .create_index(
        Index::create()
          .name("idx_model_metadata_unique_key")
          .table(ModelMetadata::Table)
          .col(ModelMetadata::Source)
          .col(ModelMetadata::Repo)
          .col(ModelMetadata::Filename)
          .col(ModelMetadata::Snapshot)
          .col(ModelMetadata::ApiModelId)
          .unique()
          .to_owned(),
      )
      .await?;

    // Indexes for common query patterns
    manager
      .create_index(
        Index::create()
          .name("idx_model_metadata_source")
          .table(ModelMetadata::Table)
          .col(ModelMetadata::Source)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_model_metadata_repo")
          .table(ModelMetadata::Table)
          .col(ModelMetadata::Repo)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_model_metadata_api_model_id")
          .table(ModelMetadata::Table)
          .col(ModelMetadata::ApiModelId)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(ModelMetadata::Table).to_owned())
      .await
  }
}
