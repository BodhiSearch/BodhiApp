use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let backend = manager.get_database_backend();
    if backend == sea_orm::DatabaseBackend::Postgres {
      manager
        .get_connection()
        .execute_unprepared("CREATE EXTENSION IF NOT EXISTS citext")
        .await?;
    }
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let backend = manager.get_database_backend();
    if backend == sea_orm::DatabaseBackend::Postgres {
      manager
        .get_connection()
        .execute_unprepared("DROP EXTENSION IF EXISTS citext")
        .await?;
    }
    Ok(())
  }
}
