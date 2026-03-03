use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum McpServers {
  Table,
  Id,
  TenantId,
  Url,
  Name,
  Description,
  Enabled,
  CreatedBy,
  UpdatedBy,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum Mcps {
  Table,
  Id,
  TenantId,
  UserId,
  McpServerId,
  Name,
  Slug,
  Description,
  Enabled,
  ToolsCache,
  ToolsFilter,
  AuthType,
  AuthUuid,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(McpServers::Table)
          .col(string(McpServers::Id).primary_key())
          .col(string(McpServers::TenantId))
          .col(string(McpServers::Url))
          .col(string(McpServers::Name).default(""))
          .col(string_null(McpServers::Description))
          .col(boolean(McpServers::Enabled).default(false))
          .col(string(McpServers::CreatedBy))
          .col(string(McpServers::UpdatedBy))
          .col(timestamp_with_time_zone(McpServers::CreatedAt))
          .col(timestamp_with_time_zone(McpServers::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_servers_tenant_id")
          .table(McpServers::Table)
          .col(McpServers::TenantId)
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(Mcps::Table)
          .col(string(Mcps::Id).primary_key())
          .col(string(Mcps::TenantId))
          .col(string(Mcps::UserId))
          .col(string(Mcps::McpServerId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcps_mcp_server_id")
              .from(Mcps::Table, Mcps::McpServerId)
              .to(McpServers::Table, McpServers::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(Mcps::Name))
          .col(string(Mcps::Slug))
          .col(string_null(Mcps::Description))
          .col(boolean(Mcps::Enabled).default(true))
          .col(string_null(Mcps::ToolsCache))
          .col(string_null(Mcps::ToolsFilter))
          .col(string(Mcps::AuthType).default("public"))
          .col(string_null(Mcps::AuthUuid))
          .col(timestamp_with_time_zone(Mcps::CreatedAt))
          .col(timestamp_with_time_zone(Mcps::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcps_user_id")
          .table(Mcps::Table)
          .col(Mcps::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcps_mcp_server_id")
          .table(Mcps::Table)
          .col(Mcps::McpServerId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcps_tenant_id")
          .table(Mcps::Table)
          .col(Mcps::TenantId)
          .to_owned(),
      )
      .await?;

    let db = manager.get_connection();
    let backend = db.get_database_backend();
    match backend {
      sea_orm::DatabaseBackend::Sqlite => {
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_servers_url_unique \
           ON mcp_servers(tenant_id, url COLLATE NOCASE)",
        )
        .await?;
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcps_user_id_slug_unique \
           ON mcps(tenant_id, user_id COLLATE NOCASE, slug COLLATE NOCASE)",
        )
        .await?;
      }
      sea_orm::DatabaseBackend::Postgres => {
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_servers_url_unique \
           ON mcp_servers(tenant_id, LOWER(url))",
        )
        .await?;
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcps_user_id_slug_unique \
           ON mcps(tenant_id, LOWER(user_id), LOWER(slug))",
        )
        .await?;
      }
      _ => {}
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Mcps::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(McpServers::Table).to_owned())
      .await
  }
}
