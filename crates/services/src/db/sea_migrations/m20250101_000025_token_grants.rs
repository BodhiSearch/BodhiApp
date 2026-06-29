use sea_orm_migration::prelude::*;

/// Adds per-resource grants to `api_tokens`: a NOT-NULL `grants` JSON text column
/// (defaulting to all-access so any pre-existing token keeps working) and a nullable
/// `last_used_at` timestamp. RLS is already enabled on the table (migration 000005),
/// so adding columns needs no policy change.
///
/// `down()` requires SQLite ≥ 3.35 for `DROP COLUMN` (satisfied by bundled `libsqlite3-sys`).
#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_GRANTS: &str = r#"{"version":"1","list_models":true,"models":{"type":"all"},"list_mcps":true,"mcps":{"type":"all"}}"#;

#[derive(DeriveIden)]
enum ApiTokens {
  Table,
  Grants,
  LastUsedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // SQLite requires separate ALTER TABLE statements per column.
    manager
      .alter_table(
        Table::alter()
          .table(ApiTokens::Table)
          .add_column(
            ColumnDef::new(ApiTokens::Grants)
              .text()
              .not_null()
              .default(DEFAULT_GRANTS),
          )
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(ApiTokens::Table)
          .add_column(
            ColumnDef::new(ApiTokens::LastUsedAt)
              .timestamp_with_time_zone()
              .null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(ApiTokens::Table)
          .drop_column(ApiTokens::Grants)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(ApiTokens::Table)
          .drop_column(ApiTokens::LastUsedAt)
          .to_owned(),
      )
      .await
  }
}
