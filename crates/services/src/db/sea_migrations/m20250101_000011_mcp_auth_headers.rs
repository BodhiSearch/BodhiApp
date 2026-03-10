use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum McpServers {
  Table,
  Id,
}

#[derive(DeriveIden)]
enum McpAuthHeaders {
  Table,
  Id,
  TenantId,
  Name,
  McpServerId,
  HeaderKey,
  EncryptedHeaderValue,
  HeaderValueSalt,
  HeaderValueNonce,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(McpAuthHeaders::Table)
          .col(string(McpAuthHeaders::Id).primary_key())
          .col(string(McpAuthHeaders::TenantId))
          .col(string(McpAuthHeaders::Name).default("Header"))
          .col(string(McpAuthHeaders::McpServerId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_mcp_auth_headers_mcp_server_id")
              .from(McpAuthHeaders::Table, McpAuthHeaders::McpServerId)
              .to(McpServers::Table, McpServers::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .col(string(McpAuthHeaders::HeaderKey))
          .col(string(McpAuthHeaders::EncryptedHeaderValue))
          .col(string(McpAuthHeaders::HeaderValueSalt))
          .col(string(McpAuthHeaders::HeaderValueNonce))
          .col(timestamp_with_time_zone(McpAuthHeaders::CreatedAt))
          .col(timestamp_with_time_zone(McpAuthHeaders::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_auth_headers_mcp_server_id")
          .table(McpAuthHeaders::Table)
          .col(McpAuthHeaders::McpServerId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_mcp_auth_headers_tenant_id")
          .table(McpAuthHeaders::Table)
          .col(McpAuthHeaders::TenantId)
          .to_owned(),
      )
      .await?;

    let db = manager.get_connection();
    let backend = db.get_database_backend();
    match backend {
      sea_orm::DatabaseBackend::Sqlite => {
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_auth_headers_server_name_unique \
           ON mcp_auth_headers(mcp_server_id COLLATE NOCASE, name COLLATE NOCASE)",
        )
        .await?;
      }
      sea_orm::DatabaseBackend::Postgres => {
        db.execute_unprepared(
          "CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_auth_headers_server_name_unique \
           ON mcp_auth_headers(LOWER(mcp_server_id), LOWER(name))",
        )
        .await?;
      }
      _ => {}
    }

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE mcp_auth_headers ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE mcp_auth_headers FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY tenant_isolation ON mcp_auth_headers
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
        .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON mcp_auth_headers;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE mcp_auth_headers DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(McpAuthHeaders::Table).to_owned())
      .await
  }
}
