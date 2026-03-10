use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let backend = manager.get_database_backend();
    if backend == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("CREATE EXTENSION IF NOT EXISTS citext")
        .await?;
      conn
        .execute_unprepared(
          r#"
            CREATE OR REPLACE FUNCTION current_tenant_id() RETURNS TEXT AS $$
              SELECT NULLIF(current_setting('app.current_tenant_id', true), '')
            $$ LANGUAGE SQL SECURITY DEFINER STABLE;
          "#,
        )
        .await?;
    }
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let backend = manager.get_database_backend();
    if backend == sea_orm::DatabaseBackend::Postgres {
      let conn = manager.get_connection();
      conn
        .execute_unprepared("DROP FUNCTION IF EXISTS current_tenant_id()")
        .await?;
      conn
        .execute_unprepared("DROP EXTENSION IF EXISTS citext")
        .await?;
    }
    Ok(())
  }
}
