use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AppToolsetConfigs {
  Table,
}

#[derive(DeriveIden)]
enum Toolsets {
  Table,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Drop RLS policies first on PostgreSQL
    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      for table in &["app_toolset_configs", "toolsets"] {
        conn
          .execute_unprepared(&format!(
            "DROP POLICY IF EXISTS tenant_isolation ON {table};"
          ))
          .await?;
        conn
          .execute_unprepared(&format!(
            "ALTER TABLE IF EXISTS {table} DISABLE ROW LEVEL SECURITY;"
          ))
          .await?;
      }
    }

    // Drop app_toolset_configs first (FK references toolsets)
    manager
      .drop_table(
        Table::drop()
          .table(AppToolsetConfigs::Table)
          .if_exists()
          .to_owned(),
      )
      .await?;

    manager
      .drop_table(Table::drop().table(Toolsets::Table).if_exists().to_owned())
      .await?;

    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    // No-op: toolset tables are permanently removed
    Ok(())
  }
}
