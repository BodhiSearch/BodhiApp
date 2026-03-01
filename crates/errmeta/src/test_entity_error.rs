use crate::{AppError, EntityError};
use rstest::rstest;
use std::collections::HashMap;

#[rstest]
fn test_entity_error_not_found() {
  let error = EntityError::NotFound("User".to_string());
  assert_eq!("User not found.", error.to_string());
  assert_eq!("entity_error-not_found", error.code());
  assert_eq!(404, error.status());
  assert_eq!("not_found_error", error.error_type());
  assert_eq!(
    HashMap::from([("var_0".to_string(), "User".to_string())]),
    error.args()
  );
}
