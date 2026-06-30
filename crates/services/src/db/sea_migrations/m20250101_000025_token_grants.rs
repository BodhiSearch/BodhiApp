use sea_orm_migration::prelude::*;

/// Adds per-resource grants to `api_tokens`: a NOT-NULL `grants` JSON text column
/// (defaulting to **deny-everything** / least-privilege) and a nullable
/// `last_used_at` timestamp. RLS is already enabled on the table (migration 000005),
/// so adding columns needs no policy change.
///
/// The column default is fail-closed and matches `services::default_grants_json()`;
/// in practice the insert path always writes `grants` explicitly, so this default
/// is the safety floor, not the value real tokens carry.
///
/// `down()` requires SQLite ≥ 3.35 for `DROP COLUMN` (satisfied by bundled `libsqlite3-sys`).
#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_GRANTS: &str = r#"{"version":"1","models_list":false,"models":{"type":"specific","ids":[]},"mcps_list":false,"mcps":{"type":"specific","ids":[]}}"#;

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

#[cfg(test)]
mod tests {
  use super::DEFAULT_GRANTS;
  use pretty_assertions::assert_eq;

  #[test]
  fn migration_default_grants_matches_code_default() {
    // The frozen migration literal must stay in sync with the code default so a
    // drift (e.g. the code default changing) is caught here rather than silently
    // leaving the column default behind.
    assert_eq!(crate::default_grants_json(), DEFAULT_GRANTS);
  }
}
