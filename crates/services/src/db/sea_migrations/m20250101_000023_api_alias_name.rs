use sea_orm_migration::prelude::*;

/// Adds the mandatory `name` column to `api_model_aliases`. The column is added with a
/// transient `DEFAULT ''` so the ALTER succeeds on existing rows, then existing rows are
/// backfilled with their `id` (NOT NULL / NOT EMPTY contract is enforced app-side via
/// `length(min=1)` validation). On PostgreSQL the default is dropped afterwards; SQLite
/// keeps the harmless default (it cannot drop column defaults without a table rebuild).
///
/// `down()` requires SQLite ≥ 3.35 for `DROP COLUMN` (satisfied by bundled `libsqlite3-sys`).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ApiModelAliases {
  Table,
  Name,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .add_column(
            ColumnDef::new(ApiModelAliases::Name)
              .text()
              .not_null()
              .default(""),
          )
          .to_owned(),
      )
      .await?;

    let conn = manager.get_connection();
    conn
      .execute_unprepared("UPDATE api_model_aliases SET name = id")
      .await?;

    if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
      conn
        .execute_unprepared("ALTER TABLE api_model_aliases ALTER COLUMN name DROP DEFAULT")
        .await?;
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(ApiModelAliases::Table)
          .drop_column(ApiModelAliases::Name)
          .to_owned(),
      )
      .await
  }
}
