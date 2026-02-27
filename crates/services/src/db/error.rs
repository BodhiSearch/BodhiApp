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
#[error("SeaORM database error: {source}.")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::InternalServer)]
pub struct SeaOrmDbError {
  #[from]
  pub source: sea_orm::DbErr,
}

impl PartialEq for SeaOrmDbError {
  fn eq(&self, other: &Self) -> bool {
    self.source.to_string() == other.source.to_string()
  }
}

impl Eq for SeaOrmDbError {}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DbError {
  #[error(transparent)]
  SeaOrmError(#[from] SeaOrmDbError),
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
  #[error("Multiple application instances found, expected at most one.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MultipleAppInstance,
  #[error("Data conversion error: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Conversion(String),
}

impl_error_from!(
  ::sea_orm::DbErr,
  DbError::SeaOrmError,
  crate::db::SeaOrmDbError
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
