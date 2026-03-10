use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum TenantsUsers {
  Table,
  TenantId,
  UserId,
  CreatedAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum Tenants {
  Table,
  Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(TenantsUsers::Table)
          .col(string(TenantsUsers::TenantId))
          .col(string(TenantsUsers::UserId))
          .col(timestamp_with_time_zone(TenantsUsers::CreatedAt))
          .col(timestamp_with_time_zone(TenantsUsers::UpdatedAt))
          .primary_key(
            Index::create()
              .col(TenantsUsers::TenantId)
              .col(TenantsUsers::UserId),
          )
          .foreign_key(
            ForeignKey::create()
              .from(TenantsUsers::Table, TenantsUsers::TenantId)
              .to(Tenants::Table, Tenants::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_tenants_users_user_id")
          .table(TenantsUsers::Table)
          .col(TenantsUsers::UserId)
          .to_owned(),
      )
      .await?;

    // RLS for tenants_users: allow cross-tenant reads (membership lookups),
    // restrict mutations to current tenant context.
    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("ALTER TABLE tenants_users ENABLE ROW LEVEL SECURITY;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE tenants_users FORCE ROW LEVEL SECURITY;")
        .await?;
      // Cross-tenant reads allowed (list_user_tenants, has_tenant_memberships)
      conn
        .execute_unprepared(
          "CREATE POLICY tenants_users_read ON tenants_users FOR SELECT USING (true);",
        )
        .await?;
      // Mutations restricted to current tenant context
      conn
        .execute_unprepared(
          "CREATE POLICY tenants_users_mutation ON tenants_users
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
        .execute_unprepared("DROP POLICY IF EXISTS tenants_users_mutation ON tenants_users;")
        .await?;
      conn
        .execute_unprepared("DROP POLICY IF EXISTS tenants_users_read ON tenants_users;")
        .await?;
      conn
        .execute_unprepared("ALTER TABLE tenants_users DISABLE ROW LEVEL SECURITY;")
        .await?;
    }
    manager
      .drop_table(Table::drop().table(TenantsUsers::Table).to_owned())
      .await
  }
}
