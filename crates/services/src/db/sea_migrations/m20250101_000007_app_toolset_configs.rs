use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AppToolsetConfigs {
  Table,
  Id,
  TenantId,
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
          .table(AppToolsetConfigs::Table)
          .col(string(AppToolsetConfigs::Id).primary_key())
          .col(string(AppToolsetConfigs::TenantId))
          .col(string(AppToolsetConfigs::ToolsetType))
          .col(boolean(AppToolsetConfigs::Enabled).default(false))
          .col(string(AppToolsetConfigs::UpdatedBy))
          .col(timestamp_with_time_zone(AppToolsetConfigs::CreatedAt))
          .col(timestamp_with_time_zone(AppToolsetConfigs::UpdatedAt))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_app_toolset_configs_tenant_id")
          .table(AppToolsetConfigs::Table)
          .col(AppToolsetConfigs::TenantId)
          .to_owned(),
      )
      .await?;

    // Composite unique index: (tenant_id, toolset_type)
    manager
      .create_index(
        Index::create()
          .name("idx_app_toolset_configs_tenant_toolset_type")
          .table(AppToolsetConfigs::Table)
          .col(AppToolsetConfigs::TenantId)
          .col(AppToolsetConfigs::ToolsetType)
          .unique()
          .to_owned(),
      )
      .await?;

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE app_toolset_configs ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE app_toolset_configs FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY tenant_isolation ON app_toolset_configs
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
        .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON app_toolset_configs;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE app_toolset_configs DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(AppToolsetConfigs::Table).to_owned())
      .await
  }
}
