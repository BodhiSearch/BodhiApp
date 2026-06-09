use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ModelRouterAliases {
  Table,
  Id,
  TenantId,
  UserId,
  Alias,
  Targets,
  Strategy,
  CreatedAt,
  UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ModelRouterAliases::Table)
          .col(
            ColumnDef::new(ModelRouterAliases::Id)
              .string()
              .not_null()
              .primary_key(),
          )
          .col(
            ColumnDef::new(ModelRouterAliases::TenantId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(ModelRouterAliases::UserId)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(ModelRouterAliases::Alias)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(ModelRouterAliases::Targets)
              .json_binary()
              .not_null(),
          )
          .col(
            ColumnDef::new(ModelRouterAliases::Strategy)
              .json_binary()
              .not_null(),
          )
          .col(
            ColumnDef::new(ModelRouterAliases::CreatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .col(
            ColumnDef::new(ModelRouterAliases::UpdatedAt)
              .timestamp_with_time_zone()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_model_router_aliases_tenant_id")
          .table(ModelRouterAliases::Table)
          .col(ModelRouterAliases::TenantId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_model_router_aliases_user_id")
          .table(ModelRouterAliases::Table)
          .col(ModelRouterAliases::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_model_router_aliases_updated_at")
          .table(ModelRouterAliases::Table)
          .col(ModelRouterAliases::UpdatedAt)
          .to_owned(),
      )
      .await?;

    let db = manager.get_connection();
    db.execute_unprepared(
      "CREATE UNIQUE INDEX IF NOT EXISTS idx_model_router_aliases_alias_unique ON model_router_aliases(tenant_id, user_id, alias)"
    ).await?;

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE model_router_aliases ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE model_router_aliases FORCE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared(
          "CREATE POLICY tenant_isolation ON model_router_aliases
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
        .execute_unprepared("DROP POLICY IF EXISTS tenant_isolation ON model_router_aliases;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE model_router_aliases DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(ModelRouterAliases::Table).to_owned())
      .await?;
    Ok(())
  }
}
