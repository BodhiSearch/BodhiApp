use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ModelMetadata {
  Table,
  Id,
  TenantId,
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
          .col(string(ModelMetadata::TenantId))
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

    // Unique constraint on (tenant_id, source, repo, filename, snapshot, api_model_id)
    manager
      .create_index(
        Index::create()
          .name("idx_model_metadata_unique_key")
          .table(ModelMetadata::Table)
          .col(ModelMetadata::TenantId)
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
          .name("idx_model_metadata_tenant_id")
          .table(ModelMetadata::Table)
          .col(ModelMetadata::TenantId)
          .to_owned(),
      )
      .await?;

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

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE model_metadata ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE model_metadata FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY tenant_isolation ON model_metadata
             FOR ALL
             USING (tenant_id = (SELECT current_tenant_id()))
             WITH CHECK (tenant_id = (SELECT current_tenant_id()));",
        )
        .await?;
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON model_metadata;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE model_metadata DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(ModelMetadata::Table).to_owned())
      .await?;
    Ok(())
  }
}
