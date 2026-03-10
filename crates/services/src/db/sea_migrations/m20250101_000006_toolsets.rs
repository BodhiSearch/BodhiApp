use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Toolsets {
  Table,
  Id,
  TenantId,
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

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Toolsets::Table)
          .col(string(Toolsets::Id).primary_key())
          .col(string(Toolsets::TenantId))
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
      .create_index(
        Index::create()
          .name("idx_toolsets_tenant_id")
          .table(Toolsets::Table)
          .col(Toolsets::TenantId)
          .to_owned(),
      )
      .await?;

    // Unique composite index: (tenant_id, user_id, toolset_type, slug)
    manager
      .create_index(
        Index::create()
          .name("idx_toolsets_tenant_user_type_slug")
          .table(Toolsets::Table)
          .col(Toolsets::TenantId)
          .col(Toolsets::UserId)
          .col(Toolsets::ToolsetType)
          .col(Toolsets::Slug)
          .unique()
          .to_owned(),
      )
      .await?;

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE toolsets ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE toolsets FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY tenant_isolation ON toolsets
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
        .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON toolsets;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE toolsets DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(Toolsets::Table).to_owned())
      .await
  }
}
