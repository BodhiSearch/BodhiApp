use objs::{impl_error_from, AppError, ErrorType};

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

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DbError {
  #[error(transparent)]
  SqlxError(#[from] SqlxError),
  #[error(transparent)]
  SqlxMigrateError(#[from] SqlxMigrateError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::BadRequest, code="db_error-strum_parse", args_delegate = false)]
  StrumParse(#[from] strum::ParseError),
  #[error("Invalid token: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  TokenValidation(String),
  #[error("Encryption error: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  EncryptionError(String),
  #[error("Prefix '{0}' is already used by another API model.")]
  #[error_meta(error_type = ErrorType::BadRequest, code = "db_error-prefix_exists")]
  PrefixExists(String),
  #[error("Item '{id}' of type '{item_type}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ItemNotFound { id: String, item_type: String },
}

impl_error_from!(::sqlx::Error, DbError::SqlxError, crate::db::SqlxError);
impl_error_from!(
  ::sqlx::migrate::MigrateError,
  DbError::SqlxMigrateError,
  crate::db::SqlxMigrateError
);

#[cfg(test)]
mod tests {
  use objs::AppError;

  #[test]
  fn test_item_not_found_error() {
    let error = crate::db::DbError::ItemNotFound {
      id: "1".to_string(),
      item_type: "user".to_string(),
    };
    assert_eq!(error.error_type(), objs::ErrorType::NotFound.to_string());
    assert_eq!(error.code(), "db_error-item_not_found");
  }
}
