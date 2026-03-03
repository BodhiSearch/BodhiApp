use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const TENANT_TABLES: &[&str] = &[
  "download_requests",
  "api_model_aliases",
  "model_metadata",
  "user_access_requests",
  "api_tokens",
  "toolsets",
  "app_toolset_configs",
  "user_aliases",
  "app_access_requests",
  "mcp_servers",
  "mcps",
  "mcp_auth_headers",
  "mcp_oauth_configs",
  "mcp_oauth_tokens",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    if manager.get_database_backend() != sea_orm::DatabaseBackend::Postgres {
      return Ok(());
    }
    let conn = manager.get_connection();

    // Create current_tenant_id() function — reads session var, null if unset
    conn
      .execute_unprepared(
        r#"
            CREATE OR REPLACE FUNCTION current_tenant_id() RETURNS TEXT AS $$
              SELECT NULLIF(current_setting('app.current_tenant_id', true), '')
            $$ LANGUAGE SQL SECURITY DEFINER STABLE;
        "#,
      )
      .await?;

    // Enable RLS + FORCE RLS on each tenant-scoped table, then add isolation policy
    for table in TENANT_TABLES {
      conn
        .execute_unprepared(&format!("ALTER TABLE {table} ENABLE ROW LEVEL SECURITY;"))
        .await?;
      conn
        .execute_unprepared(&format!("ALTER TABLE {table} FORCE ROW LEVEL SECURITY;"))
        .await?;
      conn
        .execute_unprepared(&format!(
          "CREATE POLICY tenant_isolation ON {table}
             FOR ALL
             USING (tenant_id = (SELECT current_tenant_id()))
             WITH CHECK (tenant_id = (SELECT current_tenant_id()));"
        ))
        .await?;
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    if manager.get_database_backend() != sea_orm::DatabaseBackend::Postgres {
      return Ok(());
    }
    let conn = manager.get_connection();

    for table in TENANT_TABLES.iter().rev() {
      conn
        .execute_unprepared(&format!(
          "DROP POLICY IF EXISTS tenant_isolation ON {table};"
        ))
        .await?;
      conn
        .execute_unprepared(&format!("ALTER TABLE {table} DISABLE ROW LEVEL SECURITY;"))
        .await?;
    }
    conn
      .execute_unprepared("DROP FUNCTION IF EXISTS current_tenant_id();")
      .await?;

    Ok(())
  }
}
