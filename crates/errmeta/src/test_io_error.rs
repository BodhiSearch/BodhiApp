use crate::{AppError, IoError};
use rstest::rstest;
use std::collections::HashMap;

#[rstest]
#[case(
  IoError::from(std::io::Error::new(std::io::ErrorKind::NotFound, "not found")),
  "File operation failed: not found.",
  "io_error-io",
  500,
  "internal_server_error",
  HashMap::from([("source".to_string(), "not found".to_string())])
)]
#[case(
  IoError::with_path(
    std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
    "/tmp/test"
  ),
  "File operation failed for '/tmp/test': not found.",
  "io_error-with_path",
  500,
  "internal_server_error",
  HashMap::from([
    ("source".to_string(), "not found".to_string()),
    ("path".to_string(), "/tmp/test".to_string()),
  ])
)]
#[case(
  IoError::dir_create(
    std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    "/tmp/dir"
  ),
  "Failed to create folder '/tmp/dir': denied.",
  "io_error-dir_create",
  500,
  "internal_server_error",
  HashMap::from([
    ("source".to_string(), "denied".to_string()),
    ("path".to_string(), "/tmp/dir".to_string()),
  ])
)]
#[case(
  IoError::file_read(
    std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
    "/tmp/file"
  ),
  "Failed to read file '/tmp/file': not found.",
  "io_error-file_read",
  500,
  "internal_server_error",
  HashMap::from([
    ("source".to_string(), "not found".to_string()),
    ("path".to_string(), "/tmp/file".to_string()),
  ])
)]
#[case(
  IoError::file_write(
    std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    "/tmp/file"
  ),
  "Failed to write file '/tmp/file': denied.",
  "io_error-file_write",
  500,
  "internal_server_error",
  HashMap::from([
    ("source".to_string(), "denied".to_string()),
    ("path".to_string(), "/tmp/file".to_string()),
  ])
)]
#[case(
  IoError::file_delete(
    std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
    "/tmp/file"
  ),
  "Failed to delete file '/tmp/file': not found.",
  "io_error-file_delete",
  500,
  "internal_server_error",
  HashMap::from([
    ("source".to_string(), "not found".to_string()),
    ("path".to_string(), "/tmp/file".to_string()),
  ])
)]
fn test_io_error_variants(
  #[case] error: IoError,
  #[case] expected_message: &str,
  #[case] expected_code: &str,
  #[case] expected_status: u16,
  #[case] expected_error_type: &str,
  #[case] expected_args: HashMap<String, String>,
) {
  assert_eq!(expected_message, error.to_string());
  assert_eq!(expected_code, error.code());
  assert_eq!(expected_status, error.status());
  assert_eq!(expected_error_type, error.error_type());
  assert_eq!(expected_args, error.args());
}
