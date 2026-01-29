use objs::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Database error: {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
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
#[error("Database migration error: {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
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
#[error("Item '{id}' of type '{item_type}' not found.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::NotFound)]
pub struct ItemNotFound {
  id: String,
  item_type: String,
}

#[cfg(test)]
mod tests {
  use super::*;
  use objs::AppError;

  #[test]
  fn test_item_not_found_error() {
    let error = ItemNotFound::new("1".to_string(), "user".to_string());
    assert_eq!(error.error_type(), objs::ErrorType::NotFound.to_string());
    assert_eq!(error.code(), "item_not_found");
  }
}
