use objs::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("sqlx_error")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
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
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer, status = 500)]
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

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("item_not_found")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::NotFound, status = 404)]
pub struct ItemNotFound {
  id: String,
  item_type: String,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_utils::setup_l10n_services;
  use objs::{test_utils::assert_error_message, FluentLocalizationService};
  use rstest::rstest;
  use sqlx::migrate::MigrateError;
  use std::sync::Arc;

  #[rstest]
  #[case::sqlx(
    &SqlxError::new(sqlx::Error::RowNotFound),
    "no rows returned by a query that expected to return at least one row"
  )]
  #[case::migration(
    &SqlxMigrateError::new(MigrateError::VersionMissing(1)),
    "migration 1 was previously applied but is missing in the resolved migrations"
  )]
  #[case::item_not_found(
    &ItemNotFound::new("1".to_string(), "user".to_string()),
    "item '1' of type 'user' not found in db"
  )]
  #[serial_test::serial(localization)]
  fn test_sqlx_error_message(
    #[from(setup_l10n_services)] localization_service: Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] message: String,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), &message);
  }
}
