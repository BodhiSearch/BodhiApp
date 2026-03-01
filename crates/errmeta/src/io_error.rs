use crate::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum IoError {
  #[error("File operation failed: {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Io {
    #[from]
    source: std::io::Error,
  },

  #[error("File operation failed for '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  WithPath {
    #[source]
    source: std::io::Error,
    path: String,
  },

  #[error("Failed to create folder '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DirCreate {
    #[source]
    source: std::io::Error,
    path: String,
  },

  #[error("Failed to read file '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FileRead {
    #[source]
    source: std::io::Error,
    path: String,
  },

  #[error("Failed to write file '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FileWrite {
    #[source]
    source: std::io::Error,
    path: String,
  },

  #[error("Failed to delete file '{path}': {source}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FileDelete {
    #[source]
    source: std::io::Error,
    path: String,
  },
}

impl IoError {
  pub fn with_path(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::WithPath {
      source,
      path: path.into(),
    }
  }

  pub fn dir_create(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::DirCreate {
      source,
      path: path.into(),
    }
  }

  pub fn file_read(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::FileRead {
      source,
      path: path.into(),
    }
  }

  pub fn file_write(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::FileWrite {
      source,
      path: path.into(),
    }
  }

  pub fn file_delete(source: std::io::Error, path: impl Into<String>) -> Self {
    Self::FileDelete {
      source,
      path: path.into(),
    }
  }
}

#[cfg(test)]
#[path = "test_io_error.rs"]
mod test_io_error;
