use objs::ErrorType;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("sqlx_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct SqlxError {
  #[from]
  pub source: sqlx::Error,
}

impl PartialEq for SqlxError {
  fn eq(&self, other: &Self) -> bool {
    self.source.to_string() == other.source.to_string()
  }
}

impl Eq for SqlxError {}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("sqlx_migrate_error")]
#[error_meta(error_type = ErrorType::InternalServer, status = 500)]
pub struct SqlxMigrateError {
  #[from]
  source: sqlx::migrate::MigrateError,
}

impl PartialEq for SqlxMigrateError {
  fn eq(&self, other: &Self) -> bool {
    self.source.to_string() == other.source.to_string()
  }
}

impl Eq for SqlxMigrateError {}

#[cfg(test)]
mod tests {
  use super::*;
  use fluent::{FluentBundle, FluentResource};
  use objs::test_utils::{assert_error_message, fluent_bundle};
  use rstest::rstest;
use sqlx::migrate::MigrateError;

  #[rstest]
  fn test_sqlx_error_message(fluent_bundle: FluentBundle<FluentResource>) {
    let error = SqlxError::new(sqlx::Error::RowNotFound);
    let message = "no rows returned by a query that expected to return at least one row";
    assert_error_message(&fluent_bundle, &error.code(), error.args(), &message);
  }

  #[rstest]
  fn test_sqlx_migrate_error_message(fluent_bundle: FluentBundle<FluentResource>) {
    let error = SqlxMigrateError::new(MigrateError::VersionMissing(1));
    let message = "migration 1 was previously applied but is missing in the resolved migrations";
    assert_error_message(&fluent_bundle, &error.code(), error.args(), &message);
  }
}
