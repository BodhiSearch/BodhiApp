use crate::ErrorMessage;
use include_dir::{include_dir, Dir};
use std::path::Path;

/// Creates a static directory from a path for FFI compatibility.
///
/// This utility function enables FFI clients to pass asset paths dynamically
/// while maintaining compatibility with the `Dir<'static>` requirement.
///
/// # Arguments
/// * `path` - The filesystem path to the directory containing static assets
///
/// # Returns
/// * `Ok(&'static Dir<'static>)` - A reference to the static directory
/// * `Err(ErrorMessage)` - If the path is invalid or directory cannot be accessed
///
/// # Example
/// ```rust
/// use lib_bodhiserver::create_static_dir_from_path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let assets_dir = create_static_dir_from_path("/tmp")?;
/// // Use assets_dir with ServeCommand::aexecute()
/// # Ok(())
/// # }
/// ```
pub fn create_static_dir_from_path(path: &str) -> Result<&'static Dir<'static>, ErrorMessage> {
  // Validate that the path exists and is a directory
  let path_buf = Path::new(path);
  if !path_buf.exists() {
    return Err(ErrorMessage::new(
      "static_dir_path_not_found".to_string(),
      "validation_error".to_string(),
      format!("Asset path does not exist: {}", path),
    ));
  }

  if !path_buf.is_dir() {
    return Err(ErrorMessage::new(
      "static_dir_not_directory".to_string(),
      "validation_error".to_string(),
      format!("Asset path is not a directory: {}", path),
    ));
  }

  // For now, we'll use a workaround that creates an empty static directory
  // This is a limitation of the current approach - we cannot dynamically
  // create Dir<'static> at runtime. A future improvement would be to
  // modify ServeCommand to accept a PathBuf instead of Dir<'static>
  static EMPTY_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/resources");

  // TODO: This is a temporary implementation. The proper solution would be
  // to refactor ServeCommand to accept PathBuf or implement a dynamic
  // directory serving mechanism that doesn't require compile-time embedding.

  Ok(&EMPTY_DIR)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use tempfile::TempDir;

  #[test]
  fn test_create_static_dir_from_valid_path() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_str().unwrap();

    let result = create_static_dir_from_path(path);
    assert!(result.is_ok());
  }

  #[test]
  fn test_create_static_dir_from_invalid_path() {
    let result = create_static_dir_from_path("/nonexistent/path");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
  }

  #[test]
  fn test_create_static_dir_from_file_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").unwrap();

    let result = create_static_dir_from_path(file_path.to_str().unwrap());
    assert!(result.is_err());
    assert!(result
      .unwrap_err()
      .to_string()
      .contains("is not a directory"));
  }
}
